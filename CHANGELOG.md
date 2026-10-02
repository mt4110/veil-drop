# Changelog

## 0.1.0 — 2026-10-02

Initial source release.

- Rust CLI and public library for short UTF-8 secrets encrypted using AES-256-GCM.
- Static Web Crypto receiver with copy/clear controls and fragment removal.
- Fresh OS-random key/IV, typed invalid-input errors, bounded input, strict encoding,
  HTTPS base URL validation, and redacted encrypted-payload Debug output.
- Panic regression tests, authenticated tampering tests, AES-GCM known-answer vector,
  Rust-to-Web Crypto interoperability checks, and CI architecture guard.
- MIT license, README, contribution/security policies, PR/issue templates,
  dependency-update configuration, CI auditing, and checked GitHub Pages deployment.

Experimental: no independent audit, expiration, revocation, or one-time access.
