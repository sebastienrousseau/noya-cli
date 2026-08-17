# Security Policy

## Supported Versions

| Version | Supported |
|:--------|:---------:|
| 0.0.x   | Yes       |

`noya-cli` follows the [ADR-0005 strict-lockstep versioning
contract](https://github.com/sebastienrousseau/noyalib/blob/main/doc/adr/0005-workspace-split.md).
Every release is coordinated with `noyalib` at the same
version.

## Reporting a Vulnerability

Email **sebastian.rousseau@gmail.com**. Do not open a public
issue for security reports.

Include: description, steps to reproduce, affected versions,
suggested fix (optional). Initial response within 48h; fix or
mitigation plan within 7 days.

## Threat Model — CLI-Specific

`noya-cli` ships two binaries: `noyafmt` (formatter) and
`noyavalidate` (schema validator + autofixer). Threat model:

- **Untrusted YAML files on disk**: every file the CLI reads
  goes through the same parser hardening as the library.
  DoS budgets (`max_depth`, `max_document_length`,
  `max_alias_expansions`) apply.
- **`--fix` mode**: mutates files in place. Callers must
  understand the operation is destructive to the source file;
  version-control it before `--fix`.
- **`--schema URL` mode**: fetches a JSON Schema over HTTPS.
  Only fetches from URLs the caller supplied — no automatic
  resolution.
- **Config file discovery**: reads `.noyafmt.toml` from CWD +
  upwards. Malicious config files in a repo can influence
  format behaviour; never run `noyafmt` on an untrusted repo
  in a security-sensitive context.

## Security Design

Inherits every security invariant from parent `noyalib`:
`#[forbid(unsafe_code)]`, no C deps, parser DoS guards.

## Supply Chain

- `cargo-deny` in CI (advisories + bans + licenses + sources).
- All GitHub Actions SHA-pinned.
- CI composed from `sebastienrousseau/noyalib`'s shared
  reusable workflows.
- `Cargo.lock` committed for deterministic builds.

## Build Provenance & Artefact Signing

Each release ships with:

1. SLSA Level 3 build provenance via
   `actions/attest-build-provenance`.
2. Keyless sigstore signatures (Fulcio + Rekor) on every
   published `.crate` and pre-built binary.
3. Multi-arch binaries (`noyafmt-<version>-<triple>.tar.gz` +
   `.zip`) attached to each GitHub Release with cosign bundles.
4. SBOM attached to each GitHub Release.

### Detached GPG signatures

Additive to the sigstore signing above, not a replacement. Keyless
signing is the stronger primitive — nothing long-lived to steal, and
every signature publicly logged in Rekor — but verifying it needs
`cosign` and network access. A detached `.asc` can be checked with the
`gpg` any distribution already ships, offline, which is what package
maintainers and air-gapped consumers ask for.

Every `.crate` and the SBOM in a release carry a matching `.asc`:

```sh
# Import the release-signing key (also in KEYS.asc at the repo root)
gpg --recv-keys 4B7F16C909C7A8EE9BED338A4F047EDF5F90F638

gpg --verify <artefact>.asc <artefact>
```

**Release-signing key fingerprint:**

```text
4B7F16C909C7A8EE9BED338A4F047EDF5F90F638
```

Signing key `Sebastien Rousseau <sebastian.rousseau@gmail.com>`,
ed25519, signing-only, expires 2028-08-16. Verify the fingerprint out of
band before trusting it — a key fetched over the same channel as the
artefact proves nothing on its own. The sigstore bundle needs no such
step, which is why it remains the recommended check.

## Commit Integrity

Every commit on `main` must be signed. CI rejects unsigned PR
commits via `shared-verify-signatures.yml`.
