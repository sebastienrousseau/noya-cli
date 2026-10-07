#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# The steps of the composite action in action.yml, kept in one script so
# scripts/test-action.sh exercises exactly what the action runs.
#
#   noya-action.sh install    download, verify and install the binaries
#   noya-action.sh format     noyafmt --check over the YAML under PATHS
#   noya-action.sh validate   noyavalidate (with SCHEMA) over the same set
#
# Environment: VERSION (install), PATHS and SCHEMA (format, validate),
# GH_TOKEN (install, optional: enables the attestation check).
#
# PATHS is split on whitespace with globbing off, so a file named
# `[x].yaml` is that file and not a pattern. Directories are searched
# for *.yaml and *.yml; files named directly are used as given. Names
# travel NUL-separated and reach the tools after `--`.
#
# Everything the tools print is wrapped in ::stop-commands::, so a file
# name or YAML content that looks like a workflow command (`::error::`,
# `::add-mask::`) is printed as text and never acted on.

set -euo pipefail

REPO=sebastienrousseau/noya-cli

die() {
  echo "::error::$*"
  exit 1
}

# ---------------------------------------------------------------- install

check_version() {
  [[ "${VERSION:-}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
    die "version must look like 1.2.3"
}

runner_target() {
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) echo x86_64-unknown-linux-musl ;;
    Linux-aarch64) echo aarch64-unknown-linux-musl ;;
    Darwin-x86_64) echo x86_64-apple-darwin ;;
    Darwin-arm64) echo aarch64-apple-darwin ;;
    *) die "unsupported runner: $(uname -s)-$(uname -m)" ;;
  esac
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# The build provenance attestation proves the archive was built by this
# repository's release workflow from the tag of the requested version.
# The SHA-256 sits in the same release as the archive, so on its own it
# only catches a corrupted download, not a replaced release.
verify_archive() {
  local archive=$1
  local expected
  expected=$(awk '{print $1}' "${archive}.sha256")
  [ "$expected" = "$(sha256_of "$archive")" ] || die "checksum mismatch for ${archive##*/}"
  if command -v gh >/dev/null 2>&1 && [ -n "${GH_TOKEN:-}" ]; then
    gh attestation verify "$archive" --repo "$REPO" \
      --signer-workflow "${REPO}/.github/workflows/release.yml" \
      --source-ref "refs/tags/v${VERSION}" ||
      die "attestation check failed for ${archive##*/}"
  else
    echo "::warning::gh or a token is unavailable: checked only the release's own SHA-256, not the build attestation"
  fi
}

install_tools() {
  check_version
  local target name base work dest
  target=$(runner_target)
  name="noya-cli-${VERSION}-${target}"
  base="https://github.com/${REPO}/releases/download/v${VERSION}"
  work=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/noya-cli.XXXXXX")
  curl -sSfL -o "${work}/${name}.tar.gz" "${base}/${name}.tar.gz"
  curl -sSfL -o "${work}/${name}.tar.gz.sha256" "${base}/${name}.tar.gz.sha256"
  verify_archive "${work}/${name}.tar.gz"
  tar xzf "${work}/${name}.tar.gz" -C "$work"
  dest="$HOME/.noya-cli/bin"
  mkdir -p "$dest"
  install -m 0755 "${work}/${name}/noyafmt" "${work}/${name}/noyavalidate" "$dest/"
  echo "$dest" >>"${GITHUB_PATH:?}"
  rm -rf "$work"
}

# ------------------------------------------------------- format, validate

# Fill the global FILES array with the YAML files under PATHS.
collect_files() {
  local roots root f
  set -f
  # shellcheck disable=SC2206 # deliberate split on whitespace, glob off
  roots=(${PATHS:-})
  set +f
  [ "${#roots[@]}" -gt 0 ] || roots=(.)
  FILES=()
  for root in "${roots[@]}"; do
    case $root in -*) root="./$root" ;; esac
    if [ -d "$root" ]; then
      while IFS= read -r -d '' f; do
        FILES+=("$f")
      done < <(find "$root" -type f \( -name '*.yaml' -o -name '*.yml' \) -print0 | sort -z)
    else
      FILES+=("$root")
    fi
  done
}

# Run "$@" with workflow commands disabled for everything it prints.
quoted() {
  local token status=0
  token=$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')
  echo "::stop-commands::${token}"
  "$@" || status=$?
  echo "::${token}::"
  return "$status"
}

format_files() {
  printf '%s\0' "${FILES[@]}" | xargs -0 noyafmt --check --
}

validate_files() {
  local f status=0
  for f in "${FILES[@]}"; do
    if [ -n "${SCHEMA:-}" ]; then
      noyavalidate --schema "$SCHEMA" -- "$f" || status=1
    else
      noyavalidate -- "$f" || status=1
    fi
  done
  return "$status"
}

main() {
  case "${1:-}" in
    install) install_tools ;;
    format | validate)
      collect_files
      if [ "${#FILES[@]}" -eq 0 ]; then
        echo "::notice::no YAML files under the given paths"
        return 0
      fi
      quoted "${1}_files"
      ;;
    *) die "usage: noya-action.sh install|format|validate" ;;
  esac
}

main "$@"
