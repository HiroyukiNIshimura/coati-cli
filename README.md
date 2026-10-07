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

## デバッグ

拡張機能 CodeLLDB（`vadimcn.vscode-lldb`）をインストールし、ブレークポイントを置いて F5 を押します。
`.vscode/launch.json` の構成は `--init`、`get`（パスを入力）、任意の引数の 3 つです。
統合ターミナルで動くため、対話入力も使えます。

## API 仕様（エージェント向けナレッジ）

`coati spec` で `/api/external/openapi.json`（`X-API-KEY` 認証）を `docs/openapi.json` に保存します。
コーディングエージェントは [AGENTS.md](AGENTS.md) に従い、コマンドを実装する前にこのファイルを読みます。
