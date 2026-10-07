# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.54/noya-cli-0.0.54-aarch64-apple-darwin.tar.gz"
      sha256 "ba2edfc9fad9f991f55a7379106546342a5396b5b5c303264dd64619c97c7433"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.54/noya-cli-0.0.54-x86_64-apple-darwin.tar.gz"
      sha256 "0e9a8b71e0b6fba438c9a35ef5f89f0a517f443e8544a28bc74f2096eb2f8367"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.54/noya-cli-0.0.54-aarch64-unknown-linux-musl.tar.gz"
      sha256 "cd7a048d6f5cd26d2596d771633c8a0549325a6b52fe246135e06500ef5b4742"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.54/noya-cli-0.0.54-x86_64-unknown-linux-musl.tar.gz"
      sha256 "24b047c7a1f0ec3cd5976519fa395f31c1b370a73137f110b97aee9840edee1f"
    end
  end

  def install
    bin.install "noyafmt", "noyavalidate"
    man1.install "noyafmt.1", "noyavalidate.1"
    bash_completion.install "complete/noyafmt.bash" => "noyafmt"
    bash_completion.install "complete/noyavalidate.bash" => "noyavalidate"
    zsh_completion.install "complete/_noyafmt", "complete/_noyavalidate"
    fish_completion.install "complete/noyafmt.fish", "complete/noyavalidate.fish"
  end

  test do
    assert_match "noyafmt", shell_output("#{bin}/noyafmt --version")
    (testpath/"t.yaml").write("a: 1\n")
    assert_match "a: 1", shell_output("#{bin}/noyafmt #{testpath}/t.yaml")
  end
end
