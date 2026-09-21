<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Engineering policies

## Version and dependency policy

`noya-cli` releases in strict lockstep with `noyalib`. Version 0.0.47 must pin
the core at exactly `=0.0.47`. Release work uses `feat/v0.0.47`; each subsequent
iteration increments exactly 0.0.1.

## Minimum Rust version

The minimum supported Rust version is 1.86.0. The manifest declares the floor
and CI verifies it. A change to the floor is a breaking-axis change and requires
a changelog entry.

## Generated assets

Manpages and completions are generated from the Clap definitions. Change the
command source and run `make assets`; never edit generated copies directly.

## Compatibility

Exit codes, command names, flags, configuration semantics, and formatting output
are public behaviour. During `0.0.x`, the patch component is the breaking axis.

Family-wide policies live in the core
[`POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/docs/POLICIES.md).
