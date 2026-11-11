<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# rustix 1.1.6 delta review, 2026-11-11

The fresh RustSec snapshot identified rustix 1.1.4 in both CLI locks under
[RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486.html).
Version 1.1.6 fixes zero-length xattr buffers being marked initialized to
the size reported by a query that wrote no bytes. Both locks now select
the patched registry archive without changing the dependency requirement.

The source review covers the 1.1.4 to 1.1.6 delta: all six xattr readers
initialize zero bytes for a zero-length destination and separate query
functions return sizes. Other changes use MaybeUninit for OS padding,
bound ancillary-message payloads, correct syscall and protocol constants,
normalize timeout carry and update platform cfgs. The build-script change
removes its own compiler-probe artifact under OUT_DIR. No new dependency
or ambient capability is introduced in the CLI's selected graph.

The dependency floor increases from Rust 1.63 to 1.65. The project's
Rust 1.89 floor, public API and features remain unchanged. The review does
not claim runtime execution on every platform rustix supports. The CLI's
filesystem and process suites and the release target matrix remain gates.

Registry archive SHA-256: `e5e76742e5294153d4dde1207dbec5bdd81eef6fabdc0d2118babadabfd858aa`.

Upstream source: `4a563ad698b2c20ff848515821023d2265a788d0`.

Rust source delta SHA-256: `3ba590773b9aa872b551f8d1e6218bba60fe6428624a49850fb660792d800716`.

Manifest/build-script delta SHA-256: `d46a5fdac2072473a616c2f2f476eb80a64d4f2e440ec8fe8fdf33b5d0dec43d`.
