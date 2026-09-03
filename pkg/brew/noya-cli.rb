# Formula for the sebastienrousseau/tap Homebrew tap. Pre-built
# per-arch binaries from the signed, SLSA-attested GitHub Release.
class NoyaCli < Formula
  desc "YAML formatter and JSON-Schema validator built on noyalib"
  homepage "https://github.com/sebastienrousseau/noya-cli"
  version "0.0.31"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.31/noya-cli-0.0.31-aarch64-apple-darwin.tar.gz"
      sha256 "41fcc43f2b32d0651f6916cb13e7f896fcda63ae630f6358ad66e06f1cb705a9"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.31/noya-cli-0.0.31-x86_64-apple-darwin.tar.gz"
      sha256 "5c8ae0760d951d9cb4b350d6867abb22f7a303c61c296499c2440f620e5fac3e"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.31/noya-cli-0.0.31-aarch64-unknown-linux-musl.tar.gz"
      sha256 "3330c272eb5afc1eaef83952b4629cae1ebb11dd6ecd9b116fa5a3c9db098efe"
    else
      url "https://github.com/sebastienrousseau/noya-cli/releases/download/v0.0.31/noya-cli-0.0.31-x86_64-unknown-linux-musl.tar.gz"
      sha256 "fcb74e4bde36ac7b897414aece695ebdb713fc8c5904fbb5f09c9796d00e29de"
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
