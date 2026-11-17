#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Exercise a published native bundle, cache refusals, offline restore and removal."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import subprocess
import tarfile
import urllib.request
import zipfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def install(archive, artifact, destination):
    if digest(archive) != artifact['sha256']:
        raise ValueError('archive digest mismatch')
    if destination.exists():
        raise FileExistsError(destination)
    if archive.suffix == '.zip':
        bundle = zipfile.ZipFile(archive)
        members = [(item.filename, item.file_size, item.is_dir(), item) for item in bundle.infolist()]
        if any((item.external_attr >> 16) & 0o170000 == 0o120000 for item in bundle.infolist()):
            bundle.close()
            raise ValueError('archive contains a symlink')
        read = lambda item: bundle.open(item)
    else:
        bundle = tarfile.open(archive)
        members = [(item.name, item.size, item.isdir(), item) for item in bundle.getmembers()]
        if any(not (item.isfile() or item.isdir()) for item in bundle.getmembers()):
            bundle.close()
            raise ValueError('archive contains a special file')
        read = lambda item: bundle.extractfile(item)
    try:
        if sum(size for _, size, _, _ in members) > 128 * 1024 * 1024:
            raise ValueError('native bundle exceeds the staging budget')
        for name, _, _, _ in members:
            path = PurePosixPath(name)
            if path.is_absolute() or '..' in path.parts or '\\' in name or not path.parts or path.parts[0] != artifact['directory']:
                raise ValueError('archive member is outside its recorded directory')
        destination.mkdir(parents=True)
        for name, _, directory, item in members:
            relative = PurePosixPath(name).relative_to(artifact['directory'])
            target = destination.joinpath(*relative.parts)
            if directory:
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            with read(item) as source, target.open('xb') as output:
                shutil.copyfileobj(source, output)
            if target.name in ['noyafmt', 'noyavalidate']:
                target.chmod(0o755)
    finally:
        bundle.close()


def exercise(directory, version):
    suffix = '.exe' if os.name == 'nt' else ''
    tools = {name: directory / (name + suffix) for name in ['noyafmt', 'noyavalidate']}
    versions = {}
    for name, binary in tools.items():
        result = subprocess.run([str(binary), '--version'], capture_output=True, text=True, timeout=20, check=True)
        if result.stdout.strip() != f'{name} {version}':
            raise AssertionError(f'{name}: unexpected version {result.stdout!r}')
        versions[name] = result.stdout.strip()
    formatted = subprocess.run([str(tools['noyafmt']), '--stdin'], input='a:   1\n', capture_output=True, text=True, timeout=20)
    if formatted.returncode or formatted.stdout != 'a: 1\n':
        raise AssertionError(f'installed formatter failed: {formatted}')
    invalid = subprocess.run([str(tools['noyavalidate'])], input='a: [\n', capture_output=True, text=True, timeout=20)
    if invalid.returncode != 1 or not invalid.stderr:
        raise AssertionError(f'installed validator did not refuse malformed input: {invalid}')
    return versions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--provenance', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    record = json.loads((root / 'pkg/release-artifacts.json').read_text())
    arch = {'arm64': 'aarch64', 'aarch64': 'aarch64', 'amd64': 'x86_64', 'x86_64': 'x86_64'}[platform.machine().lower()]
    suffix = {'Darwin': 'apple-darwin', 'Linux': 'unknown-linux-musl', 'Windows': 'pc-windows-msvc'}[platform.system()]
    target = arch + '-' + suffix
    artifact = record['artifacts'][target]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {'status': 'failed', 'target': target, 'version': record['version'], 'release_source': record['source'],
              'recipe_record_sha256': digest(root / 'pkg/release-artifacts.json'), 'consumer_sha256': digest(Path(__file__)),
              'scope': 'Actual published native archive in private staging; no package-manager or candidate-registry installation claim.'}
    try:
        archive = output / artifact['url'].rsplit('/', 1)[1]
        with urllib.request.urlopen(artifact['url'], timeout=60) as response, archive.open('xb') as stream:
            shutil.copyfileobj(response, stream)
        if digest(archive) != artifact['sha256']:
            raise ValueError('download digest mismatch')
        report['archive_sha256'] = digest(archive)
        if args.provenance:
            with (output / 'provenance.json').open('w') as stdout, (output / 'provenance.stderr').open('w') as stderr:
                subprocess.run(['gh', 'attestation', 'verify', str(archive), '--repo', 'sebastienrousseau/noya-cli',
                                '--signer-workflow', 'sebastienrousseau/noya-cli/.github/workflows/release.yml',
                                '--source-ref', 'refs/tags/v' + record['version'], '--format', 'json'],
                               stdout=stdout, stderr=stderr, timeout=120, check=True)
            report['provenance_verified'] = True
        else:
            report['provenance_verified'] = False
        installed = output / 'installed'
        install(archive, artifact, installed)
        report['installed_versions'] = exercise(installed, record['version'])
        before = {str(p.relative_to(installed)): digest(p) for p in installed.rglob('*') if p.is_file()}
        corrupt = output / ('corrupt-' + archive.name)
        original = archive.read_bytes()
        corrupt.write_bytes(original[:-1] + bytes([original[-1] ^ 1]))
        for label, candidate, error in [('missing', output / 'missing', FileNotFoundError), ('corrupt', corrupt, ValueError)]:
            destination = output / (label + '-install')
            try:
                install(candidate, artifact, destination)
            except error:
                if destination.exists():
                    raise AssertionError(label + ' cache refusal wrote installation files')
            else:
                raise AssertionError(label + ' cache was accepted')
        after = {str(p.relative_to(installed)): digest(p) for p in installed.rglob('*') if p.is_file()}
        if before != after:
            raise AssertionError('cache refusal changed the existing installation')
        report['cache_refusals'] = ['missing', 'corrupt']
        restored = output / 'restored'
        install(archive, artifact, restored)
        report['offline_restored_versions'] = exercise(restored, record['version'])
        shutil.rmtree(installed)
        shutil.rmtree(restored)
        if installed.exists() or restored.exists():
            raise AssertionError('private uninstall left files behind')
        report['private_uninstall_passed'] = True
        report['status'] = 'passed'
    finally:
        (output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
