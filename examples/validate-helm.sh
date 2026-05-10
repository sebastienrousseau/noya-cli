#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# validate-helm.sh — gate `values.yaml` files in a Helm chart
# tree against the chart's `values.schema.json`.
#
# Helm itself runs schema validation at template / install
# time, but a pre-merge gate using noyavalidate catches schema
# drift earlier (in PR review) and produces richer error output
# than `helm lint` (rich miette diagnostics with caret + span).
#
# Usage (from repo root, with charts/ as the chart parent):
#   crates/noya-cli/examples/validate-helm.sh charts/

set -euo pipefail

CHARTS_DIR="${1:-charts}"

if [ ! -d "$CHARTS_DIR" ]; then
  echo "✗ chart directory not found: $CHARTS_DIR"
  exit 1
fi

ec=0
shopt -s nullglob

# 1. Walk every chart directory.
for chart in "$CHARTS_DIR"/*/; do
  chart="${chart%/}"
  schema="$chart/values.schema.json"
  values_main="$chart/values.yaml"

  if [ ! -f "$schema" ]; then
    echo "○ skip $chart  (no values.schema.json — consider adding one)"
    continue
  fi

  # 2. Validate the chart's own values.yaml.
  if [ -f "$values_main" ]; then
    echo "::group::validate $values_main"
    noyavalidate --schema "$schema" "$values_main" || ec=1
    echo "::endgroup::"
  fi

  # 3. Validate every values-*.yaml override (env-specific).
  for override in "$chart"/values-*.yaml; do
    [ -f "$override" ] || continue
    echo "::group::validate $override"
    noyavalidate --schema "$schema" "$override" || ec=1
    echo "::endgroup::"
  done
done

if [ "$ec" -ne 0 ]; then
  echo "✗ One or more values files failed schema validation."
  exit 1
fi

echo "✓ All values files pass their chart's schema."
