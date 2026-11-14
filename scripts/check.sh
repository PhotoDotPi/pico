#!/usr/bin/env bash
set -uo pipefail

failed=()

run_step() {
    local name="$1"
    shift
    printf '\n==> %s\n' "$name"
    if "$@"; then
        return 0
    fi
    printf 'FAILED: %s (exit %d)\n' "$name" "$?" >&2
    failed+=("$name")
}

run_step 'formatting (cargo fmt --check)' cargo fmt --all -- --check
run_step 'linting (cargo clippy -D warnings)' \
    cargo clippy --all-targets --all-features -- -D warnings
run_step 'tests (cargo test)' cargo test --all-features
run_step 'docs (cargo doc)' env RUSTDOCFLAGS='-D warnings' \
    cargo doc --no-deps --all-features

if [ "${#failed[@]}" -gt 0 ]; then
    printf '\nFailed: %s\n' "${failed[*]}" >&2
    exit 1
fi
printf '\nAll fast checks passed.\n'
