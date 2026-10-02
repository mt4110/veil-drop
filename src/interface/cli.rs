// src/interface/cli.rs
//
// CLI インターフェース: TUI パスワードプロンプトと STDIN パイプ入力の両方に対応。
//
// 【UX 設計】
// - パイプ入力: STDIN を上限付きで読み取り、改行を含めて暗号化。
// - 引数なし実行: `dialoguer::Password` でターミナル上で文字を隠した入力プロンプトを表示。
//   Shell の history には残らない（プログラムに渡された後は環境変数や引数に露出しない）。
// - `--base-url` オプション: 自己ホスト環境向けに URL ベースを上書き可能。

use crate::core::crypto::MAX_SECRET_BYTES;
use clap::Parser;
use dialoguer::Password;

/// Encrypt short secrets into bearer URLs.
#[derive(Parser, Debug)]
#[command(
    name = "veil-drop",
    version,
    about = "Encrypt a secret and generate a shareable URL (server-free)",
    long_about = None,
)]
pub struct Cli {
    /// フロントエンドのベース URL（デフォルト: GitHub Pages 公式サイト）
    #[arg(long, value_name = "URL")]
    pub base_url: Option<String>,
    /// Leave the clipboard unchanged (stdout still contains the secret URL)
    #[arg(long)]
    pub no_clipboard: bool,
}

/// stdin がパイプ（非 TTY）かどうかを判定する
pub fn is_piped() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}

/// 入力元に応じて機密テキストを取得する。
/// stdin はそのまま保持し、TTY は非表示プロンプトを使用する。
pub fn read_secret(_cli: &Cli) -> anyhow::Result<String> {
    // パターン 2: パイプ入力
    if is_piped() {
        use std::io;
        return read_bounded(io::stdin().lock());
    }

    // パターン 3: TTY — ターミナル上で文字を隠すプロンプト
    let secret = Password::new()
        .with_prompt("🔒 Enter secret to encrypt")
        .with_confirmation("Confirm secret", "Secrets don't match, try again")
        .interact()?;

    if secret.is_empty() || secret.len() > MAX_SECRET_BYTES {
        anyhow::bail!("Secret must contain 1 to 1024 UTF-8 bytes");
    }

    Ok(secret)
}

fn read_bounded(reader: impl std::io::Read) -> anyhow::Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    reader
        .take((MAX_SECRET_BYTES + 1) as u64)
        .read_to_string(&mut buf)?;
    if buf.is_empty() || buf.len() > MAX_SECRET_BYTES {
        anyhow::bail!("Secret must contain 1 to 1024 UTF-8 bytes");
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdin_preserves_whitespace_and_newlines() {
        assert_eq!(
            read_bounded(" 日本語\r\n\n".as_bytes()).unwrap(),
            " 日本語\r\n\n"
        );
    }

    #[test]
    fn stdin_rejects_empty_oversized_and_invalid_utf8() {
        assert!(read_bounded(&b""[..]).is_err());
        assert!(read_bounded(&vec![b'a'; 1025][..]).is_err());
        assert!(read_bounded(&[0xff][..]).is_err());
        assert_eq!(read_bounded(&vec![b'a'; 1024][..]).unwrap().len(), 1024);
    }

    #[test]
    fn secrets_cannot_be_passed_as_arguments() {
        assert!(Cli::try_parse_from(["veil-drop", "secret"]).is_err());
    }
}
