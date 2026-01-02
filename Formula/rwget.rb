# typed: false
# frozen_string_literal: true

class Rwget < Formula
  desc "wget-compatible wrapper with automatic fallback"
  homepage "https://github.com/dipankardas011/rwget"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/dipankardas011/rwget/releases/download/v1.0.0/rwget-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/dipankardas011/rwget/releases/download/v1.0.0/rwget-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/dipankardas011/rwget/releases/download/v1.0.0/rwget-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/dipankardas011/rwget/releases/download/v1.0.0/rwget-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "wget"

  def install
    bin.install "rwget"
    bin.install "rwgetd"

    # Install shell completions
    generate_completions_from_executable(bin/"rwget", "--rwget-completions")

    # Install man page
    man1.install "rwget.1" if File.exist?("rwget.1")
  end

  def caveats
    <<~EOS
      rwget is a drop-in replacement for wget that automatically retries
      with browser emulation when sites block standard wget requests.

      For Stage 3 (JavaScript preflight), you may want to pre-download Chromium:
        rwget --rwget-download-chromium

      For more information:
        rwget --rwget-help
    EOS
  end

  test do
    assert_match "rwget #{version}", shell_output("#{bin}/rwget --rwget-version")

    # Test that wget passthrough works
    system bin/"rwget", "--rwget-no-fallback", "--help"
  end
end
