# coati-cli — エージェント向け指示

coati は Coati External API の REST クライアント CLI（Rust）です。

## API ナレッジ

- API 仕様は [docs/openapi.json](docs/openapi.json) です。API を呼ぶコマンドを**追加・変更する前に必ず読み**、エンドポイント・パラメータ・スキーマは仕様に従ってください。
- 認証は `X-API-KEY` ヘッダです。仕様の取得先は `GET /api/external/openapi.json` です。
- 仕様は `cargo run -- spec` で更新できます（ローカルの開発用証明書なら `--insecure` を付ける）。更新したらコミットしてください。

### `docs/openapi.json` が存在しない場合

1. まず `cargo run -- spec`（必要なら `--insecure`）を実行して取得します。
2. 設定（URL と API キー）が未初期化の場合、対話入力が必要なため、ユーザーに `coati --init` の実行を依頼してください。API キーをエージェントが尋ねて保存したり、リポジトリに書いたりしてはいけません。
3. 取得に失敗した場合（接続不可・認証エラー等）は、仕様を推測して実装せず、原因をユーザーに報告して指示を仰いでください。

## 規約

- API キーはリポジトリに含めません（`~/.coati/config.toml` に保存されます）。
- ビルドは `cargo build`。ソースは `src/` 配下です。
- ドキュメントとコミュニケーションは日本語で行います。
