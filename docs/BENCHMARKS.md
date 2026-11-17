<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

The `cli_dispatch` Criterion harness measures command dispatch and representative
format and validation workloads in release mode.

```bash
cargo bench --bench cli_dispatch
```

Record the CPU, operating system, Rust version, commit, input size, feature set,
and full command with any reported result. Compare measurements from the same
machine and power profile. CI compiles and smoke-runs benchmark targets, but does
not reject a change using noisy shared-runner wall-clock thresholds.

Publish a result only with its raw samples, source commit and environment.
The presence of this harness does not establish a latency guarantee.
