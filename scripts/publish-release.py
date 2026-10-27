#!/usr/bin/env python3
"""Create the 1.3 draft without moving existing tags or modifying published releases."""
import argparse
import json
import os
from pathlib import Path
import subprocess

TAG = '1.3'


def api(endpoint, method='GET', payload=None):
    command = ['gh', 'api', endpoint, '--method', method]
    if payload is not None:
        command += ['--input', '-']
    result = subprocess.run(command, input=json.dumps(payload) if payload is not None else None, text=True, capture_output=True)
    if result.returncode:
        # gh emits API status codes in stderr. Only an explicit 404 means absent.
        if method == 'GET' and '(HTTP 404)' in result.stderr:
            return None
        raise RuntimeError(result.stderr.strip())
    return json.loads(result.stdout) if result.stdout.strip() else None


def check_tag(repo, sha):
    ref = api(f'repos/{repo}/git/ref/tags/{TAG}')
    if ref is None:
        return False
    obj = ref['object']
    while obj['type'] == 'tag':
        obj = api(f'repos/{repo}/git/tags/{obj["sha"]}')['object']
    if obj['type'] != 'commit' or obj['sha'] != sha:
        raise ValueError('Existing tag 1.3 points to another revision; refusing to move it')
    return True


def find_release(repo):
    # The tag lookup endpoint may exclude drafts. List authenticated releases instead.
    page = 1
    while True:
        releases = api(f'repos/{repo}/releases?per_page=100&page={page}')
        if releases is None:
            raise RuntimeError('Cannot list repository releases')
        for release in releases:
            if release['tag_name'] == TAG:
                return release
        if len(releases) < 100:
            return None
        page += 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['draft', 'publish'])
    parser.add_argument('--sha', required=True)
    args = parser.parse_args()
    if len(args.sha) != 40 or any(c not in '0123456789abcdef' for c in args.sha):
        parser.error('--sha must be a full lowercase commit SHA')
    repo = os.environ['GH_REPO']
    tag_exists = check_tag(repo, args.sha)
    release = find_release(repo)
    if release and not release['draft']:
        raise ValueError('Release 1.3 is already published; refusing to replace its assets')
    if args.operation == 'draft':
        # Create the reference explicitly: draft releases may defer tag creation.
        if not tag_exists:
            api(f'repos/{repo}/git/refs', 'POST', {'ref': f'refs/tags/{TAG}', 'sha': args.sha})
        body = Path('docs/releases/1.3.md').read_text()
        body += f'\n\nSource commit: `{args.sha}`\n'
        payload = {'tag_name': TAG, 'target_commitish': args.sha, 'name': 'Semantic Analyzer', 'body': body, 'draft': True, 'prerelease': False}
        if release:
            api(f'repos/{repo}/releases/{release["id"]}', 'PATCH', payload)
        else:
            api(f'repos/{repo}/releases', 'POST', payload)
    else:
        if not tag_exists or not release:
            raise ValueError('Missing validated tag or draft')
        api(f'repos/{repo}/releases/{release["id"]}', 'PATCH', {'draft': False, 'prerelease': False, 'make_latest': 'true'})


if __name__ == '__main__':
    main()
