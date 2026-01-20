# Homebrew formula for null-e
# To use: Copy this file to your homebrew-tap repo at Formula/null-e.rb
#
# Users can then install with:
#   brew tap us/tap
#   brew install null-e

class NullE < Formula
  desc "The friendly disk cleanup robot - Send your cruft to /dev/null!"
  homepage "https://github.com/us/null-e"
  url "https://github.com/us/null-e/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"
  head "https://github.com/us/null-e.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "null-e", shell_output("#{bin}/null-e --version")
  end
end

# To get the SHA256 hash after creating a release:
#   curl -sL https://github.com/us/null-e/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
