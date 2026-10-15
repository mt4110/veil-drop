# Contributing

Keep changes small, reviewable, and within the current short-text sharing scope.
Use synthetic input only in issues, tests, screenshots, logs, and examples.
For security reports, follow [SECURITY.md](SECURITY.md) instead of public issues.

## Architecture

- `src/core/crypto.rs`: encoding, bounded authenticated encryption/decryption.
- `src/core/engine.rs`: receiver URL validation and orchestration; no application I/O.
- `src/interface/cli.rs`: CLI options, hidden prompt, bounded stdin.
- `src/bin/veil-drop.rs`: entry point, stdout/status messages, clipboard.
- `docs/crypto.js`: browser wire-format validation and Web Crypto decryption.
- `docs/app.js`: transient receiver state and user interactions.

Run `mise install --locked` and the README development commands before opening a PR. Node
tests require a built debug Rust binary (`mise exec -- cargo build --locked`). Run
`mise exec -- npm run lint:markdown` to check Markdown. The architecture check is a
lexical guard with explicit boundaries (including no explicit core panic/unwrap),
not a complete parser or security proof;
review new pathways manually and extend tests when a rule needs to change.

The PR template asks for purpose, observed behavior, verification, and security
impact. A maintainer reviews the final diff and target commit's CI before merging.
Dependency PRs should explain why the dependency is needed, its maintenance/license
impact, and any security changes. Keep `Cargo.lock` committed. External Actions are
pinned by SHA and updated by Dependabot. No contributor agreement is required;
contributions are licensed under this repository's MIT license.

## Release procedure

Update the exact versions in `mise.toml`, then refresh `mise.lock` for the supported
platforms. Verify installation with `mise install --locked`; PR CI also checks that
its active Rust and Node.js versions match the pins.

1. Update package version and CHANGELOG; finish required checks on the target commit.
2. Review the intended source/artifacts. The initial release contains source only.
3. Create an annotated signed tag: `git tag -s vX.Y.Z -m 'Release vX.Y.Z'`.
4. Verify it locally using a trusted configured signing key: `git verify-tag vX.Y.Z`.
5. Push the reviewed commit and tag; create release notes for the exact tag.
6. Confirm remote signature verification, target-SHA CI, and Pages deployment.

Do not move or replace a published release tag. Publish a new patch version for
fixes. GitHub Pages follows checked `main`, not a tag; the receiver can therefore
change after a source release. For a pinned receiver, self-host `docs/` from a
verified tag and protect the hosting account.

## Ubuntu distribution candidate

The separate `Ubuntu distribution candidate` workflow builds the GNU release on
Ubuntu 24.04, packages the selected dependency and Rust runtime license texts,
and tests the extracted binary in another Ubuntu 24.04 container without Rust.
`VEIL_DROP_TEST_BINARY` selects that binary for the existing Web Crypto tests;
without it, local tests keep using `target/debug/veil-drop`.

The candidate ZIP and SHA256SUMS are retained as CI artifacts, never automatically
uploaded to a Release. Review BUILD.json, runtime library requirements, original
license texts and final-SHA checks before a separate publication decision.
Do not change the existing v0.1.0 tag. A candidate filename is not a new release.
