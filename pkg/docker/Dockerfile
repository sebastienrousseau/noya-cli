# syntax=docker/dockerfile:1.7
# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0

# pkg/docker/Dockerfile — Debian-slim image carrying `noyafmt` and
# `noyavalidate` with the full miette renderer (colour, terminal-size
# aware diagnostics) and a shell for ad-hoc scripting. Published as
# ghcr.io/sebastienrousseau/noya-cli:<version> by the release workflow,
# with SLSA provenance and a keyless cosign signature.
#
# The default entrypoint is `noyavalidate`; run `noyafmt` explicitly:
#   docker run --rm -v "$PWD:/work" -w /work ghcr.io/sebastienrousseau/noya-cli:latest config.yaml
#   docker run --rm -v "$PWD:/work" -w /work --entrypoint noyafmt ghcr.io/sebastienrousseau/noya-cli:latest --check config.yaml

FROM rust:1.98-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS build
WORKDIR /src
COPY . .
# Reproducible-build hygiene: no absolute paths in the binaries, a fixed
# timestamp when the caller supplies one.
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0}
ENV RUSTFLAGS="--remap-path-prefix=/src=/build --remap-path-prefix=/usr/local/cargo=/cargo"
RUN cargo build --release --locked --bin noyafmt --bin noyavalidate

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
LABEL org.opencontainers.image.title="noya-cli"
LABEL org.opencontainers.image.description="noyafmt and noyavalidate: format and validate YAML with the noyalib parser, full miette diagnostics."
LABEL org.opencontainers.image.source="https://github.com/sebastienrousseau/noya-cli"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system noya \
    && useradd  --system --gid noya --create-home --home-dir /home/noya noya
COPY --from=build /src/target/release/noyafmt      /usr/local/bin/noyafmt
COPY --from=build /src/target/release/noyavalidate /usr/local/bin/noyavalidate
COPY --from=build /src/docs/noyafmt.1              /usr/share/man/man1/noyafmt.1
COPY --from=build /src/docs/noyavalidate.1         /usr/share/man/man1/noyavalidate.1
USER noya:noya
WORKDIR /home/noya
ENTRYPOINT ["/usr/local/bin/noyavalidate"]
