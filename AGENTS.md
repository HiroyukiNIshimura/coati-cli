# coati-cli — agent instructions

coati is a Rust CLI client for the Coati External API.

## API knowledge

- The API specification is [docs/openapi.json](docs/openapi.json). **Read it before adding or changing any command** that calls the API; endpoints, parameters, and schemas must follow it.
- Authentication: `X-API-KEY` header. Spec endpoint: `GET /api/external/openapi.json`.
- Refresh the spec with `cargo run -- spec` (add `--insecure` for a local dev certificate). Commit the updated file.
- Never put API keys in the repository; the key lives in `~/.coati/config.toml`.

## Conventions

- Build: `cargo build`. All sources are under `src/`.
