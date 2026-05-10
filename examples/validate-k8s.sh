#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# CI gate for Kubernetes manifests. Runs `noyavalidate` against
# every YAML in the manifests/ directory using the JSON Schema
# under schema/k8s.yaml. Exits 1 on any syntax error or schema
# violation; the failing files + line/column are printed to
# stderr in rustc-style format.
#
# Use as a GitHub Actions step:
#
#   - name: Validate Kubernetes manifests
#     run: crates/noya-cli/examples/validate-k8s.sh

set -euo pipefail
IFS=$'\n\t'

MANIFESTS_DIR="${1:-manifests}"
SCHEMA_FILE="${SCHEMA_FILE:-schema/k8s.yaml}"

if [[ ! -d "$MANIFESTS_DIR" ]]; then
    echo "manifests directory not found: $MANIFESTS_DIR" >&2
    exit 2
fi

mapfile -t FILES < <(find "$MANIFESTS_DIR" -type f \( -name '*.yaml' -o -name '*.yml' \))

if [[ ${#FILES[@]} -eq 0 ]]; then
    echo "no YAML manifests found under $MANIFESTS_DIR/"
    exit 0
fi

echo "→ validating ${#FILES[@]} manifest(s) against $SCHEMA_FILE"
fail=0
for f in "${FILES[@]}"; do
    if ! noyavalidate --schema "$SCHEMA_FILE" "$f"; then
        echo "::error file=$f::schema validation failed"
        fail=1
    fi
done

exit $fail
