#!/bin/bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "Usage: $0 <version> <revision> <artifacts_dir> <output_path>" >&2
  exit 1
fi

VERSION="$1"
REVISION="$2"
ARTIFACTS_DIR="$3"
OUTPUT_PATH="$4"

sha_from_file() {
  awk '{print $1}' "$1"
}

DARWIN_ARM64_SHA="$(sha_from_file "${ARTIFACTS_DIR}/tuhucar-darwin-arm64.sha256")"
DARWIN_X64_SHA="$(sha_from_file "${ARTIFACTS_DIR}/tuhucar-darwin-x64.sha256")"
LINUX_ARM64_SHA="$(sha_from_file "${ARTIFACTS_DIR}/tuhucar-linux-arm64.sha256")"
LINUX_X64_SHA="$(sha_from_file "${ARTIFACTS_DIR}/tuhucar-linux-x64.sha256")"

mkdir -p "$(dirname "$OUTPUT_PATH")"

cat >"$OUTPUT_PATH" <<EOF
class Tuhucar < Formula
  desc "CLI for Tuhu car-care knowledge workflows"
  homepage "https://github.com/tuhucar/cli"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tuhucar/cli/releases/download/v${VERSION}/tuhucar-darwin-arm64"
      sha256 "${DARWIN_ARM64_SHA}"
    else
      url "https://github.com/tuhucar/cli/releases/download/v${VERSION}/tuhucar-darwin-x64"
      sha256 "${DARWIN_X64_SHA}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tuhucar/cli/releases/download/v${VERSION}/tuhucar-linux-arm64"
      sha256 "${LINUX_ARM64_SHA}"
    else
      url "https://github.com/tuhucar/cli/releases/download/v${VERSION}/tuhucar-linux-x64"
      sha256 "${LINUX_X64_SHA}"
    end
  end

  head "https://github.com/tuhucar/cli.git", branch: "main"

  def install
    asset = if OS.mac?
      Hardware::CPU.arm? ? "tuhucar-darwin-arm64" : "tuhucar-darwin-x64"
    elsif OS.linux?
      Hardware::CPU.arm? ? "tuhucar-linux-arm64" : "tuhucar-linux-x64"
    else
      raise "Unsupported platform"
    end

    chmod 0755, asset
    bin.install asset => "tuhucar"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tuhucar --version")
  end
end
EOF

echo "Wrote Homebrew formula to ${OUTPUT_PATH} for version ${VERSION} (${REVISION})."
