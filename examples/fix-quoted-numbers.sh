#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Demo of the schema-driven autofix flow. Shows how
# `noyavalidate --fix` rewrites a quoted-number scalar
# (`port: "8080"`) into the schema's declared integer type.

set -euo pipefail

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/schema.yaml" <<'EOF'
type: object
required: [host, port]
properties:
  host: { type: string }
  port: { type: integer, minimum: 1, maximum: 65535 }
EOF

cat > "$WORK/config.yaml" <<'EOF'
host: api.example.com
port: "8080"   # comment preserved through the fix
EOF

echo "── before ──"
cat "$WORK/config.yaml"

echo
echo "── strict validate (expected: schema violation) ──"
noyavalidate --schema "$WORK/schema.yaml" "$WORK/config.yaml" \
    && echo "unexpectedly clean" \
    || echo "(non-zero exit as expected)"

echo
echo "── --fix ──"
noyavalidate --schema "$WORK/schema.yaml" --fix "$WORK/config.yaml"

echo
echo "── after ──"
cat "$WORK/config.yaml"

echo
echo "── validate again (expected: ok) ──"
noyavalidate --schema "$WORK/schema.yaml" "$WORK/config.yaml"
