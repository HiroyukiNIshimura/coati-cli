# coati-cli
coatiの外部APIを利用するためのラッパーCLI

## Usage

```sh
cargo build --release

coati -b https://api.example.com get /users -q page=1
coati -b https://api.example.com -t $TOKEN post /users -d '{"name":"foo"}'
coati put https://api.example.com/users/1 -d @body.json
coati delete https://api.example.com/users/1 -i
```

Options: `-b/--base-url` (`COATI_BASE_URL`), `-t/--token` (`COATI_TOKEN`, Bearer), `-H`, `-q`, `-i`, `--raw`, `--timeout`.
Exit codes: 0 success, 1 HTTP error status, 2 request/usage error.
