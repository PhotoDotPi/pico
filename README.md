# pico

**A modern, backward-compatible replacement for the Unix `echo` utility.**

[![Rust](https://img.shields.io/badge/rust-1.85%2B-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org) [![Edition 2024](https://img.shields.io/badge/edition-2024-000000?style=flat-square&logo=rust&logoColor=white)](https://doc.rust-lang.org/edition-guide/) [![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-22D3EE?style=flat-square)](./LICENSE) [![CI](https://img.shields.io/github/actions/workflow/status/PhotoDotPi/pico/ci.yml?style=flat-square&label=ci)](https://github.com/PhotoDotPi/pico/actions)

---

## What is pico?

`pico` is a drop-in replacement for the standard `echo` command. It speaks the same flags (`-n`, `-e`, `-E`) and honours `POSIXLY_CORRECT`, so existing scripts keep working untouched. On top of that it offers an **extended mode** (`pico --extended`) with case conversion, encoding (hex, base64, JSON, URL), width and alignment, repetition, line numbering, and colour — all in a single, dependency-free binary.

No hosted service, no configuration files, no runtime dependencies. Build it, put it on your `PATH`, and use it.

## Features

**Compatibility**

- Drop-in replacement for POSIX `echo`
- `-n` suppresses the trailing newline
- `-e` / `-E` enable or disable backslash escape interpretation
- `POSIXLY_CORRECT` mode treats options as data
- `PICO_COMPAT=1` disables extended mode entirely

**Extended mode** (`pico --extended`)

- Case conversion: upper, lower, ASCII variants
- Encoders: hex, base64 (standard / url-safe, padded / unpadded), JSON, URL percent-encoding
- Width padding and alignment (left, right, centre) with truncation
- Repetition with custom separators
- Line numbering
- Colour output with `auto` / `always` / `never`, `NO_COLOR` support, and named styles
- Deterministic transformation order

## Requirements

- Rust 1.85 or newer (edition 2024)
- A Unix-like environment, or Windows with a compatible shell

## Installation

### From source

```sh
git clone https://github.com/PhotoDotPi/pico.git
cd pico
cargo build --release
./target/release/pico --version
```

### Install locally

```sh
cargo install --path .
```

### Package managers

```sh
# Homebrew (macOS/Linux)
brew tap PhotoDotPi/tap
brew install pico

# Scoop (Windows)
scoop bucket add photodotpi https://github.com/PhotoDotPi/scoop-bucket
scoop install pico

# winget (Windows)
winget install PhotoDotPi.pico

# Chocolatey (Windows)
choco install pico

# crates.io
cargo install pico
```

### One-line installer

```sh
# macOS/Linux
curl -fsSL https://raw.githubusercontent.com/PhotoDotPi/pico/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/PhotoDotPi/pico/main/install.ps1 | iex
```

## Usage

```sh
# Plain echo
pico hello world

# Suppress newline
pico -n hi

# Backslash escapes
pico -e "a\tb"

# Extended mode
pico --extended --case=upper "hello"
pico --extended --width=20 --align=right "hi"
pico --extended --repeat=3 --separator=" | " "pico"
pico --extended --hex "pico"
pico --extended --base64=standard "pico"
pico --extended --json "pico"
pico --extended --url=component "a b/c"
pico --extended --number-lines "one\ntwo"
pico --extended --color=always --style=success "ok"
```

Run `pico --help` for the compatibility help and `pico --extended --help` for the full extended option reference.

## Development

```sh
cargo build            # build
cargo test             # run the test suite
cargo clippy --all     # lint
cargo fmt              # format
```

Contributions are welcome — see [CONTRIBUTING.md](./CONTRIBUTING.md). Bug reports use the issue templates; report security issues privately as described in [SECURITY.md](./SECURITY.md).

## License

pico is licensed under the [GNU Affero General Public License v3.0](./LICENSE). Copyright (c) 2026 PhotoDotPi.

## Security

Please report vulnerabilities privately. See [SECURITY.md](./SECURITY.md).

## Code of Conduct

This project follows the [Contributor Covenant](./CODE_OF_CONDUCT.md).

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).
