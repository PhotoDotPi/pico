# Contributing to pico

Thank you for your interest in contributing. Contributions of all kinds are
welcome: bug reports, feature requests, documentation improvements, and code.

## Getting started

1. Fork the repository and clone your fork.
2. Ensure Rust 1.85+ is installed (`rustup show`).
3. Build and test:

   ```sh
   cargo build
   cargo test
   ```

## Workflow

- Create a feature branch from `main`.
- Keep changes focused and atomic.
- Follow the existing code style (`cargo fmt`, `cargo clippy --all`).
- Add or update tests for any behavior change.
- Update documentation when relevant.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new option
fix: handle empty input
docs: update README
```

## Pull requests

- Open a draft PR early if you want feedback.
- Fill in the PR template and link related issues.
- Ensure CI is green before requesting review.
- Be responsive to review feedback.

## Reporting bugs

Use the bug report issue template. Include reproduction steps, expected
behavior, and your environment.

## Security

Do not open public issues for security vulnerabilities. See [SECURITY.md](./SECURITY.md).
