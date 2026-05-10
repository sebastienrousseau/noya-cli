#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# CI pipeline example — `noyafmt --check` + `noyavalidate --schema`
# as a single gate before merging YAML changes.
#
# Drop this into `.github/workflows/yaml-gate.yml` (or any other
# CI runner) as a shell step. Both binaries are installed via
# `cargo install noya-cli`; the script assumes they're on $PATH.

set -euo pipefail

# 1. Format check — fails fast with a diff if any tracked YAML
#    is mis-formatted relative to the canonical noyafmt output.
echo "::group::noyafmt --check (formatting gate)"
git ls-files '*.yaml' '*.yml' | xargs -r noyafmt --check
echo "::endgroup::"

# 2. Schema validation — point each manifest at its schema and
#    fail the build on any structural error. Adjust the
#    --schema flag to your project's schema location.
echo "::group::noyavalidate --schema (structural gate)"
for manifest in $(git ls-files 'config/*.yaml' 'k8s/*.yaml'); do
  schema="schemas/$(basename "$manifest" .yaml).schema.json"
  if [ -f "$schema" ]; then
    echo "validating $manifest against $schema"
    noyavalidate --schema "$schema" "$manifest"
  else
    echo "no schema for $manifest — skipping (consider adding one)"
  fi
done
echo "::endgroup::"

# 3. Optional: run noyavalidate --fix on PRs and commit the
#    auto-fixes back. Skipped here — most teams want the
#    fix to be a manual `noyavalidate --schema --fix` invocation
#    rather than an opaque CI commit.
