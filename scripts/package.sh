#!/usr/bin/env bash
set -euo pipefail

target="${1:?target triple required}"
out="${2:?output dir required}"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
name="pico-${version}-${target}"
root="${out}/${name}"

mkdir -p "${root}"
cargo build --release --target "${target}"
cp "target/${target}/release/pico" "${root}/pico" 2>/dev/null || cp "target/${target}/release/pico.exe" "${root}/pico.exe"
cp LICENSE "${root}/LICENSE"

( cd "${out}" && tar -czf "${name}.tar.gz" "${name}" )
( cd "${out}" && sha256sum "${name}.tar.gz" > "${name}.tar.gz.sha256" )
echo "Created ${out}/${name}.tar.gz"
