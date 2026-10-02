# Release tasks

## v0.1.0 implementation

- [x] Correct repository/receiver URLs and package metadata.
- [x] Bound UTF-8 input; preserve stdin whitespace; remove secret arguments.
- [x] Return typed errors for malformed keys/payloads and unavailable entropy.
- [x] Test malformed-input panic risks, tampering, boundaries, and known-answer vector.
- [x] Test real Rust CLI to production Web Crypto interoperability.
- [x] Consume URL fragment; clear stale state; handle clipboard failures honestly.
- [x] Replace unsupported security/lifetime guarantees with a documented threat model.
- [x] Add CI architecture guard, formatting, Clippy, platform tests, RustSec auditing.
- [x] Add OSS documentation, MIT license, PR/issue templates, and Dependabot configuration.

- [x] Validate receiver interactions in desktop and mobile-sized Chrome.
- [x] Complete local dependency audit with no advisories/warnings reported.

## Publication checks

The final remote publication evidence belongs in the GitHub release notes:
second commit SHA, verified signed tag, target-commit CI, and the deployed Pages URL.
Follow CONTRIBUTING.md; do not treat this implementation checklist as proof of deployment.

## Future changes requiring a separate decision

- Independent security review before claiming suitability for high-value secrets.
- An HTTP-header-capable self-host deployment if frame-ancestors protection is needed.
- Expiry, revocation, and one-time links require a new backend/trust design; do not
  imply that the current static implementation supplies these features.
