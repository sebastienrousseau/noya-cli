#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Self-test for the composite action (action.yml), run against
# scripts/noya-action.sh, the script the action's steps call.
#
#   cargo build && scripts/test-action.sh
#
# The binaries come from NOYA_BIN_DIR (default: target/debug, or
# $CARGO_TARGET_DIR/debug). No network: the install step is only
# checked for refusing a malformed version before it downloads.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

bin_dir=${NOYA_BIN_DIR:-${CARGO_TARGET_DIR:-target}/debug}
if [ ! -x "$bin_dir/noyafmt" ] || [ ! -x "$bin_dir/noyavalidate" ]; then
  echo "build first: no noyafmt/noyavalidate in $bin_dir" >&2
  exit 2
fi
PATH="$(cd "$bin_dir" && pwd):$PATH"
export PATH

# NOYA_ACTION_SCRIPT points the test at another implementation, for
# example the previous inline steps, to show what the test catches.
action=${NOYA_ACTION_SCRIPT:-$PWD/scripts/noya-action.sh}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
failures=0

pass() { echo "ok   - $1"; }
fail() { echo "FAIL - $1"; failures=$((failures + 1)); }

# expect <want: 0|nonzero> <description> <command...>
expect() {
  local want=$1 what=$2 status=0
  shift 2
  "$@" >"$work/out" 2>&1 || status=$?
  if { [ "$want" = 0 ] && [ "$status" -eq 0 ]; } ||
    { [ "$want" = nonzero ] && [ "$status" -ne 0 ]; }; then
    pass "$what"
  else
    fail "$what (exit $status)"
    sed 's/^/     | /' "$work/out"
  fi
}

# A line the runner would act on: a workflow command outside a
# ::stop-commands:: block.
live_commands() {
  awk '
    stop != "" { if ($0 == "::" stop "::") stop = ""; next }
    /^::stop-commands::/ { stop = substr($0, 18); next }
    /^::[a-z-]+( [^:]*)?::/ && !/^::notice::no YAML files/ { print; found = 1 }
    END { exit found ? 0 : 1 }
  ' "$1"
}

run() { env "$@" "$action" "$MODE"; }

# 1. A file named like a glob is that file, not a pattern: an invalid
#    [x].yaml next to a valid x.yaml must fail validation.
mkdir glob
printf 'a: 1\n' >glob/x.yaml
printf 'a: [\n' >'glob/[x].yaml'
MODE=validate expect nonzero "invalid [x].yaml under a directory is validated" run PATHS=glob
MODE=validate expect nonzero "invalid [x].yaml named directly is validated" run 'PATHS=glob/[x].yaml'
MODE=validate expect 0 "valid x.yaml alone passes" run PATHS=glob/x.yaml

# 2. Directories work for the format step, recursively.
mkdir -p tree/nested
printf 'a: 1\n' >tree/a.yaml
MODE=format expect 0 "formatted directory passes noyafmt --check" run PATHS=tree
printf 'b:    2\n' >tree/nested/b.yml
MODE=format expect nonzero "unformatted nested file fails noyafmt --check" run PATHS=tree
MODE=format expect nonzero "unformatted file is found with several paths" run 'PATHS=glob tree'

# 3. A file name that is a workflow command is printed, never acted on,
#    whether the file is found in a directory or named directly (then
#    noyafmt --check prints the bare name at the start of a line).
mkdir spoof
printf 'a:    1\n' >'spoof/::warning::spoofed.yaml'
printf 'a: [\n' >'spoof/::error::x.yaml'
cp spoof/* .
for paths in spoof '::warning::spoofed.yaml ::error::x.yaml'; do
  for MODE in format validate; do
    status=0
    PATHS=$paths "$action" "$MODE" >"$work/log" 2>&1 || status=$?
    if [ "$status" -eq 0 ]; then
      fail "$MODE $paths: should fail the step"
    elif live_commands "$work/log" >"$work/live"; then
      fail "$MODE $paths: file name became a workflow command: $(head -1 "$work/live")"
    else
      pass "$MODE $paths: file names printed inside ::stop-commands::"
    fi
  done
done

# 4. A path that looks like an option is a path.
printf 'a: 1\n' >--help.yaml
MODE=format expect 0 "a file named --help.yaml is checked, not parsed as a flag" run PATHS=--help.yaml
MODE=validate expect 0 "noyavalidate takes --help.yaml as a file" run PATHS=--help.yaml

# 5. The version input is validated before anything is downloaded. A
#    stand-in curl records any download attempt.
mkdir fakebin
printf '#!/bin/sh\ntouch "%s/curl-called"\nexit 1\n' "$work" >fakebin/curl
chmod +x fakebin/curl
for v in '0.0.55; touch pwned' '../../x' $'0.0.55\nx' 'latest' ''; do
  rm -f curl-called
  MODE=install expect nonzero "version $(printf %q "$v") is refused" \
    run "VERSION=$v" "GITHUB_PATH=$work/gh-path" "PATH=$work/fakebin:$PATH"
  if [ -e curl-called ]; then
    fail "version $(printf %q "$v") reached the download"
  fi
done
if [ -e pwned ]; then
  fail "version input ran a command"
else
  pass "no command ran from the version input"
fi

# 6. Nothing to check is not an error.
mkdir empty
MODE=validate expect 0 "a directory without YAML passes with a notice" run PATHS=empty

if [ "$failures" -ne 0 ]; then
  echo "$failures action self-test(s) failed"
  exit 1
fi
echo "all action self-tests passed"
