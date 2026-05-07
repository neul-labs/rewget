# typed: false
# frozen_string_literal: true

class Rewget < Formula
  desc "wget-compatible wrapper with automatic fallback"
  homepage "https://github.com/neul-labs/rewget"
  version "1.0.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/neul-labs/rewget/releases/download/v1.0.0/rewget-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/neul-labs/rewget/releases/download/v1.0.0/rewget-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/neul-labs/rewget/releases/download/v1.0.0/rewget-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/neul-labs/rewget/releases/download/v1.0.0/rewget-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "wget"

  def install
    bin.install "rewget"
    bin.install "rewgetd"

    # Install shell completions
    generate_completions_from_executable(bin/"rewget", "--rewget-completions")

    # Install man page
    man1.install "rewget.1" if File.exist?("rewget.1")
  end

  def caveats
    <<~EOS
      rewget is a drop-in replacement for wget that automatically retries
      with browser emulation when sites block standard wget requests.

      For Stage 3 (JavaScript preflight), you may want to pre-download Chromium:
        rewget --rewget-download-chromium

      For more information:
        rewget --rewget-help
    EOS
  end

  test do
    assert_match "rewget #{version}", shell_output("#{bin}/rewget --rewget-version")

    # Test that wget passthrough works
    system bin/"rewget", "--rewget-no-fallback", "--help"
  end
end
