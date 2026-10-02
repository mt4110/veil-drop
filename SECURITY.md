# Security policy

## Status and reporting

v0.1.x is experimental and receives fixes on `main`. There has been no independent
security audit of veil-drop. Automated tests are not proof that all vulnerabilities
have been eliminated.

Report vulnerabilities privately through
[GitHub's private vulnerability reporting](https://github.com/mt4110/veil-drop/security/advisories/new).
Include version, platform, impact, and a reproduction using synthetic secrets.
Do not publish real keys, bearer URLs, or plaintext in public issues. If private
reporting is unavailable, open a public issue asking for a private contact without
including exploit details. There is no guaranteed response SLA or bounty program.

## Trust boundaries

- **Sender:** trust the executable, dependencies, OS randomness, terminal, and device.
- **Transport:** the full link includes both key and ciphertext. Anyone who obtains
  it can decrypt; use a trusted communication channel. Encryption does not conceal
  plaintext from a service that can read the complete shared URL.
- **Receiver:** trust the HTTPS origin, hosting account, browser, and extensions.
  Delivered JavaScript can read fragments and plaintext. Non-extractable Web Crypto
  keys do not stop a script that already has the raw key or decrypts the plaintext.
- **Hosting:** the fragment is excluded from ordinary HTTP requests. Public HTML,
  JS and CSS still download, and hosting providers see IP/request metadata.
- **Endpoint remnants:** plaintext and URLs may remain in clipboard managers,
  terminal scrollback, screenshots, browser history/sync, and other application
  memory. Replacing the current URL and clearing the textarea reduce exposure;
  neither guarantees erasure or removes the original message.

## Implemented controls

- AES-256-GCM; new random 256-bit key and 96-bit IV on every encryption; 128-bit tag.
  OS entropy errors are propagated. The CLI never lets the user supply a reused IV.
- Strict Base64URL and length checks, 1–1,024-byte UTF-8 plaintext, bounded stdin.
  Invalid key sizes return errors rather than panicking at a fixed-length cast.
- HTTPS receiver URL validation; loopback HTTP for development; no URL credentials,
  queries, or pre-existing fragments. Secret command-line arguments are unsupported.
- No analytics, application persistence, or decryption API calls. Plaintext is
  assigned to textarea value, never interpreted as HTML. Raw decoded browser key
  buffers are cleared after use, but JS strings and browser memory are not reliably
  zeroizable and the Rust implementation makes no memory-erasure guarantee.
- Meta CSP: `default-src 'none'`, same-origin scripts/styles, `connect-src 'none'`,
  `base-uri 'none'`, `form-action 'none'`, `object-src 'none'`; no-referrer policy.
  CSP is defense in depth, not a substitute for trustworthy delivered code.
- Receiver consumes the current fragment before processing, clears visible text
  on page exit, and ignores stale async results after navigation or explicit clear.
- No keys/payloads in `EncryptedPayload` Debug output or receiver console errors.
- CI architecture guard, malformed-input panic regressions, interoperability tests,
  RustSec lockfile auditing, and SHA-pinned Actions.

## Limits and non-goals

Links are bearer credentials, reusable indefinitely, without revocation, expiry,
recipient authentication, sender identity, or one-time access. This static design
cannot enforce those controls; they require a different trust/storage design.
The authenticated tag detects tampering under the supplied key, not the sender's
identity. Do not use this tool as a password manager or as a substitute for an
authenticated end-to-end messaging system.

GitHub Pages does not provide custom security response headers. Meta CSP cannot
apply `frame-ancestors`, so framing/clickjacking protection is not guaranteed.
A self-hosted deployment should set `frame-ancestors 'none'` in an HTTP CSP header.
This project does not claim zero trust, verified safety, guaranteed memory erasure,
or a complete supply-chain defense. Tests primarily exercise supported desktop
Rust targets and Web Crypto; embedded processors and older browsers are not supported.

## Sources

- [URI fragments and HTTP requests (MDN)](https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Fragment)
- [AES-GCM crate security notes, version 0.10.3](https://docs.rs/aes-gcm/0.10.3/aes_gcm/)
- [Content Security Policy (MDN)](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy)
- [RustSec advisory database](https://rustsec.org/)
