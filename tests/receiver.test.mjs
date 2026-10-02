import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { decryptPayload, parseFragment, decodeBase64url } from '../docs/crypto.js';
const binary = fileURLToPath(new URL(`../target/debug/veil-drop${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));
const b64 = (bytes) => Buffer.from(bytes).toString('base64url');
const key = new Uint8Array(32);
const nistPayload = Uint8Array.from(Buffer.from('000000000000000000000000cea7403d4d606b6e074ec5d3baf39d18d0d1c8a799996bf0265b98b5d48ab919', 'hex'));
const fragment = `payload=${b64(nistPayload)}&key=${b64(key)}`;

test('Web Crypto decrypts the AES-256-GCM known-answer vector', async () => {
  assert.equal(await decryptPayload(nistPayload, key), '\0'.repeat(16));
});

test('Rust CLI URLs decrypt with the production Web Crypto receiver', async () => {
  for (const text of ['synthetic-api-key', '\ufeff日本語\r\n\n', '<script>alert(1)</script>', 'x'.repeat(1024)]) {
    const output = execFileSync(binary, ['--no-clipboard'], { input: text, encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] }).trim();
    const url = new URL(output);
    assert.equal(url.origin, 'https://mt4110.github.io');
    assert.equal(url.pathname, '/veil-drop/');
    assert.equal(url.search, '');
    const { payload, key } = parseFragment(url.hash.slice(1));
    assert.equal(await decryptPayload(payload, key), text);
  }
});

test('invalid encodings, duplicate/missing parameters and oversized input fail', () => {
  for (const input of ['', 'A', 'AB', 'AA=', '+/==', '日本語', 'AA\n']) assert.throws(() => decodeBase64url(input));
  for (const input of ['', fragment + '&key=' + b64(key), fragment + '&extra=x', fragment.replace('&key=', '&other='), fragment.replace('payload=', 'payload=%'), 'payload=AA&key=' + b64(key), 'x'.repeat(1460)]) {
    assert.throws(() => parseFragment(input));
  }
  assert.throws(() => parseFragment(`payload=${b64(new Uint8Array(1053))}&key=${b64(key)}`));
});

test('wrong key lengths and truncated payloads fail before import/decrypt', async () => {
  for (const length of [0, 16, 24, 31, 33, 64]) await assert.rejects(decryptPayload(nistPayload, new Uint8Array(length)));
  for (const length of [0, 11, 12, 27, 28, 1053]) await assert.rejects(decryptPayload(new Uint8Array(length), key));
});

test('IV, ciphertext and authentication tag tampering fail', async () => {
  for (const index of [0, 12, nistPayload.length - 1]) {
    const changed = nistPayload.slice(); changed[index] ^= 1;
    await assert.rejects(decryptPayload(changed, key));
  }
  const wrongKey = key.slice(); wrongKey[0] ^= 1;
  await assert.rejects(decryptPayload(nistPayload, wrongKey));
});

test('authenticated non-UTF-8 plaintext is rejected', async () => {
  const iv = new Uint8Array(12);
  const imported = await crypto.subtle.importKey('raw', key, 'AES-GCM', false, ['encrypt']);
  const ciphertext = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, imported, new Uint8Array([0xff])));
  await assert.rejects(decryptPayload(new Uint8Array([...iv, ...ciphertext]), key));
});

test('CLI rejects empty, oversized input and insecure base URLs without a secret URL', () => {
  for (const { input, args } of [{ input: '', args: [] }, { input: 'x'.repeat(1025), args: [] }, { input: 'secret', args: ['--base-url', 'http://example.com'] }, { input: 'secret', args: ['--base-url', 'https://example.com/#old'] }]) {
    const result = spawnSync(binary, ['--no-clipboard', ...args], { input, encoding: 'utf8' });
    assert.notEqual(result.status, 0);
    assert.equal(result.stdout, '');
    assert.doesNotMatch(result.stderr, /panicked at|#payload=/);
  }
});
