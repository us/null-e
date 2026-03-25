# typed: false
# frozen_string_literal: true

class NullE < Formula
  desc "The friendly disk cleanup robot - send your cruft to /dev/null"
  homepage "https://github.com/us/null-e"
  version "0.2.0"
  license "WTFPL"

  on_macos do
    on_arm do
      url "https://github.com/us/null-e/releases/download/v#{version}/null-e-darwin-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_ARM64"

      def install
        bin.install "null-e"
      end
    end

    on_intel do
      url "https://github.com/us/null-e/releases/download/v#{version}/null-e-darwin-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_X86_64"

      def install
        bin.install "null-e"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/us/null-e/releases/download/v#{version}/null-e-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"

      def install
        bin.install "null-e"
      end
    end

    on_intel do
      url "https://github.com/us/null-e/releases/download/v#{version}/null-e-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"

      def install
        bin.install "null-e"
      end
    end
  end

  test do
    system "#{bin}/null-e", "--version"
  end
end
