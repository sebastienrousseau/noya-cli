#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Pre-commit hook that runs `noyafmt --check` against every
# staged YAML file. Mirrors the rustfmt / prettier idiom:
# pre-commit refuses the commit if any file would change under
# the formatter; the developer runs `noyafmt --write <files>`
# to auto-fix.
#
# Install:
#   cp crates/noya-cli/examples/format-precommit.sh \
#      .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit

set -euo pipefail
IFS=$'\n\t'

# Collect every staged .yaml / .yml file. `--diff-filter=ACM`
# excludes deletions; `--name-only` gives plain paths.
mapfile -t FILES < <(
    git diff --cached --name-only --diff-filter=ACM \
    | grep -E '\.ya?ml$' || true
)

if [[ ${#FILES[@]} -eq 0 ]]; then
    exit 0
fi

if ! noyafmt --check "${FILES[@]}"; then
    echo
    echo "noyafmt found unformatted YAML in your staged files."
    echo "Re-run with --write to fix:"
    echo
    printf '  noyafmt --write '
    printf '%q ' "${FILES[@]}"
    echo
    exit 1
fi
