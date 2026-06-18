#!/bin/sh
# null-e CLI installer:  curl -fsSL https://raw.githubusercontent.com/us/null-e/main/install.sh | sh
# Downloads the right prebuilt binary for your OS/arch from the latest GitHub release.
# ponytail: covers the platforms the release workflow ships; add a case if you add a target.
set -eu

REPO="us/null-e"
BIN="null-e"
# Install dir: first writable of these, else ~/.local/bin.
for d in /usr/local/bin "$HOME/.local/bin"; do
  if [ -d "$d" ] && [ -w "$d" ]; then DEST="$d"; break; fi
done
DEST="${DEST:-$HOME/.local/bin}"
mkdir -p "$DEST"

os=$(uname -s); arch=$(uname -m)
case "$os" in
  Darwin) o=darwin ;;
  Linux)  o=linux ;;
  *) echo "unsupported OS: $os (Windows: download the .exe from the releases page)" >&2; exit 1 ;;
esac
case "$arch" in
  arm64|aarch64) a=aarch64 ;;
  x86_64|amd64)  a=x86_64 ;;
  *) echo "unsupported arch: $arch" >&2; exit 1 ;;
esac

asset="${BIN}-${o}-${a}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"
echo "Downloading $asset ..."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "$url" -o "$tmp/a.tgz" || { echo "download failed: $url" >&2; exit 1; }
tar -xzf "$tmp/a.tgz" -C "$tmp"
binpath=$(find "$tmp" -name "$BIN" -type f | head -1)
[ -n "$binpath" ] || { echo "binary '$BIN' not found in archive" >&2; exit 1; }
install -m 755 "$binpath" "$DEST/$BIN"

echo "Installed $BIN -> $DEST/$BIN"
case ":$PATH:" in *":$DEST:"*) : ;; *) echo "Note: add $DEST to your PATH." ;; esac
