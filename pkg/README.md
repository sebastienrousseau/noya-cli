<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Distribution channel sources

Per-channel packaging for the noya-cli binaries, generated against a
released version's signed artefacts (hashes from the release's
`.sha256` assets). At each release, update version + hashes here and
in the downstream channel:

| Channel | Source here | Downstream |
| :--- | :--- | :--- |
| Homebrew | `brew/noya-cli.rb` | [sebastienrousseau/homebrew-tap](https://github.com/sebastienrousseau/homebrew-tap) `Formula/` |
| Scoop | `scoop/noya-cli.json` | [sebastienrousseau/scoop-bucket](https://github.com/sebastienrousseau/scoop-bucket) `bucket/` (autoupdate reads new releases itself) |
| AUR | `aur/PKGBUILD` | `noyalib-bin` on aur.archlinux.org — push with the maintainer's AUR SSH key: `git clone ssh://aur@aur.archlinux.org/noyalib-bin.git`, copy PKGBUILD, `makepkg --printsrcinfo > .SRCINFO`, commit, push |
| deb / rpm | built by release.yml | attached to every GitHub Release |
| Debian / Fedora proper | `docs/packaging.md` in the core repo | rust-team / SIG submission, from source |

## Container image

`pkg/docker/Dockerfile` builds `ghcr.io/sebastienrousseau/noya-cli`, Debian-slim with both binaries, their manpages, and the full miette renderer. The release workflow publishes it for linux/amd64 and linux/arm64 with SLSA provenance and a keyless cosign signature; a `workflow_dispatch` dry run builds it without pushing.
