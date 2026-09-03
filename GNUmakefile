# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# GNU Make install contract for noya-cli (noyafmt, noyavalidate).
#
#   make                 build release binaries
#   make test            run the test suite
#   make assets          regenerate manpages (docs/*.1) and shell
#                        completions (complete/*) from the clap
#                        definitions via build.rs (NOYA_GEN_ASSETS=1)
#   make check-assets    fail if the tracked copies differ from what
#                        the clap definitions generate (CI gate)
#   make install         install binaries, manpages, completions
#   make uninstall       remove everything install placed
#
# PREFIX (default /usr/local) and DESTDIR are honored the usual way:
#
#   make install PREFIX=/usr
#   make install DESTDIR=/tmp/stage
#
# Dev tasks (lint, fuzz, sbom, …) live in the core repository's
# Makefile; this file is only the Unix build-and-install surface.

SHELL := /bin/sh
CARGO ?= cargo
PREFIX ?= /usr/local
DESTDIR ?=

bindir      = $(PREFIX)/bin
mandir      = $(PREFIX)/share/man/man1
bashcompdir = $(PREFIX)/share/bash-completion/completions
zshcompdir  = $(PREFIX)/share/zsh/site-functions
fishcompdir = $(PREFIX)/share/fish/vendor_completions.d

BINS = noyafmt noyavalidate
RELEASE_DIR = target/release

.PHONY: all build test assets check-assets install uninstall clean

all: build

build:
	$(CARGO) build --release

test:
	$(CARGO) test

# build.rs generates into Cargo's OUT_DIR when NOYA_GEN_ASSETS=1;
# copy the freshest generated set into the tracked locations. The
# tracked copies exist because distro packagers prefer pre-built
# artefacts; check-assets keeps them bit-identical to the source of
# truth (the clap Command definitions).
assets:
	NOYA_GEN_ASSETS=1 $(CARGO) build --release
	@out=$$(ls -td target/release/build/noya-cli-*/out 2>/dev/null | head -1); \
	test -n "$$out" || { echo "generated assets not found under target/release/build" >&2; exit 1; }; \
	for b in $(BINS); do \
	  cp "$$out/$$b.1" docs/; \
	  cp "$$out/$$b.bash" "$$out/$$b.fish" "$$out/_$$b" "$$out/_$$b.ps1" complete/; \
	done; \
	echo "assets refreshed from $$out"

check-assets: assets
	@git diff --exit-code -- docs/*.1 complete/ \
	  || { echo "tracked manpages/completions drifted from the clap definitions — run 'make assets' and commit" >&2; exit 1; }

install: build
	install -d "$(DESTDIR)$(bindir)" "$(DESTDIR)$(mandir)" \
	  "$(DESTDIR)$(bashcompdir)" "$(DESTDIR)$(zshcompdir)" "$(DESTDIR)$(fishcompdir)"
	for b in $(BINS); do \
	  install -m 0755 "$(RELEASE_DIR)/$$b" "$(DESTDIR)$(bindir)/$$b"; \
	  install -m 0644 "docs/$$b.1" "$(DESTDIR)$(mandir)/$$b.1"; \
	  install -m 0644 "complete/$$b.bash" "$(DESTDIR)$(bashcompdir)/$$b"; \
	  install -m 0644 "complete/_$$b" "$(DESTDIR)$(zshcompdir)/_$$b"; \
	  install -m 0644 "complete/$$b.fish" "$(DESTDIR)$(fishcompdir)/$$b.fish"; \
	done

uninstall:
	for b in $(BINS); do \
	  rm -f "$(DESTDIR)$(bindir)/$$b" \
	        "$(DESTDIR)$(mandir)/$$b.1" \
	        "$(DESTDIR)$(bashcompdir)/$$b" \
	        "$(DESTDIR)$(zshcompdir)/_$$b" \
	        "$(DESTDIR)$(fishcompdir)/$$b.fish"; \
	done

clean:
	$(CARGO) clean
