# coati-cli
coatiの外部APIを利用するためのラッパーCLI

## セットアップ

初回起動時、または `coati --init` を実行すると、API の URL と API キーの入力を求められ、
`~/.coati/config.toml` に保存されます（各 OS のユーザーホーム直下。Windows では
`C:\Users\<名前>\.coati`）。Unix 系ではファイルの権限を所有者のみに制限します。
API キーは平文で保存される点に注意してください。

## 使い方

```sh
cargo build --release

coati --init
coati item list <workspace> --page 1 --is-active true
coati item get <workspace> <itemNumber>
coati task list <workspace> <itemNumber> --is-completed false
coati task get <workspace> <itemNumber> <sequence>
coati task comments <workspace> <itemNumber> <sequence>
coati spec

# 任意のエンドポイントを直接呼ぶ
coati api get /api/external/workspaces/<workspace>/items -q page=1
coati api post /api/external/ping -d '{"message":"hi"}'
```

コマンドは `coati <カテゴリ> <操作>` の形で、`coati --help` / `coati <カテゴリ> --help` で確認できます。

オプション:

- `-b/--base-url`（環境変数 `COATI_BASE_URL`）、`-k/--api-key`（環境変数 `COATI_API_KEY`、`X-API-KEY` ヘッダで送信）: 保存済みの設定より優先されます。
- `-H`: ヘッダ追加（グローバル）、`api` コマンドのみ `-q`: クエリ追加、`-d`: ボディ（`@ファイル` / `@-` で標準入力）
- `-i`: ステータスとヘッダを表示、`--raw`: JSON を整形しない、`--timeout`: タイムアウト秒
- `--insecure`: TLS 証明書の検証を省略（ローカルの開発用証明書向け）

終了コード: 0 = 成功、1 = HTTP エラー、2 = リクエスト/引数エラー

## ビルドと配布

```sh
cargo build --release
```

配布するのは `target/release/coati`（Windows では `coati.exe`）の **1 ファイルのみ**です。
TLS は rustls を使っているため OpenSSL などの外部ライブラリは不要で、`docs/` や `target/` 内の他のファイルも不要です。

- 利用者は `coati` を PATH の通ったディレクトリ（例: `/usr/local/bin`、`~/.local/bin`）に置き、`coati --init` で URL と API キーを設定します。`~/.coati/config.toml` は同梱しないでください（API キーが含まれます）。
- バイナリはビルドした OS・CPU 専用です（macOS arm64 でビルドしたものは Windows / Linux では動きません）。他の OS 向けには、その OS 上でビルドしてください。
- macOS で、ダウンロードしたバイナリが Gatekeeper にブロックされる場合は、配布元とバイナリを信頼できることを確認したうえで、`xattr -d com.apple.quarantine coati` を実行できます（未署名のため）。
- 配布物の作成例（バージョンと CPU アーキテクチャは対象に合わせて変更してください）:

  ```sh
  version=0.1.0
  os=macos
  arch=arm64
  tar czf "coati-${version}-${os}-${arch}.tar.gz" -C target/release coati
  ```

  Windows では PowerShell で zip を作成できます:

  ```powershell
  $version = "0.1.0"
  $arch = "x86_64"
  Compress-Archive -Path target\release\coati.exe -DestinationPath "coati-$version-windows-$arch.zip"
  ```

## デバッグ

拡張機能 CodeLLDB（`vadimcn.vscode-lldb`）をインストールし、ブレークポイントを置いて F5 を押します。
`.vscode/launch.json` の構成は `--init`、`api get`（パスを入力）、任意の引数の 3 つです。
統合ターミナルで動くため、対話入力も使えます。

## API 仕様（エージェント向けナレッジ）

`coati spec` で `/api/external/openapi.json`（`X-API-KEY` 認証）を `docs/openapi.json` に保存します。
コーディングエージェントは [AGENTS.md](AGENTS.md) に従い、コマンドを実装する前にこのファイルを読みます。
