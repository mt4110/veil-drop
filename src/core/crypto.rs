// src/core/crypto.rs
//
// AES-256-GCM 暗号化ロジック
//
// 【設計上の注意点】
// - Key (32 bytes) と IV/Nonce (12 bytes) は毎回 OsRng で生成する。
//   絶対に再利用しないこと（GCM の安全保証が崩壊する）。
// - Payload のバイナリレイアウト:
//   [ IV (12 bytes) ][ Ciphertext ][ AuthTag (16 bytes) ]
//   aes-gcm クレートは Ciphertext + AuthTag を一体で返すため、
//   手動で分離する必要はない。
// - Base64URL (No Padding) を使う理由: URL フラグメント内で
//   `+` や `/` や `=` がエスケープ問題を引き起こすのを防ぐ。

use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use thiserror::Error;

/// Short text only; keeps the default share URL below 2 KiB.
pub const MAX_SECRET_BYTES: usize = 1024;
const MAX_PAYLOAD_B64: usize = ((MAX_SECRET_BYTES + 28) * 4).div_ceil(3);

/// クリプトモジュール固有のエラー型
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed: invalid key, IV, or ciphertext")]
    DecryptionFailed,
    #[error("Invalid Base64URL payload")]
    InvalidPayload,
    #[error("Payload too short (12-byte IV and 16-byte authentication tag required)")]
    PayloadTooShort,
    #[error("AES-256-GCM requires a 32-byte key")]
    InvalidKeyLength,
    #[error("Secret must contain 1 to 1024 UTF-8 bytes")]
    InvalidSecretLength,
    #[error("Payload exceeds the 1024-byte secret limit")]
    PayloadTooLarge,
    #[error("Operating system randomness unavailable")]
    RandomnessUnavailable,
}

/// 暗号化の出力をまとめた構造体
pub struct EncryptedPayload {
    /// Base64URL (No Padding) エンコードされた [IV || Ciphertext+Tag]
    pub payload_b64: String,
    /// Base64URL (No Padding) エンコードされた 256-bit Key
    pub key_b64: String,
}

// Avoid accidentally logging a bearer link's key through Debug.
impl std::fmt::Debug for EncryptedPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedPayload")
            .field("payload_b64", &"[redacted]")
            .field("key_b64", &"[redacted]")
            .finish()
    }
}

/// 平文テキストを AES-256-GCM で暗号化する。
///
/// 毎呼び出しで新しい Key と IV を OsRng から生成する。
/// ランダム衝突の確率はゼロではなく、絶対的な一意性は保証しない。
pub fn encrypt(plaintext: &str) -> Result<EncryptedPayload, CryptoError> {
    if plaintext.is_empty() || plaintext.len() > MAX_SECRET_BYTES {
        return Err(CryptoError::InvalidSecretLength);
    }
    // --- 鍵と IV の生成 ---
    let mut key = [0u8; 32];
    let mut iv = [0u8; 12];
    OsRng
        .try_fill_bytes(&mut key)
        .map_err(|_| CryptoError::RandomnessUnavailable)?;
    OsRng
        .try_fill_bytes(&mut iv)
        .map_err(|_| CryptoError::RandomnessUnavailable)?;
    let nonce = Nonce::from_slice(&iv);

    // --- 暗号化 ---
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::InvalidKeyLength)?;
    let ciphertext_with_tag = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;
    // aes-gcm は Ciphertext + AuthTag(16B) を連結した Vec<u8> を返す。

    // --- Payload 組み立て: [IV (12B)] + [Ciphertext+Tag] ---
    let mut payload = Vec::with_capacity(12 + ciphertext_with_tag.len());
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&ciphertext_with_tag);

    // --- Base64URL エンコード ---
    Ok(EncryptedPayload {
        payload_b64: URL_SAFE_NO_PAD.encode(&payload),
        key_b64: URL_SAFE_NO_PAD.encode(key),
    })
}

/// Base64URL エンコードされた payload と key からUTF-8テキストへ復号する。
pub fn decrypt(payload_b64: &str, key_b64: &str) -> Result<String, CryptoError> {
    if payload_b64.len() > MAX_PAYLOAD_B64 {
        return Err(CryptoError::PayloadTooLarge);
    }
    if key_b64.len() != 43 {
        return Err(CryptoError::InvalidKeyLength);
    }
    // --- デコード ---
    let payload = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| CryptoError::InvalidPayload)?;
    let key_bytes = URL_SAFE_NO_PAD
        .decode(key_b64)
        .map_err(|_| CryptoError::InvalidPayload)?;

    if payload.len() < 29 {
        return Err(CryptoError::PayloadTooShort);
    }
    if payload.len() > MAX_SECRET_BYTES + 28 {
        return Err(CryptoError::PayloadTooLarge);
    }

    // --- Payload 分解: IV(12B) + Ciphertext+Tag ---
    let (iv_bytes, ciphertext_with_tag) = payload.split_at(12);
    let nonce = Nonce::from_slice(iv_bytes);

    // --- 復号 ---
    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::InvalidKeyLength)?;
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext_with_tag)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext_bytes).map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_key_lengths_never_panic() {
        let encrypted = encrypt("secret").unwrap();
        for length in 0..=128 {
            if length == 32 {
                continue;
            }
            let key = URL_SAFE_NO_PAD.encode(vec![0; length]);
            let result = std::panic::catch_unwind(|| decrypt(&encrypted.payload_b64, &key));
            assert!(
                matches!(result, Ok(Err(CryptoError::InvalidKeyLength))),
                "key length {length}"
            );
        }
    }

    #[test]
    fn malformed_payloads_never_panic() {
        let key = URL_SAFE_NO_PAD.encode([0u8; 32]);
        for length in 0..29 {
            let payload = URL_SAFE_NO_PAD.encode(vec![0; length]);
            let result = std::panic::catch_unwind(|| decrypt(&payload, &key));
            assert!(
                matches!(result, Ok(Err(CryptoError::PayloadTooShort))),
                "payload length {length}"
            );
        }
        for input in ["!", "AA=", "a", "こんにちは", "++//", "\n", "AB"] {
            assert!(matches!(
                std::panic::catch_unwind(|| decrypt(input, &key)),
                Ok(Err(_))
            ));
        }
        let oversized = "A".repeat(MAX_PAYLOAD_B64 + 1);
        assert!(matches!(
            decrypt(&oversized, &key),
            Err(CryptoError::PayloadTooLarge)
        ));
        assert!(matches!(
            decrypt(&URL_SAFE_NO_PAD.encode(vec![0; 1053]), &key),
            Err(CryptoError::PayloadTooLarge)
        ));
    }

    #[test]
    fn iv_ciphertext_and_tag_tampering_are_rejected() {
        let encrypted = encrypt("日本語の秘密\n").unwrap();
        let payload = URL_SAFE_NO_PAD.decode(&encrypted.payload_b64).unwrap();
        for index in 0..payload.len() {
            let mut changed = payload.clone();
            changed[index] ^= 1;
            assert!(matches!(
                decrypt(&URL_SAFE_NO_PAD.encode(changed), &encrypted.key_b64),
                Err(CryptoError::DecryptionFailed)
            ));
        }
    }

    #[test]
    fn secret_size_boundaries_and_utf8_are_preserved() {
        assert!(matches!(encrypt(""), Err(CryptoError::InvalidSecretLength)));
        assert!(matches!(
            encrypt(&"x".repeat(1025)),
            Err(CryptoError::InvalidSecretLength)
        ));
        for text in ["\u{feff}日本語\r\n\n".to_string(), "x".repeat(1024)] {
            let encrypted = encrypt(&text).unwrap();
            assert_eq!(
                decrypt(&encrypted.payload_b64, &encrypted.key_b64).unwrap(),
                text
            );
        }
    }

    #[test]
    fn aes256_gcm_known_answer() {
        // NIST AES-256-GCM vector: zero key/IV, sixteen zero plaintext bytes.
        let key = URL_SAFE_NO_PAD.encode([0u8; 32]);
        let mut payload = vec![0u8; 12];
        payload.extend(
            hex::decode("cea7403d4d606b6e074ec5d3baf39d18d0d1c8a799996bf0265b98b5d48ab919")
                .unwrap(),
        );
        assert_eq!(
            decrypt(&URL_SAFE_NO_PAD.encode(payload), &key)
                .unwrap()
                .as_bytes(),
            &[0u8; 16]
        );
    }

    #[test]
    fn debug_does_not_include_keys() {
        let encrypted = encrypt("secret").unwrap();
        let debug = format!("{encrypted:?}");
        assert!(!debug.contains(&encrypted.key_b64));
        assert!(!debug.contains(&encrypted.payload_b64));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "super-secret-api-key-12345!@#";
        let encrypted = encrypt(original).expect("encryption should succeed");

        // payload と key が空でないことを確認
        assert!(!encrypted.payload_b64.is_empty());
        assert!(!encrypted.key_b64.is_empty());

        // 復号して元のテキストと一致することを確認
        let decrypted =
            decrypt(&encrypted.payload_b64, &encrypted.key_b64).expect("decryption should succeed");
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_payloads() {
        let plaintext = "same secret";
        let enc1 = encrypt(plaintext).unwrap();
        let enc2 = encrypt(plaintext).unwrap();

        // 2回の独立したOS乱数サンプルが一致しないことのスモーク確認。
        assert_ne!(enc1.payload_b64, enc2.payload_b64);
        assert_ne!(enc1.key_b64, enc2.key_b64);
    }

    #[test]
    fn test_tampered_key_fails_decryption() {
        let encrypted = encrypt("secret").unwrap();
        // key を 1 バイト変えて復号を試みる
        let mut bad_key = URL_SAFE_NO_PAD.decode(&encrypted.key_b64).unwrap();
        bad_key[0] ^= 0xff;
        let bad_key_b64 = URL_SAFE_NO_PAD.encode(&bad_key);

        let result = decrypt(&encrypted.payload_b64, &bad_key_b64);
        assert!(result.is_err());
    }

    #[test]
    fn test_base64url_no_padding() {
        let encrypted = encrypt("test").unwrap();
        // No Padding: `=` が含まれないこと
        assert!(!encrypted.payload_b64.contains('='));
        assert!(!encrypted.key_b64.contains('='));
        // URL Safe: `+` と `/` が含まれないこと
        assert!(!encrypted.payload_b64.contains('+'));
        assert!(!encrypted.payload_b64.contains('/'));
    }
}
