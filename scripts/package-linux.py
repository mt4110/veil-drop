#!/usr/bin/env python3
"""Package the locked Linux release binary and the selected dependency notices."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import zipfile


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def write_archive(bundle, archive):
    # Rust distribution files can have epoch timestamps. ZIP starts in 1980;
    # clamp only unsupported timestamps, retaining bytes and executable modes.
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED,
                         strict_timestamps=False) as z:
        for path in sorted(bundle.rglob('*')):
            if path.is_file():
                z.write(path, path.relative_to(bundle).as_posix())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = 'x86_64-unknown-linux-gnu'
    metadata = json.loads(output('cargo', 'metadata', '--locked', '--format-version', '1',
                                 '--filter-platform', target))
    packages = {p['id']: p for p in metadata['packages']}
    nodes = {n['id']: n for n in metadata['resolve']['nodes']}
    root_id = metadata['resolve']['root']
    assert packages[root_id]['name'] == 'veil-drop'
    version = packages[root_id]['version']
    binary = args.binary.resolve(strict=True)
    header = binary.read_bytes()[:20]
    if header[:6] != b'\x7fELF\x02\x01' or header[18:20] != b'\x3e\x00':
        raise ValueError('Expected little-endian ELF64 x86_64')
    if output(str(binary), '--version') != 'veil-drop ' + version:
        raise ValueError('Binary version differs from package version')
    revision = output('git', 'rev-parse', 'HEAD')
    if output('git', 'status', '--porcelain', '--untracked-files=no'):
        raise ValueError('Tracked source must be clean before packaging')
    args.out.mkdir(parents=True, exist_ok=False)
    bundle = args.out / 'bundle'
    bundle.mkdir()
    shutil.copy2(binary, bundle / 'veil-drop')
    for name in ['LICENSE', 'SECURITY.md']:
        shutil.copy2(root / name, bundle / name)
    shutil.copy2(root / 'docs/install-linux.md', bundle / 'INSTALL.md')
    selected = set()
    pending = [root_id]
    while pending:
        package_id = pending.pop()
        if package_id in selected:
            continue
        selected.add(package_id)
        for dependency in nodes[package_id]['deps']:
            if any(k['kind'] in (None, 'build') for k in dependency['dep_kinds']):
                pending.append(dependency['pkg'])
    inventory = []
    for package_id in sorted(selected):
        if package_id == root_id:
            continue
        package = packages[package_id]
        directory = Path(package['manifest_path']).parent
        texts = {p for p in directory.iterdir() if p.is_file()
                 and re.match(r'(?i)^(licen[cs]e|copying|notice)([._-].*)?$', p.name)}
        if package.get('license_file'):
            texts.add(directory / package['license_file'])
        if not texts or not package.get('license'):
            raise ValueError('Review missing license information: ' + package['name'])
        dest = bundle / 'third_party' / (package['name'] + '-' + package['version'])
        dest.mkdir(parents=True)
        for text in sorted(texts):
            shutil.copy2(text, dest / text.name)
        inventory.append({k: package.get(k) for k in
                          ['name', 'version', 'license', 'source', 'checksum']})
    sysroot = Path(output('rustc', '--print', 'sysroot'))
    runtime = sysroot / 'share/doc/rust'
    dest = bundle / 'third_party/rust-runtime'
    dest.mkdir(parents=True)
    for name in ['COPYRIGHT-library.html', 'COPYRIGHT.html']:
        shutil.copy2(runtime / name, dest / name)
    licenses = runtime / 'licenses'
    for name in ['MIT.txt', 'Apache-2.0.txt', 'LLVM-exception.txt']:
        if not (licenses / name).is_file():
            raise ValueError('Missing Rust runtime license: ' + name)
    shutil.copytree(licenses, dest / 'licenses')
    manifest = {'source_revision': revision, 'target': target, 'version': version,
                'rustc': output('rustc', '-vV'), 'dependencies': inventory,
                'cargo_lock_sha256': hashlib.sha256((root / 'Cargo.lock').read_bytes()).hexdigest()}
    (bundle / 'BUILD.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (bundle / 'THIRD_PARTY_NOTICES.md').write_text(
        '# Third-party notices\n\nSelected normal/build dependencies for the Linux target.\n'
        'Original license and notice texts are retained in third_party/.\n'
        'Rust runtime texts and COPYRIGHT-library.html are included separately.\n'
        'System shared libraries are not redistributed. See INSTALL.md for runtime scope.\n\n'
        + ''.join('- {name} {version}: {license}\n'.format(**p) for p in inventory))
    sums = []
    for path in sorted(bundle.rglob('*')):
        if path.is_file():
            sums.append(hashlib.sha256(path.read_bytes()).hexdigest()
                        + '  ' + path.relative_to(bundle).as_posix())
    (bundle / 'SHA256SUMS').write_text('\n'.join(sums) + '\n')
    archive = args.out / ('veil-drop-' + version + '-' + target + '.zip')
    write_archive(bundle, archive)
    (args.out / 'SHA256SUMS').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()
                                      + '  ' + archive.name + '\n')
    print('Created Linux candidate and notices:', archive.name)


if __name__ == '__main__':
    main()
