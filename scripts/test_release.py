"""Release verification must reject corrupt, mixed-revision and incomplete assets."""
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


def pe(imports=(), machine=0x8664):
    data = bytearray(2048)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 0x80)
    data[0x80:0x84] = b'PE\0\0'
    struct.pack_into('<HH', data, 0x84, machine, 1)
    struct.pack_into('<H', data, 0x94, 240)
    struct.pack_into('<H', data, 0x98, 0x20b)
    struct.pack_into('<I', data, 0x98 + 60, 512)
    struct.pack_into('<II', data, 0x98 + 120, 0x1000, (len(imports) + 1) * 20)
    section = 0x98 + 240
    struct.pack_into('<IIII', data, section + 8, 1536, 0x1000, 1536, 512)
    for i, name in enumerate(imports):
        pos = 1024 + i * 128
        raw = name.encode() + b'\0'
        data[pos:pos+len(raw)] = raw
        struct.pack_into('<I', data, 512 + i * 20 + 12, 0x1000 + pos - 512)
    return bytes(data)


class ReleaseVerification(unittest.TestCase):
    def test_windows_package_rejects_missing_runtime_dll(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp) / 'windows.zip'
            with zipfile.ZipFile(archive, 'w') as output:
                output.writestr('IDE_C--.exe', pe(['KERNEL32.dll', 'libgtk-4-1.dll']))
            with self.assertRaisesRegex(ValueError, 'libgtk-4-1.dll'):
                release.verify_windows(archive)
            with zipfile.ZipFile(archive, 'a') as output:
                output.writestr('libgtk-4-1.dll', pe(['api-ms-win-core-file-l1-1-0.dll', 'winspool.drv']))
            release.verify_windows(archive)

    def test_windows_package_rejects_wrong_architecture_and_unsafe_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp) / 'windows.zip'
            with zipfile.ZipFile(archive, 'w') as output:
                output.writestr('IDE_C--.exe', pe(machine=0x14c))
            with self.assertRaisesRegex(ValueError, 'x86-64'):
                release.verify_windows(archive)
            with zipfile.ZipFile(archive, 'w') as output:
                output.writestr('../IDE_C--.exe', pe())
            with self.assertRaisesRegex(ValueError, 'Unsafe'):
                release.verify_windows(archive)

    def test_release_rejects_mixed_commits_and_modified_assets(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            sha = 'a' * 40
            for platform, name in release.ASSETS.items():
                asset = root / name
                if name.endswith('.zip'):
                    with zipfile.ZipFile(asset, 'w') as output:
                        output.writestr('IDE_C--.exe', pe())
                else:
                    elf = bytearray(64)
                    elf[:6] = b'\x7fELF\x02\x01'
                    struct.pack_into('<H', elf, 18, 62)
                    asset.write_bytes(elf)
                manifest = {'sha': sha, 'version': '1.3.0', 'platform': platform, 'asset': name, 'sha256': release.digest(asset)}
                (root / f'build-{platform}.json').write_text(json.dumps(manifest))
            release.verify_directory(root, sha)
            manifest_path = root / 'build-linux.json'
            manifest = json.loads(manifest_path.read_text())
            manifest['sha'] = 'b' * 40
            manifest_path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'revision'):
                release.verify_directory(root, sha)
            manifest['sha'] = sha
            manifest_path.write_text(json.dumps(manifest))
            (root / 'IDE_C--').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                release.verify_directory(root, sha)

    def test_release_rejects_missing_asset(self):
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaisesRegex(ValueError, 'Missing'):
                release.verify_directory(Path(temp), 'a' * 40)


class PublicationGuards(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location('publication', Path(__file__).with_name('publish-release.py'))
        self.publication = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.publication)

    def test_existing_draft_is_updated_without_creating_a_duplicate(self):
        from unittest.mock import patch
        sha = 'a' * 40
        calls = []
        def api(endpoint, method='GET', payload=None):
            calls.append((endpoint, method, payload))
            if '/git/ref/' in endpoint:
                return {'object': {'type': 'commit', 'sha': sha}}
            if endpoint.endswith('/releases?per_page=100&page=1'):
                return [{'id': 42, 'tag_name': '1.3', 'draft': True}]
            return None
        with patch.object(self.publication, 'api', side_effect=api), patch.dict('os.environ', {'GH_REPO': 'owner/repo'}), patch('sys.argv', ['publish-release.py', 'draft', '--sha', sha]):
            self.publication.main()
        self.assertTrue(any(endpoint.endswith('/releases/42') and method == 'PATCH' for endpoint, method, _ in calls))
        self.assertFalse(any(endpoint.endswith('/releases') and method == 'POST' for endpoint, method, _ in calls))

    def test_published_release_is_never_replaced(self):
        from unittest.mock import patch
        sha = 'a' * 40
        def api(endpoint, method='GET', payload=None):
            self.assertEqual(method, 'GET')
            if '/git/ref/' in endpoint:
                return {'object': {'type': 'commit', 'sha': sha}}
            if endpoint.endswith('/releases?per_page=100&page=1'):
                return [{'id': 42, 'tag_name': '1.3', 'draft': False}]
            return None
        with patch.object(self.publication, 'api', side_effect=api), patch.dict('os.environ', {'GH_REPO': 'owner/repo'}), patch('sys.argv', ['publish-release.py', 'draft', '--sha', sha]):
            with self.assertRaisesRegex(ValueError, 'already published'):
                self.publication.main()

    def test_existing_tag_for_another_commit_is_never_moved(self):
        from unittest.mock import patch
        with patch.object(self.publication, 'api', return_value={'object': {'type': 'commit', 'sha': 'b' * 40}}):
            with self.assertRaisesRegex(ValueError, 'another revision'):
                self.publication.check_tag('owner/repo', 'a' * 40)


if __name__ == '__main__':
    unittest.main()
