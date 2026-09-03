# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.32/noya-cli-0.0.32-aarch64-apple-darwin.tar.gz"
      sha256 "3663c7d81ed8d7bea8c39858a9c40e588dc05c376c7294a0f0a322fd98ee8862"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.32/noya-cli-0.0.32-x86_64-apple-darwin.tar.gz"
      sha256 "7fe5c6b052772f45bdab7432348cc6c334da014a5f33fb908118e65457568cec"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.32/noya-cli-0.0.32-aarch64-unknown-linux-musl.tar.gz"
      sha256 "cbb545c926e77cc575224c6ca7a5cdcffac6cd251da1305f4514bdf25741f551"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.32/noya-cli-0.0.32-x86_64-unknown-linux-musl.tar.gz"
      sha256 "6b976df707f31508c8d25f586c95eaa5d43652f07e5b6889c024d6bb1168c836"
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
