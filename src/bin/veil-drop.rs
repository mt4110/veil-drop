// src/bin/veil-drop.rs
//
// CLI エントリーポイント
// ライブラリ関数を呼び出すだけの薄いシェル。

use arboard::Clipboard;
use clap::Parser;
use veil_drop::{
    core::engine::generate_share_url,
    interface::cli::{read_secret, Cli},
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // --- 入力取得 ---
    let secret = read_secret(&cli)?;

    // --- URL 生成 ---
    let url = generate_share_url(&secret, cli.base_url.as_deref())?;

    // --- クリップボードにコピー ---
    let copied = !cli.no_clipboard
        && match Clipboard::new() {
            Ok(mut cb) => cb.set_text(&url).is_ok(),
            Err(_) => false,
        };

    // --- 出力 ---
    println!("{url}");

    if copied {
        eprintln!("Share URL copied to clipboard.");
    } else if !cli.no_clipboard {
        eprintln!("Clipboard unavailable; copy the URL from stdout.");
    }

    eprintln!(
        "Anyone with the full URL can decrypt the secret. Share it through a trusted channel."
    );

    Ok(())
}
