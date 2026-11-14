#!/usr/bin/env bash
set -euo pipefail

repo="PhotoDotPi/pico"
install_dir="${PICO_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux)  os_slug="linux" ;;
  Darwin) os_slug="macos" ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64) arch_slug="x86_64" ;;
  aarch64|arm64) arch_slug="aarch64" ;;
  *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

if [ "$os_slug" = "linux" ]; then
  target="${arch_slug}-unknown-linux-gnu"
else
  target="${arch_slug}-apple-darwin"
fi

tag="$(curl -fsSL "https://api.github.com/repos/${repo}/releases/latest" | grep -o '"tag_name": "[^"]*"' | head -n1 | cut -d'"' -f4)"
version="${tag#pico-v}"
asset="pico-${version}-${target}.tar.gz"
url="https://github.com/${repo}/releases/download/${tag}/${asset}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "Downloading ${url} ..."
curl -fsSL "$url" -o "$tmp/${asset}"
tar -xzf "$tmp/${asset}" -C "$tmp"

mkdir -p "$install_dir"
if [ -f "$tmp/pico-${version}-${target}/pico" ]; then
  install -m 755 "$tmp/pico-${version}-${target}/pico" "$install_dir/pico"
else
  install -m 755 "$tmp/pico" "$install_dir/pico"
fi
echo "Installed pico to ${install_dir}/pico"
echo "Ensure ${install_dir} is on your PATH."
