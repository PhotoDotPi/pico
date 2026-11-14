# Security Policy

## Supported versions

Only the latest release on the `main` branch receives security fixes.

## Reporting a vulnerability

Please do **not** open a public issue. Instead, report privately using
GitHub's private vulnerability reporting feature on this repository.

Include:

- A description of the vulnerability
- Steps to reproduce
- Affected versions
- Any potential impact

You can expect an initial response within 7 days. We will coordinate
disclosure and credit reporters unless they prefer to remain anonymous.

## Scope

`pico` is a command-line utility that reads arguments and writes to
standard output. It does not open network connections or execute
external programs. Please still report any issue that could lead to
unexpected behavior, resource exhaustion, or incorrect output.
