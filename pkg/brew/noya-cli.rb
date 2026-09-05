# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.33/noya-cli-0.0.33-aarch64-apple-darwin.tar.gz"
      sha256 "6824741da33f9dc8eb67cdc32dbebd9dc36d8ffaae557e3c123bb2083026d60d"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.33/noya-cli-0.0.33-x86_64-apple-darwin.tar.gz"
      sha256 "80c36b1b92a867bac630aa0f8d58765821fea24f0ee1ff037f26cc23811c2a26"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.33/noya-cli-0.0.33-aarch64-unknown-linux-musl.tar.gz"
      sha256 "e66f40ec8114b1599fa228ec930239c8780d0a2f91df56b16f94838a9f10b3ef"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.33/noya-cli-0.0.33-x86_64-unknown-linux-musl.tar.gz"
      sha256 "dc2b915614e6ad1cbec46fc35eaa4ad828f1ee8987014f62743f3d3c5dbae614"
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
