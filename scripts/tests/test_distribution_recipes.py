#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Exercise recipe drift refusal using the recorded published artifacts."""

import importlib.util
import json
from pathlib import Path
import re
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('recipes', ROOT / 'scripts/check-distribution-recipes.py')
RECIPES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RECIPES)


class DistributionRecipes(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        shutil.copytree(ROOT / 'pkg', self.root / 'pkg')
        shutil.copyfile(ROOT / 'action.yml', self.root / 'action.yml')

    def test_recorded_sources_agree(self):
        self.assertEqual(RECIPES.check(self.root), [])

    def test_each_channel_refuses_a_stale_version(self):
        for name in ['pkg/brew/noya-cli.rb', 'pkg/scoop/noya-cli.json', 'pkg/aur/PKGBUILD', 'action.yml']:
            with self.subTest(channel=name):
                path = self.root / name
                original = path.read_text()
                version = json.loads((self.root / 'pkg/release-artifacts.json').read_text())['version']
                path.write_text(original.replace(version, '0.0.1'))
                self.assertTrue(RECIPES.check(self.root))
                path.write_text(original)

    def test_scoop_refuses_another_architectures_digest(self):
        path = self.root / 'pkg/scoop/noya-cli.json'
        data = json.loads(path.read_text())
        data['architecture']['64bit']['hash'] = data['architecture']['arm64']['hash']
        path.write_text(json.dumps(data))
        self.assertIn('Scoop 64bit digest drifted', RECIPES.check(self.root))

    def test_scoop_refuses_an_incorrect_extraction_directory(self):
        path = self.root / 'pkg/scoop/noya-cli.json'
        data = json.loads(path.read_text())
        data['architecture']['arm64']['extract_dir'] += '-missing'
        path.write_text(json.dumps(data))
        self.assertIn('Scoop arm64 extraction directory drifted', RECIPES.check(self.root))

    def test_aur_refuses_stripping_or_debug_splitting_release_binaries(self):
        path = self.root / 'pkg/aur/PKGBUILD'
        original = re.sub(r'^options=.*\n', '', path.read_text(), flags=re.MULTILINE)
        message = 'AUR must preserve release binaries without stripping or debug splitting'
        path.write_text(original + "\noptions=('!strip' '!debug')\n")
        self.assertEqual(RECIPES.check(self.root), [])
        for options in ["('!strip')", "('!debug')", "('strip' '!strip' '!debug')", '()']:
            with self.subTest(options=options):
                path.write_text(original + '\noptions=' + options + '\n')
                self.assertIn(message, RECIPES.check(self.root))


if __name__ == '__main__':
    unittest.main()
