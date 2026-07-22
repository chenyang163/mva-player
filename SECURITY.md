# Security Policy

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, report them privately to the maintainer.

### Process

1. **Contact**: Email the maintainer directly (contact details will be published
   once a dedicated security address is established).
2. **Acknowledge**: You will receive an acknowledgment within 48 hours.
3. **Investigate**: The maintainer will investigate and keep you informed of
   progress.
4. **Disclose**: Once resolved, a security advisory will be published on GitHub.

### Scope

- Crate-level vulnerabilities in any `mva-*` crate
- Unsound `unsafe` code (there should be none — report any as a bug)
- Panics from untrusted input
- Denial-of-service vectors

### Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |

Currently only the latest release is supported. No backports are provided.
