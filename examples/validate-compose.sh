#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# validate-compose.sh — gate `docker-compose.yml` /
# `compose.yaml` files against the upstream Compose spec.
#
# Docker's compose loader is permissive (it accepts deprecated
# v2 / v3 syntax that newer engines reject silently). A
# pre-merge gate using noyavalidate against the Compose
# JSON Schema catches drift before it hits `docker compose up`.
#
# Usage (from repo root):
#   crates/noya-cli/examples/validate-compose.sh
#   crates/noya-cli/examples/validate-compose.sh path/to/dir
#   crates/noya-cli/examples/validate-compose.sh path/to/file.yaml

set -euo pipefail

TARGET="${1:-.}"
SCHEMA_URL="https://raw.githubusercontent.com/compose-spec/compose-spec/master/schema/compose-spec.json"
SCHEMA_LOCAL="$(dirname "$0")/.cache/compose-spec.schema.json"

mkdir -p "$(dirname "$SCHEMA_LOCAL")"
if [ ! -f "$SCHEMA_LOCAL" ]; then
  echo "Fetching Compose spec schema → $SCHEMA_LOCAL"
  curl -sSfL "$SCHEMA_URL" -o "$SCHEMA_LOCAL"
fi

# Build the file list.
FILES=()
if [ -f "$TARGET" ]; then
  FILES+=("$TARGET")
elif [ -d "$TARGET" ]; then
  shopt -s nullglob
  for f in "$TARGET"/docker-compose.yml \
           "$TARGET"/docker-compose.yaml \
           "$TARGET"/compose.yml \
           "$TARGET"/compose.yaml \
           "$TARGET"/docker-compose.*.yml \
           "$TARGET"/docker-compose.*.yaml \
           "$TARGET"/compose.*.yml \
           "$TARGET"/compose.*.yaml; do
    FILES+=("$f")
  done
fi

if [ "${#FILES[@]}" -eq 0 ]; then
  echo "○ no compose files under $TARGET — nothing to validate"
  exit 0
fi

ec=0
for f in "${FILES[@]}"; do
  echo "::group::validate $f"
  noyavalidate --schema "$SCHEMA_LOCAL" "$f" || ec=1
  echo "::endgroup::"
done

if [ "$ec" -ne 0 ]; then
  echo "✗ One or more Compose files failed validation."
  exit 1
fi

echo "✓ All Compose files in $TARGET pass schema validation."
