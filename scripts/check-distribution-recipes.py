#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Check channel sources against recorded, published native archive digests."""

import argparse
import json
from pathlib import Path
import re


def check(root):
    record = json.loads((root / 'pkg/release-artifacts.json').read_text())
    version = record['version']
    artifacts = record['artifacts']
    errors = []

    def require(condition, description):
        if not condition:
            errors.append(description)

    def archive(target):
        return artifacts[target]

    brew = (root / 'pkg/brew/noya-cli.rb').read_text()
    entries = re.findall(r'url "([^"]+)"\s+sha256 "([0-9a-f]+)"', brew)
    expected = {(archive(target)['url'], archive(target)['sha256']) for target in [
        'aarch64-apple-darwin', 'x86_64-apple-darwin',
        'aarch64-unknown-linux-musl', 'x86_64-unknown-linux-musl']}
    require(len(entries) == 4 and set(entries) == expected, 'Homebrew version, URLs or digests drifted')

    scoop = json.loads((root / 'pkg/scoop/noya-cli.json').read_text())
    require(scoop['version'] == version, 'Scoop version drifted')
    for name, target in [('64bit', 'x86_64-pc-windows-msvc'), ('arm64', 'aarch64-pc-windows-msvc')]:
        entry = scoop['architecture'][name]
        artifact = archive(target)
        require(entry['url'] == artifact['url'], f'Scoop {name} URL drifted')
        require(entry['hash'] == artifact['sha256'], f'Scoop {name} digest drifted')
        require(entry['extract_dir'] == artifact['directory'], f'Scoop {name} extraction directory drifted')

    aur = (root / 'pkg/aur/PKGBUILD').read_text()
    require(f'pkgver={version}\n' in aur, 'AUR version drifted')
    for arch in ['x86_64', 'aarch64']:
        artifact = archive(f'{arch}-unknown-linux-musl')
        require(f"sha256sums_{arch}=('{artifact['sha256']}')" in aur, f'AUR {arch} digest drifted')
        require(f'source_{arch}=("$url/releases/download/v$pkgver/noya-cli-$pkgver-{arch}-unknown-linux-musl.tar.gz")' in aur,
                f'AUR {arch} URL drifted')

    action = (root / 'action.yml').read_text()
    match = re.search(r'^  version:\n(?:(?:    .*|)\n)*?    default: "([^"]+)"', action, re.MULTILINE)
    require(match is not None and match.group(1) == version, 'Action default differs from the recorded published release')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    errors = check(args.root)
    for error in errors:
        print(error)
    if not errors:
        print('Distribution recipes match all six recorded published archives.')
    raise SystemExit(bool(errors))


if __name__ == '__main__':
    main()
