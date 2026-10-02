# veil-drop

Encrypt short text into a shareable URL. The Rust CLI creates an AES-256-GCM
ciphertext and a fresh random key; the static receiver decrypts it with Web Crypto
in the recipient's browser. There is no backend for storing secrets.

[Receiver](https://mt4110.github.io/veil-drop/) · [Security model](SECURITY.md) ·
[Contributing](CONTRIBUTING.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)

> **The complete URL is a secret. Anyone who has it can decrypt the text.**
> This is an experimental v0.1.0 tool, not an independently audited secret manager.
> Links do not expire, cannot be revoked, and are not limited to one opening.

## Install

Install [Rust](https://www.rust-lang.org/tools/install), then build the signed release tag:

```sh
git clone https://github.com/mt4110/veil-drop.git
cd veil-drop
git checkout v0.1.0
cargo install --path . --locked
veil-drop --version
```

This release is source-only: there are no prebuilt binaries or crates.io package.
Rust stable is tested on macOS, Linux, and Windows in CI. Desktop clipboard support
depends on your environment; headless use works with `--no-clipboard`.

## Use

Run without arguments to enter a secret in a hidden terminal prompt:

```sh
veil-drop
```

Or read a UTF-8 file through stdin. Keep private files outside this repository:

```sh
veil-drop --no-clipboard < /path/to/private-secret.txt
```

The CLI prints **only the URL to stdout** and status messages to stderr. It tries to
copy the URL to the system clipboard unless `--no-clipboard` is set. The URL is just
as sensitive as the original text; terminal logs, redirect targets, clipboard
history, and cloud clipboard sync can retain it. Avoid putting a real secret in a
shell command, argument, issue, log, or screenshot.

Send the complete URL through a trusted channel. The receiver removes the fragment
from the current address/history entry before decrypting. Use **Copy** to copy the
plaintext or **Clear display** to remove it from the page. Refreshing loses the
current display; opening the original link again still works.

Inputs must contain **1–1,024 UTF-8 bytes**, not characters. Stdin preserves all
whitespace, including trailing newlines; invalid UTF-8 is rejected. The limit is a
conservative short-link policy: the default receiver URL remains below 2 KiB at the
limit. Messaging services may still truncate, rewrite, or reject a link.

## Self-host the receiver

Serve the contents of `docs/` from a trusted HTTPS origin, then select that origin:

```sh
veil-drop --base-url https://secrets.example.org/ --no-clipboard
```

Only HTTPS is accepted, with HTTP allowed for `localhost` and loopback addresses
for development. The base URL must point to a receiver **directory**, with no
credentials, query, or fragment. Pointing at a server you do not trust defeats the
security model: its JavaScript can read the whole link and the decrypted text.

For local testing (Python 3 required):

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory docs
# In another terminal:
veil-drop --base-url http://127.0.0.1:8000/ --no-clipboard
```

For hosts that support response headers, add `Content-Security-Policy` with the same
directives as `docs/index.html`, plus `frame-ancestors 'none'`, and add
`Referrer-Policy: no-referrer`. GitHub Pages cannot set these custom headers; its
meta CSP does not prevent the page from being embedded. See [SECURITY.md](SECURITY.md).

## How it works

1. The CLI generates a fresh 32-byte key and 12-byte IV using OS randomness.
2. AES-256-GCM encrypts UTF-8 text with a 16-byte authentication tag, without AAD.
3. The link uses unpadded, canonical Base64URL:

```text
https://mt4110.github.io/veil-drop/#payload=<IV || ciphertext || tag>&key=<32-byte key>
```

4. The receiver validates lengths and encoding, imports a non-extractable,
   decrypt-only key, verifies the tag, and displays text in a readonly textarea.

Browsers exclude fragments from ordinary HTTP requests. This protects them from
ordinary receiver access logs, but does **not** protect the full URL from messaging
platforms, browser extensions, browser history/sync, or a compromised receiver.
The host still receives requests for the public page/assets and associated metadata.
No analytics, external executable assets, application storage, or decryption API
requests are used. The CSP blocks fetch/WebSocket-style connections; it is defense
in depth and does not make compromised hosting trustworthy.

## Library

```rust
use veil_drop::core::engine::{generate_share_url, EngineError};

fn main() -> Result<(), EngineError> {
    let _link = generate_share_url("synthetic example", None)?;
    // Treat the link as secret: do not log it.
    Ok(())
}
```

`core::crypto` exposes `encrypt`, `decrypt`, and `MAX_SECRET_BYTES`.
Malformed public input returns typed errors, including invalid key length, truncated
payload, oversized input, invalid base URL, and unavailable OS entropy. The core
performs no application I/O; the CLI owns stdin/stdout and clipboard handling.

## Development and releases

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
npm run check:architecture
npm test
```

Node.js 22+ is needed only for development tests, with no npm dependencies or install
step. Rust regressions cover panic-prone malformed inputs, authentication failures,
UTF-8 and size boundaries, and a known-answer AES-256-GCM vector. Receiver tests
verify that URLs produced by the actual Rust binary decrypt through the production
Web Crypto code. The lexical architecture guard checks core boundaries and receiver
policy; it is not a complete static analyzer.

CI runs these checks on pull requests and pushes, and RustSec auditing checks the
lockfile. Pages deploys `docs/` only after checks pass for `main`. GitHub Actions are
pinned to commit SHAs. See [CONTRIBUTING.md](CONTRIBUTING.md) for review and signed-tag
release steps. Dependencies retain their own licenses; `Cargo.lock` records the
resolved versions.
