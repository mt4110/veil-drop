// Wire format: 12-byte IV || ciphertext || 16-byte AES-GCM tag.
export const MAX_SECRET_BYTES = 1024;
const MAX_PAYLOAD_BYTES = MAX_SECRET_BYTES + 28;
const MAX_PAYLOAD_B64 = Math.ceil(MAX_PAYLOAD_BYTES * 4 / 3);
export const MAX_FRAGMENT_LENGTH = MAX_PAYLOAD_B64 + 56;

export function decodeBase64url(value) {
  if (!value || value.length > MAX_PAYLOAD_B64 || !/^[A-Za-z0-9_-]+$/.test(value)
      || value.length % 4 === 1) {
    throw new Error('URLのデータ形式が正しくありません。');
  }
  const binary = atob(value.replace(/-/g, '+').replace(/_/g, '/')
    + '='.repeat((4 - value.length % 4) % 4));
  const canonical = btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  if (canonical !== value) throw new Error('URLのデータ形式が正しくありません。');
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

export function parseFragment(fragment) {
  if (fragment.length > MAX_FRAGMENT_LENGTH) throw new Error('共有URLが長すぎます。');
  // Reject duplicate, unknown, encoded, and reordered parameters.
  const match = /^payload=([A-Za-z0-9_-]+)&key=([A-Za-z0-9_-]{43})$/.exec(fragment);
  if (!match) throw new Error('共有URLに必要なデータがないか、形式が正しくありません。');
  const payload = decodeBase64url(match[1]);
  const key = decodeBase64url(match[2]);
  if (payload.length < 29 || payload.length > MAX_PAYLOAD_BYTES || key.length !== 32) {
    throw new Error('暗号文または鍵の長さが正しくありません。');
  }
  return { payload, key };
}

export async function decryptPayload(payload, key) {
  if (payload.length < 29 || payload.length > MAX_PAYLOAD_BYTES || key.length !== 32) {
    throw new Error('暗号文または鍵の長さが正しくありません。');
  }
  const cryptoKey = await crypto.subtle.importKey('raw', key, 'AES-GCM', false, ['decrypt']);
  let plaintext;
  try {
    plaintext = new Uint8Array(await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: payload.subarray(0, 12), tagLength: 128 },
      cryptoKey, payload.subarray(12)
    ));
    // Reject invalid UTF-8; preserve a leading BOM just like Rust String::from_utf8.
    return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(plaintext);
  } finally {
    plaintext?.fill(0);
  }
}
