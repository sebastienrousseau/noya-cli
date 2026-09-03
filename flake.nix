# SPDX-FileCopyrightText: 2026 Noyalib
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Nix flake for the noya-cli binaries (noyafmt + noyavalidate),
# building from source with the committed lockfile. The nixpkgs
# submission proper builds from the crates.io release; this flake
# serves `nix run`/`nix profile install` directly from the repo:
#
#   nix run github:sebastienrousseau/noya-cli -- --help
{
  description = "noyafmt and noyavalidate: YAML formatter and JSON-Schema validator built on noyalib";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      forAllSystems = f: nixpkgs.lib.genAttrs
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ]
        (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: rec {
        noya-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "noya-cli";
          version = "0.0.32";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          # The pre-release [patch.crates-io] entry resolves the core
          # from git between lockstep releases.
          cargoLock.allowBuiltinFetchGit = true;
          postInstall = ''
            installManPage docs/noyafmt.1 docs/noyavalidate.1
            installShellCompletion --bash --name noyafmt complete/noyafmt.bash
            installShellCompletion --bash --name noyavalidate complete/noyavalidate.bash
            installShellCompletion --zsh complete/_noyafmt complete/_noyavalidate
            installShellCompletion --fish complete/noyafmt.fish complete/noyavalidate.fish
          '';
          nativeBuildInputs = [ pkgs.installShellFiles ];
          meta = {
            description = "YAML formatter and JSON-Schema validator built on the noyalib library";
            homepage = "https://github.com/sebastienrousseau/noya-cli";
            license = with pkgs.lib.licenses; [ mit asl20 ];
            mainProgram = "noyafmt";
          };
        };
        default = noya-cli;
      });
    };
}
