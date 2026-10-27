#!/usr/bin/env python3
"""Stage and verify the four release assets using only Python's standard library."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import struct
import subprocess
import tomllib
import zipfile

VERSION = '1.3.0'
ASSETS = {
    'linux': 'IDE_C--',
    'appimage': 'IDE_C--.AppImage',
    'msys2': 'IDE_C--_windows_msys2.zip',
    'mingw': 'IDE_C--_windows_mingw.zip',
}
# Windows-provided libraries. Everything else must be included in the portable ZIP.
SYSTEM_DLLS = set('''advapi32 avrt bcrypt bcryptprimitives cabinet cfgmgr32 comctl32 comdlg32
crypt32 cryptbase cryptsp d2d1 d3d9 d3d11 d3d12 dbghelp dcomp dnsapi dsound dwmapi dwrite
 dxgi faultrep gdi32 gdiplus glu32 hid imm32 iphlpapi kernel32 kernelbase mpr msasn1
msimg32 msvcrt msvcp_win ncrypt netapi32 normaliz ntdll ole32 oleacc oleaut32 opengl32
powrprof propsys psapi rpcrt4 rsaenh secur32 setupapi shcore shell32 shlwapi user32 userenv
ucrtbase usp10 uxtheme version wer windowscodecs winhttp wininet winmm winscard winspool
wintrust win32u wldap32 ws2_32 wtsapi32'''.split())


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def pe_imports(data):
    """Read the PE32+ import table, rejecting truncated or non-x86-64 images."""
    try:
        if data[:2] != b'MZ':
            raise ValueError('Missing PE executable header')
        pe = struct.unpack_from('<I', data, 0x3c)[0]
        if data[pe:pe + 4] != b'PE\0\0':
            raise ValueError('Missing PE signature')
        machine, sections = struct.unpack_from('<HH', data, pe + 4)
        if machine != 0x8664:
            raise ValueError('Windows binary must be x86-64')
        optional_size = struct.unpack_from('<H', data, pe + 20)[0]
        optional = pe + 24
        if optional_size < 128 or struct.unpack_from('<H', data, optional)[0] != 0x20b:
            raise ValueError('Expected a PE32+ optional header')
        headers_size = struct.unpack_from('<I', data, optional + 60)[0]

        def rva_offset(rva):
            if rva < headers_size and rva < len(data):
                return rva
            for index in range(sections):
                section = optional + optional_size + 40 * index
                _, address, raw_size, raw_pointer = struct.unpack_from('<IIII', data, section + 8)
                if address <= rva < address + raw_size:
                    offset = raw_pointer + rva - address
                    if offset < len(data):
                        return offset
            raise ValueError('Invalid PE import address')

        address, size = struct.unpack_from('<II', data, optional + 120)
        if not address:
            return set()
        start = rva_offset(address)
        imports = set()
        for index in range(min(size // 20, len(data) // 20)):
            entry = struct.unpack_from('<IIIII', data, start + index * 20)
            if not any(entry):
                return imports
            name_start = rva_offset(entry[3])
            name_end = data.find(b'\0', name_start, name_start + 260)
            if name_end == -1:
                raise ValueError('Unterminated PE import name')
            imports.add(data[name_start:name_end].decode('ascii').lower())
        raise ValueError('Unterminated PE import table')
    except (struct.error, UnicodeDecodeError) as error:
        raise ValueError('Truncated or invalid PE binary') from error


def verify_windows(path):
    with zipfile.ZipFile(path) as archive:
        bad = archive.testzip()
        if bad:
            raise ValueError(f'Corrupt ZIP member: {bad}')
        names = archive.namelist()
        for name in names:
            parts = PurePosixPath(name).parts
            if not parts or '..' in parts or name.startswith('/') or '\\' in name or ':' in name:
                raise ValueError(f'Unsafe ZIP path: {name}')
        lower_names = [name.lower() for name in names]
        if len(lower_names) != len(set(lower_names)):
            raise ValueError('Duplicate ZIP member')
        if 'ide_c--.exe' not in lower_names:
            raise ValueError('Missing root IDE_C--.exe')
        bundled = {name.lower() for name in names if '/' not in name}
        for name in names:
            if not name.lower().endswith(('.dll', '.exe')):
                continue
            for dependency in pe_imports(archive.read(name)):
                system = (
                    dependency.removesuffix('.dll').removesuffix('.drv') in SYSTEM_DLLS
                    or dependency.startswith(('api-ms-win-', 'ext-ms-win-'))
                )
                if not system and dependency not in bundled:
                    raise ValueError(f'Missing runtime DLL {dependency} required by {name}')


def verify_elf(path):
    with path.open('rb') as stream:
        header = stream.read(64)
    if len(header) < 64 or header[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', header, 18)[0] != 62:
        raise ValueError(f'Expected x86-64 ELF binary: {path.name}')


def verify_directory(root, sha, *, manifests=True):
    for platform, name in ASSETS.items():
        asset = root / name
        if not asset.is_file():
            raise ValueError(f'Missing asset: {name}')
        if manifests:
            manifest = root / f'build-{platform}.json'
            if not manifest.is_file():
                raise ValueError(f'Missing build provenance: {platform}')
            record = json.loads(manifest.read_text())
            if record.get('sha') != sha or record.get('version') != VERSION:
                raise ValueError(f'Mismatched source revision or version for {name}')
            if record.get('platform') != platform or record.get('asset') != name:
                raise ValueError(f'Mismatched asset provenance for {name}')
            if record.get('sha256') != digest(asset):
                raise ValueError(f'Invalid checksum for {name}')
        if name.endswith('.zip'):
            verify_windows(asset)
        else:
            verify_elf(asset)


def stage(platform, sha, root):
    if tomllib.loads(Path('Cargo.toml').read_text())['package']['version'] != VERSION:
        raise ValueError('Cargo package version does not match release version')
    actual_sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    if actual_sha != sha:
        raise ValueError('Checkout does not match requested source SHA')
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--', 'Cargo.toml', 'Cargo.lock', 'src'], check=True)
    name = ASSETS[platform]
    source = Path('releases' if name.endswith('.zip') else 'target/release') / name
    root.mkdir(parents=True, exist_ok=True)
    destination = root / name
    shutil.copy2(source, destination)
    record = {'sha': sha, 'version': VERSION, 'platform': platform, 'asset': name, 'sha256': digest(destination)}
    (root / f'build-{platform}.json').write_text(json.dumps(record, indent=2) + '\n')


def checksums(root):
    return ''.join(f'{digest(root / name)}  {name}\n' for name in ASSETS.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['stage', 'verify', 'downloaded'])
    parser.add_argument('--platform', choices=ASSETS)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--directory', type=Path, default=Path('release-artifacts'))
    args = parser.parse_args()
    if len(args.sha) != 40 or any(char not in '0123456789abcdef' for char in args.sha):
        parser.error('--sha must be a full lowercase commit SHA')
    if args.operation == 'stage':
        if not args.platform:
            parser.error('stage requires --platform')
        stage(args.platform, args.sha, args.directory)
    else:
        verify_directory(args.directory, args.sha, manifests=args.operation == 'verify')
        checksum_path = args.directory / 'SHA256SUMS'
        expected = checksums(args.directory)
        if args.operation == 'verify':
            checksum_path.write_text(expected)
        elif checksum_path.read_text() != expected:
            raise ValueError('Downloaded assets do not match SHA256SUMS')
        print(f'Verified all four x86-64 assets for {args.sha}')


if __name__ == '__main__':
    main()
