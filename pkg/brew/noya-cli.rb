# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.60/noya-cli-0.0.60-aarch64-apple-darwin.tar.gz"
      sha256 "9f3729fdf112a05569f512b980a73a1e2ad5fb4e3e7ede7c992213a31614eb89"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.60/noya-cli-0.0.60-x86_64-apple-darwin.tar.gz"
      sha256 "c04801ce82e044bb9bc04a3fb55bbb250d18634ae53ea44ae3190e35d98b5388"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.60/noya-cli-0.0.60-aarch64-unknown-linux-musl.tar.gz"
      sha256 "e271d9ade16db1712c6c0e994b20431da032fc20e81857e97ce484203d381a68"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.60/noya-cli-0.0.60-x86_64-unknown-linux-musl.tar.gz"
      sha256 "67c38e35f12d5e2512b1dab886c8db8da65e52963f80c5f6fc19000398d9e96f"
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
