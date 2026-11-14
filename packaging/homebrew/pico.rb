class Pico < Formula
  desc "Modern, backward-compatible replacement for the Unix echo utility"
  homepage "https://github.com/PhotoDotPi/pico"
  version "1.0.0"
  license "AGPL-3.0-only"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "pico"
  end

  test do
    assert_equal "hello", shell_output("#{bin}/pico hello").strip
  end
end
