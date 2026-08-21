#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Verify the README's command examples against the binaries themselves.
#
# WHY THIS EXISTS, AND WHY IT IS NOT THE CORE'S SCRIPT
#
# `noyalib`'s harness compiles every ```rust block in its README. This
# repo's README contains **no Rust blocks** — it documents two command
# line tools, so its examples are shell invocations. Porting the core's
# script here would find nothing, pass trivially, and report a green
# tick for a check that never ran. The scorecard row would look
# identical either way, which is precisely the failure mode this whole
# harness exists to avoid.
#
# What actually rots in a CLI README is flags: one gets renamed, and
# every copy-pasteable example in the docs silently becomes wrong. So
# that is what this checks.
#
#   1. Find every README line invoking `noyafmt` or `noyavalidate`.
#   2. Extract the long flags used on that line.
#   3. Assert each one appears in that binary's own `--help`.
#
# Exit non-zero on the first flag the binary does not define.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

README=${1:-README.md}
[ -f "$README" ] || { echo "no $README"; exit 1; }

fail=0
checked=0

for bin in noyafmt noyavalidate; do
  help=$(cargo run -q --bin "$bin" -- --help 2>/dev/null) || {
    echo "  [FAIL] could not run $bin --help"; exit 1; }
  known=$(printf '%s' "$help" | grep -oE '\-\-[a-z0-9-]+' | sort -u)

  # Lines that invoke this binary, comments stripped so a `# --foo` note
  # is not mistaken for a flag in use.
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    cmd=${line%%#*}
    for flag in $(printf '%s' "$cmd" | grep -oE '\-\-[a-z0-9-]+' | sort -u); do
      checked=$((checked + 1))
      if ! printf '%s\n' "$known" | grep -qx -- "$flag"; then
        echo "  [FAIL] $README documents '$flag' for $bin, which does not define it"
        fail=1
      fi
    done
  done < <(grep -E "^\s*\\\$?\s*$bin\b" "$README")
done

if [ "$fail" -ne 0 ]; then
  echo "── README documents flags the binaries do not define ──"
  exit 1
fi
echo "── All $checked README flag use(s) match the binaries' --help ──"
