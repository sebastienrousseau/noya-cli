#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Private archive staging must refuse path escape before writing files."""

import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('consumer', ROOT / 'scripts/check-distribution-consumer.py')
CONSUMER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CONSUMER)


class DistributionConsumer(unittest.TestCase):
    def test_tar_and_zip_refuse_parent_traversal_before_staging(self):
        for kind in ['tar', 'zip']:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                archive = root / ('bundle.' + kind)
                if kind == 'tar':
                    with tarfile.open(archive, 'w') as bundle:
                        member = tarfile.TarInfo('bundle/../escaped')
                        member.size = 4
                        bundle.addfile(member, io.BytesIO(b'data'))
                else:
                    with zipfile.ZipFile(archive, 'w') as bundle:
                        bundle.writestr('bundle/../escaped', b'data')
                artifact = {'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(), 'directory': 'bundle'}
                with self.assertRaises(ValueError):
                    CONSUMER.install(archive, artifact, root / 'installed')
                self.assertFalse((root / 'installed').exists())
                self.assertFalse((root / 'escaped').exists())

    def test_tar_symlinks_are_refused_before_staging(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / 'bundle.tar'
            with tarfile.open(archive, 'w') as bundle:
                member = tarfile.TarInfo('bundle/link')
                member.type = tarfile.SYMTYPE
                member.linkname = '../outside'
                bundle.addfile(member)
            artifact = {'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(), 'directory': 'bundle'}
            with self.assertRaises(ValueError):
                CONSUMER.install(archive, artifact, root / 'installed')
            self.assertFalse((root / 'installed').exists())


if __name__ == '__main__':
    unittest.main()
