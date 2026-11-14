class Pico < Formula
  desc "Modern, backward-compatible replacement for the Unix echo utility"
  homepage "https://github.com/PhotoDotPi/pico"
  version "1.0.0"
  license "AGPL-3.0-only"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "385ec770ca8e296f2ffe52da353b9843397a494051cc9c6f340bf83028ff900b"
    else
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "2aca3fd1e0ca83010378a607a9e91461f5907d88d595e746113fe901697d4372"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d784ff82f7345734a0e3fb4cdbdf155bf93c42f38276febc430ebbfe9c264d8f"
    else
      url "https://github.com/PhotoDotPi/pico/releases/download/pico-v1.0.0/pico-1.0.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "bb150ae66749a7f984be40059522492eb663134a44aaf4076c444bbae4c109b9"
    end
  end

  def install
    bin.install "pico"
  end

  test do
    assert_equal "hello", shell_output("#{bin}/pico hello").strip
  end
end
