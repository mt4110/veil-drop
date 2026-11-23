# Ubuntu 24.04 x86_64 向けインストール

対象はUbuntu 24.04 x86_64です。musl静的版ではなく、システムのglibc等を使います。
他ディストリビューション・旧glibc・ARM64への対応や性能は保証しません。
初回導線は`--no-clipboard`を使い、X11/Waylandのコピー動作は確認範囲外です。

## 取得済みZIPから使う

GitHub ReleaseのZIP、SHA256SUMS、veil-drop.sigstore.jsonを、新しい同じディレクトリへ保存します。
Rust・Git・Node.jsは通常利用に不要です。展開にはunzipを使います。

```sh
gh attestation verify veil-drop-0.2.0-x86_64-unknown-linux-gnu.zip \
  --repo mt4110/veil-drop --source-ref refs/tags/v0.2.0 \
  --signer-workflow mt4110/veil-drop/.github/workflows/linux-distribution.yml \
  --bundle veil-drop.sigstore.json &&
sha256sum --strict -c SHA256SUMS &&
unzip veil-drop-0.2.0-x86_64-unknown-linux-gnu.zip -d cli &&
(cd cli && sha256sum -c SHA256SUMS) &&
./cli/veil-drop --version
```

ZIP名のOK、内部ファイルのOK、`veil-drop 0.2.0`を確認します。
GitHub CLI（gh）は来歴の検証に使います。BUILD.jsonにはソースのコミットとツールチェーンを記録しています。
INSTALL.md・LICENSE・SECURITY.md・第三者表記は実行ファイルと一緒に保持してください。

端末で以下を起動し、非表示プロンプトへ入力します。動作確認には架空の短文を使います。
次回も保存した実行ファイルの絶対パスで起動できます。

```sh
/absolute/path/to/cli/veil-drop --no-clipboard
```

標準出力の共有URL全体が秘密です。端末ログ・Issue・スクリーンショットへ残さず、
信頼できる経路で相手へ送ります。受信者はブラウザーで開き、復号した内容を確認します。
有効期限・失効・一度限りの制限はありません。高価値の秘密への適合や独立監査は未確認です。
通常は既存のGitHub Pages受信ページを使い、その配信JavaScriptを信頼する構成です。
Pagesはmainから更新され得るため、CLIのタグで受信コードまで固定されるわけではありません。

## 配布候補の検証範囲

CIでは固定RustでUbuntu 24.04上のリリース版を生成し、展開後のCLIを別のUbuntu 24.04
コンテナで実行します。その環境にcargo/rustcがないことを確認します。
検証用のNode.jsで既存Web Crypto受信コードを呼び、架空入力の往復、上限、不正入力、
改ざんを確認します。Node.jsは検査用で、利用者のCLI実行には不要です。
この検査はブラウザーUI・クリップボード・実利用の継続率の証明ではありません。
