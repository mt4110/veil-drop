// Deliberately small lexical guard, not a complete Rust/JavaScript static analyzer.
import { readFileSync, readdirSync } from 'node:fs';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const read = (file) => readFileSync(path.join(root, file), 'utf8');
const code = (source) => source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
for (const file of readdirSync(path.join(root, 'src/core')).filter((name) => name.endsWith('.rs'))) {
  const source = code(read(`src/core/${file}`)).split('#[cfg(test)]')[0];
  assert.doesNotMatch(source, /\b(?:crate|super|veil_drop)::(?:interface|cli)\b|\b(?:arboard|clap|dialoguer)::/, `core/${file}: core must not depend on interface`);
  assert.doesNotMatch(source, /\bstd::(?:io|fs|net|process)\b|\bstd::\{[^}]*\b(?:io|fs|net|process)\b|\b(?:reqwest|tokio|ureq)::|\b(?:print|println|eprint|eprintln|dbg)!/, `core/${file}: no I/O in the core (OS entropy is provided by aes-gcm)`);
  assert.doesNotMatch(source, /\.(?:unwrap|expect)\s*\(|\b(?:panic|todo|unimplemented|unreachable)!/, `core/${file}: propagate errors instead of explicitly panicking`);
}
assert.doesNotMatch(code(read('src/core/crypto.rs')).split('#[cfg(test)]')[0], /\burl::|\b(?:crate|super)::engine\b/, 'crypto must not depend on URL orchestration');
for (const file of readdirSync(path.join(root, 'docs')).filter((name) => name.endsWith('.js'))) {
  const source = code(read(`docs/${file}`));
  assert.doesNotMatch(source, /\b(?:fetch|XMLHttpRequest|WebSocket|EventSource|sendBeacon|localStorage|sessionStorage|indexedDB|eval)\b|\.innerHTML\b|\.outerHTML\b|document\.write\b|document\.cookie\b|import\s*\(/, `docs/${file}: no communication, persistent storage, HTML injection, or dynamic imports`);
  for (const match of source.matchAll(/\b(?:import|export)\s+[\s\S]*?\bfrom\s+['"]([^'"]+)['"]/g)) {
    assert.match(match[1], /^\.\/[a-z-]+\.js$/, `docs/${file}: only local static JS imports`);
  }
}
const html = read('docs/index.html');
assert.match(html, /connect-src 'none'/, 'receiver CSP must deny outbound API requests');
assert.match(html, /form-action 'none'/, 'receiver CSP must deny form submissions');
assert.match(html, /base-uri 'none'/, 'receiver CSP must prevent base URL injection');
assert.match(html, /name="referrer" content="no-referrer"/, 'receiver must suppress referrers');
assert.doesNotMatch(html, /<(?:script|link|iframe)[^>]*(?:src|href)=["'](?:https?:)?\/\//i, 'no third-party executable assets');
console.log('Architecture guard passed: core boundaries and static receiver policy.');
