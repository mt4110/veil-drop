# veil-drop

**日本語** · [English](README.en.md)

短いテキストを暗号化し、URLで共有するツールです。Rust CLIがAES-256-GCMで暗号化し、
受信ページがブラウザー内のWeb Crypto APIで復号します。秘密を保存するサーバーや
データベースはありません。

[受信ページ](https://mt4110.github.io/veil-drop/) ·
[セキュリティ](SECURITY.md) · [開発への参加](CONTRIBUTING.md) ·
[変更履歴](CHANGELOG.md) · [MITライセンス](LICENSE)

> **共有URL全体が秘密情報です。URLを持つ人は誰でも復号できます。**
> このv0.1.0は実験的な公開版で、第三者のセキュリティ監査は受けていません。
> URLに有効期限・失効機能・一度限りの制限はありません。

## インストール

[Git](https://git-scm.com/)と[mise](https://mise.jdx.dev/)を用意します。
リポジトリを取得してから、固定されたRustとNode.jsをインストールし、ソースからCLIを導入します。

```sh
git clone https://github.com/mt4110/veil-drop.git
cd veil-drop
mise install --locked
mise exec -- cargo install --path . --locked
veil-drop --version
```

ビルド済みバイナリやcrates.ioパッケージは配布していません。
macOS・Linux・Windowsで、CIが同じRustバージョンを検証します。クリップボードは
デスクトップ環境により動作が異なります。GUIのない環境では`--no-clipboard`を使います。

## 使い方

引数を付けずに起動すると、秘密を画面に表示しない入力プロンプトが開きます。

```sh
veil-drop
```

UTF-8のファイルは標準入力から渡せます。秘密ファイルはこのリポジトリの外に置いてください。

```sh
veil-drop --no-clipboard < /path/to/private-secret.txt
```

標準出力にはURLだけを表示し、状態メッセージは標準エラーに表示します。
`--no-clipboard`を付けない場合、URLのクリップボードへのコピーも試みます。
URLは元のテキストと同じように慎重に扱ってください。端末ログ、リダイレクト先、
クリップボード履歴、クラウド同期などに残る場合があります。実際の秘密をシェルの
コマンド、引数、Issue、ログ、スクリーンショットに含めないでください。

完全なURLを信頼できる経路で相手に送ります。受信ページは復号前に、今開いている
ページのURLから鍵情報を取り除きます。「コピー」で平文をコピーできます。「表示を消す」
ボタンやページの再読み込みで、画面の平文を消去できます。元の共有URLが残っていれば、
再び開いて復号できます。

秘密のサイズ上限は**1〜1,024 UTF-8バイト**です（文字数ではありません）。標準入力の
空白や末尾改行は保持し、不正なUTF-8は受け付けません。この上限では既定の共有URLが
2 KiB未満になります。メッセージサービスによってはURLが切り詰められたり、書き換え・
拒否されたりする場合があります。

## 受信ページのセルフホスト

`docs/`の内容を、信頼できるHTTPSのWebサイトで配信します。CLIにそのURLを指定します。

```sh
veil-drop --base-url https://secrets.example.org/ --no-clipboard
```

HTTPSを必須とし、開発時は`localhost`とループバックIPアドレスに限りHTTPを使えます。
ベースURLには受信ページのディレクトリを指定してください。認証情報、クエリ、既存の
フラグメントは指定できません。信頼できないサーバーを受信先にすると、そのJavaScriptに
共有URLと復号後の平文を読み取られるおそれがあります。

Python 3を使ってローカルで試せます。

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory docs
# 別の端末で実行:
veil-drop --base-url http://127.0.0.1:8000/ --no-clipboard
```

レスポンスヘッダーを設定できるホストでは、`docs/index.html`と同じContent-Security-Policy
に加え、`frame-ancestors 'none'`を設定し、`Referrer-Policy: no-referrer`を指定してください。
GitHub Pagesでは任意のセキュリティヘッダーを設定できず、HTML内のCSPだけではページの
埋め込みを防げません。詳しくは[SECURITY.md](SECURITY.md)を参照してください。

## 仕組み

1. CLIはOSの乱数で、毎回32バイトの鍵と12バイトのIVを生成します。
2. UTF-8テキストをAES-256-GCMで暗号化します。認証タグは16バイトで、AADは使いません。
3. IV・暗号文・タグ、および鍵をパディングなしのBase64URLに変換します。
4. 受信ページは形式と長さを検証し、抽出不可・復号専用の鍵をWeb Cryptoに読み込みます。
   認証タグを検証してからテキストを読み取り専用のテキスト欄に表示します。

共有URLは次の形式です。

```text
https://mt4110.github.io/veil-drop/#payload=<IV || ciphertext || tag>&key=<32-byte key>
```

ブラウザーは通常、フラグメントをHTTPリクエストに含めません。そのため、受信サーバーの
通常のアクセスログには共有URLの鍵と暗号文は送られません。ただし、メッセージサービス、
ブラウザー拡張機能、履歴同期、侵害された受信ページなどからURLが漏れる可能性があります。
受信ページのホストには、HTML・JavaScript・CSSのリクエストと接続元などの情報が伝わります。
解析ツールや外部スクリプト、アプリ内の永続保存、復号APIはありません。CSPは通信を制限する
追加の対策ですが、侵害されたホストを信頼できる状態にはしません。

## Rustライブラリ

```rust
use veil_drop::core::engine::{generate_share_url, EngineError};

fn main() -> Result<(), EngineError> {
    let _link = generate_share_url("synthetic example", None)?;
    // 共有URLは秘密情報として扱い、ログに記録しない。
    Ok(())
}
```

`core::crypto`は`encrypt`、`decrypt`、`MAX_SECRET_BYTES`を公開しています。不正な
鍵長・短すぎるデータ・上限を超える入力・無効なベースURL・OS乱数の取得失敗を、
型付きエラーとして返します。コアはアプリケーションI/Oを行わず、CLIが標準入出力と
クリップボードを担当します。

## 開発とリリース

miseで固定バージョンをインストールします。依存関係は`Cargo.lock`とnpmの
`package-lock.json`で固定されます。新しいターミナル環境では`mise exec --`を付けて実行できます。

```sh
mise install --locked
mise exec -- cargo fmt --check
mise exec -- cargo clippy --locked --all-targets -- -D warnings
mise exec -- cargo test --locked
mise exec -- cargo build --locked
mise exec -- npm ci
mise exec -- npm run check:architecture
mise exec -- npm run lint:markdown
mise exec -- npm test
```

Node.jsは開発時のMarkdown整形チェックと受信ページテストにのみ使います。
RustとNode.jsのバージョンは`mise.toml`で固定します。`mise.lock`には5つの対応プラット
フォーム用Node.js配布物のURLとSHA-256チェックサムも記録し、CIは同じバージョンを使います。
Rustのテストは不正データによるパニック、認証失敗、UTF-8とサイズ境界、既知の
AES-256-GCMテストベクトルを確認します。受信側のテストでは、実際にRust CLIが作った
URLをブラウザー標準のWeb Cryptoコードで復号します。アーキテクチャガードはコアと
受信ページの依存境界を字句検査します。完全な静的解析ではありません。

CIはこの検査をプルリクエストとpushのたびに実行し、Markdown LintとRustSecによる
依存監査も行います。GitHub Pagesは`main`のチェック成功後に`docs/`を公開します。
GitHub ActionsはコミットSHAで固定しています。レビューと署名付きタグの手順は
[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。各依存パッケージの
ライセンスはその著作権表示に従います。
