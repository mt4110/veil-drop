# 0.1.1 Ubuntu 24.04 x86_64向け配布候補

この手順は未公開の0.1.1 GNU版候補用です。現在の公開v0.1.0はソース配布のみで、
ビルド済みZIPの公開は別途判断します。公開済みタグは移動しません。

対象はUbuntu 24.04 x86_64です。musl静的版ではなく、システムのglibc等を使います。
他ディストリビューション・旧glibc・ARM64への対応や性能は保証しません。
初回導線は`--no-clipboard`を使い、X11/Waylandのコピー動作は確認範囲外です。

## 取得済みZIPから使う

レビュー用成果物のZIPと外側のSHA256SUMSを、新しい同じディレクトリへ保存します。
Rust・Git・Node.jsは通常利用に不要です。展開にはunzipを使います。

```sh
sha256sum --ignore-missing -c SHA256SUMS &&
unzip veil-drop-0.1.1-x86_64-unknown-linux-gnu.zip -d cli &&
(cd cli && sha256sum -c SHA256SUMS) &&
./cli/veil-drop --version
```

ZIP名のOK、内部ファイルのOK、`veil-drop 0.1.1`を確認します。
同じ版番号の候補はBUILD.jsonのsource_revisionと外側のチェックサムで区別します。
旧候補 `ubuntu-gnu-candidate-41ef967` のZIPは内部版番号0.1.0で、この0.1.1候補とは別です。
公開前にマージ後の最終SHAから再生成・検証し、署名タグと公開を別途判断します。
チェックサムは一致確認であり、独立した発行者署名の代わりではありません。
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
