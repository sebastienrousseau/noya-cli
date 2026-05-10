#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# validate-github-actions.sh — fail the build on any malformed
# GitHub Actions workflow file before the runner picks it up.
#
# GitHub's workflow loader is forgiving (it skips invalid jobs
# silently in some cases); a local pre-merge gate using
# noyavalidate catches structural errors that would otherwise
# silently disable a job.
#
# Usage (from repo root):
#   crates/noya-cli/examples/validate-github-actions.sh
#
# Wire into a pre-commit hook or the CI YAML-gate workflow to
# fail PRs that touch broken workflow files.

set -euo pipefail

WORKFLOW_DIR=".github/workflows"
SCHEMA_URL="https://json.schemastore.org/github-workflow.json"
SCHEMA_LOCAL="$(dirname "$0")/.cache/github-workflow.schema.json"

# 1. Cache the schema locally so the gate runs offline / fast on
#    successive invocations.
mkdir -p "$(dirname "$SCHEMA_LOCAL")"
if [ ! -f "$SCHEMA_LOCAL" ]; then
  echo "Fetching GitHub Actions schema → $SCHEMA_LOCAL"
  curl -sSfL "$SCHEMA_URL" -o "$SCHEMA_LOCAL"
fi

# 2. Validate every workflow file. noyavalidate prints rich
#    miette diagnostics on failure (line + column + caret).
ec=0
for wf in "$WORKFLOW_DIR"/*.yml "$WORKFLOW_DIR"/*.yaml; do
  [ -f "$wf" ] || continue
  echo "::group::validate $wf"
  noyavalidate --schema "$SCHEMA_LOCAL" "$wf" || ec=1
  echo "::endgroup::"
done

if [ "$ec" -ne 0 ]; then
  echo "✗ One or more workflow files failed validation."
  echo "  Run noyavalidate --schema $SCHEMA_LOCAL <file> for full diagnostics."
  exit 1
fi

echo "✓ All workflow files in $WORKFLOW_DIR pass schema validation."
