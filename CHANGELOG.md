# Changelog

## 0.1.1 — Unreleased

- Make Japanese the default README and add an English version
  with language links at the top of both files.
- Pin Rust and Node.js with mise and check the same versions in CI; record
  platform-specific Node.js URLs and checksums.
- Lint project documentation and contribution templates with MarkdownLint in CI.

- Prepare an Ubuntu 24.04 x86_64 GNU ZIP and external/internal SHA256SUMS;
  retain source revision, toolchain and dependency information in BUILD.json.
- Include project, selected dependency and Rust runtime license/notice texts.
- Verify the extracted CLI without Rust in a separate Ubuntu 24.04 container,
  using synthetic generation/decryption, input-boundary and tampering tests.
- Document installation, dynamic-linking scope and unverified browser UI/clipboard behavior.
- Update dialoguer to 0.12 and thiserror to 2; maintain locked dependencies and
  update the GitHub Pages deployment actions.

This prepares the first prebuilt distribution. It is not published yet. The
cryptographic protocol and receiver behavior remain unchanged from v0.1.0.
The published v0.1.0 and the earlier 0.1.0 distribution candidate remain unchanged.

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
