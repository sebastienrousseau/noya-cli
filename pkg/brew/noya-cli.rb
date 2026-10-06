# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.53/noya-cli-0.0.53-aarch64-apple-darwin.tar.gz"
      sha256 "bf032c31221a154b0c577273b45082e760d7f6ceea34229881293137ead6598f"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.53/noya-cli-0.0.53-x86_64-apple-darwin.tar.gz"
      sha256 "44fdf1636a751717c64ec3ccf2a1c8ac2e1e92eec1a1e5ae261a2dabb8d34671"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.53/noya-cli-0.0.53-aarch64-unknown-linux-musl.tar.gz"
      sha256 "02d6cf3a73e7bcd7c8b9e1cdae1c59065ab2b8ab49d72bdd4ecaa37c38a81823"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.53/noya-cli-0.0.53-x86_64-unknown-linux-musl.tar.gz"
      sha256 "2c12fc1b799c555fa2c15960855b11ae4b4ce0a476c03cf90e21ca6d75f9bfcf"
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
