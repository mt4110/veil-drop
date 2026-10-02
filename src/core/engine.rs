// src/core/engine.rs
//
// フロー制御: 入力テキストを受け取り、共有URLを組み立てて返す。
// CLI・ライブラリ双方から呼び出せるよう、アプリケーションI/Oから分離する。

use crate::core::crypto::{self, CryptoError};
use thiserror::Error;
use url::{Host, Url};

/// エンジン固有のエラー型
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("Base URL must be HTTPS (or HTTP on localhost/loopback), without credentials, query, or fragment")]
    InvalidBaseUrl,
}

/// 既定の受信先。呼び出し側はbase_url引数で上書き可能。
pub const DEFAULT_BASE_URL: &str = "https://mt4110.github.io/veil-drop/";

/// 入力テキストを暗号化し、共有用の URL フラグメントを含む完全な URL を返す。
///
/// # 引数
/// - `plaintext`   : 暗号化する機密テキスト
/// - `base_url`    : `None` の場合はデフォルト URL を使用
///
/// # 戻り値
/// - `Ok(String)` : 完全な共有 URL
pub fn generate_share_url(plaintext: &str, base_url: Option<&str>) -> Result<String, EngineError> {
    let raw_base = base_url.unwrap_or(DEFAULT_BASE_URL);
    if raw_base
        .chars()
        .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(EngineError::InvalidBaseUrl);
    }
    let mut base = Url::parse(raw_base).map_err(|_| EngineError::InvalidBaseUrl)?;
    let loopback = match base.host() {
        Some(Host::Domain("localhost")) => true,
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if !base.has_host()
        || !(base.scheme() == "https" || (base.scheme() == "http" && loopback))
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(EngineError::InvalidBaseUrl);
    }
    let path = format!("{}/", base.path().trim_end_matches('/'));
    base.set_path(&path);
    let encrypted = crypto::encrypt(plaintext)?;

    // URL フラグメント (`#` 以降) にペイロードを格納する。
    // The fragment is excluded from HTTP requests; the complete URL is a secret.
    base.set_fragment(Some(&format!(
        "payload={}&key={}",
        encrypted.payload_b64, encrypted.key_b64
    )));
    Ok(base.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_and_malformed_base_urls_return_errors_without_panics() {
        for base in [
            "",
            "garbage",
            "javascript:alert(1)",
            "file:///tmp/x",
            "http://example.com",
            "https://user:password@example.com",
            "https://example.com/?q=x",
            "https://example.com/#x",
            "https://example.com/#",
            "https://example.com/?",
            "https://example.com/\n",
            " https://example.com",
            "https://[",
            "http://localhost.evil.test",
        ] {
            assert!(
                matches!(
                    std::panic::catch_unwind(|| generate_share_url("secret", Some(base))),
                    Ok(Err(EngineError::InvalidBaseUrl))
                ),
                "base {base:?}"
            );
        }
    }

    #[test]
    fn default_url_is_bounded_and_receiver_matches() {
        let link = generate_share_url(&"x".repeat(1024), None).unwrap();
        assert!(link.starts_with(DEFAULT_BASE_URL));
        assert!(link.len() < 2048);
        let parsed = Url::parse(&link).unwrap();
        assert!(parsed.query().is_none());
        assert!(parsed.fragment().unwrap().starts_with("payload="));
    }

    #[test]
    fn loopback_development_receivers_are_supported() {
        for base in [
            "http://localhost:8000",
            "http://127.0.0.1:8000",
            "http://[::1]:8000",
        ] {
            assert!(generate_share_url("secret", Some(base)).is_ok());
        }
    }

    #[test]
    fn test_url_contains_fragment() {
        let url = generate_share_url("my secret", None).unwrap();
        // `#` が含まれること (フラグメント方式)
        assert!(url.contains('#'));
        // payload= と key= が含まれること
        assert!(url.contains("payload="));
        assert!(url.contains("key="));
    }

    #[test]
    fn test_custom_base_url() {
        let url = generate_share_url("test", Some("https://myuser.github.io/veil-drop")).unwrap();
        assert!(url.starts_with("https://myuser.github.io/veil-drop/"));
    }

    #[test]
    fn test_trailing_slash_normalized() {
        let url1 = generate_share_url("x", Some("https://example.com/veil-drop")).unwrap();
        let url2 = generate_share_url("x", Some("https://example.com/veil-drop/")).unwrap();
        // ベース部分のスラッシュ有無で二重スラッシュにならないこと
        assert!(!url1.contains("//veil-drop"));
        assert!(!url2.contains("//veil-drop"));
    }
}
