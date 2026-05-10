#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# validate-pyproject.sh — gate Python project YAML configs
# against their schemas.
#
# Note: PEP 518's canonical project file is `pyproject.toml`,
# *not* YAML. This script targets the *adjacent* YAML configs
# Python projects ship: `mkdocs.yml` (MkDocs site config),
# `.pre-commit-config.yaml`, `tox.yaml` / `tox-config.yaml`
# variants, `cibuildwheel` overrides, and so on. Each has a
# published JSON Schema on schemastore.org.
#
# Usage (from repo root):
#   crates/noya-cli/examples/validate-pyproject.sh

set -euo pipefail

CACHE="$(dirname "$0")/.cache"
mkdir -p "$CACHE"

# Schema-store mappings: <filename> → <schema URL>
declare -a TARGETS=(
  "mkdocs.yml=https://json.schemastore.org/mkdocs-1.6.json"
  ".pre-commit-config.yaml=https://json.schemastore.org/pre-commit-config.json"
  ".github/dependabot.yml=https://json.schemastore.org/dependabot-2.0.json"
  ".readthedocs.yml=https://json.schemastore.org/rtd-config.json"
  ".readthedocs.yaml=https://json.schemastore.org/rtd-config.json"
  ".gitlab-ci.yml=https://json.schemastore.org/gitlab-ci.json"
  ".gitlab-ci.yaml=https://json.schemastore.org/gitlab-ci.json"
  ".circleci/config.yml=https://json.schemastore.org/circleciconfig.json"
)

ec=0

for target in "${TARGETS[@]}"; do
  filename="${target%%=*}"
  schema_url="${target#*=}"

  # Resolve any glob (e.g., the file might not exist).
  [ -f "$filename" ] || continue

  schema_local="$CACHE/$(basename "$schema_url")"
  if [ ! -f "$schema_local" ]; then
    echo "Fetching $(basename "$schema_url") → $schema_local"
    curl -sSfL "$schema_url" -o "$schema_local"
  fi

  echo "::group::validate $filename"
  noyavalidate --schema "$schema_local" "$filename" || ec=1
  echo "::endgroup::"
done

if [ "$ec" -ne 0 ]; then
  echo "✗ One or more project YAML files failed validation."
  exit 1
fi

echo "✓ All project YAML files pass their respective schemas."
