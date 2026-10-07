# coati-cli
coatiの外部APIを利用するためのラッパーCLI

## Setup

On first launch (or with `coati --init`) you are asked for the API URL and API key,
which are saved to `~/.coati/config.toml` (the user home directory on each OS, e.g.
`C:\Users\<name>\.coati` on Windows; file permissions are restricted on Unix).

## Usage

```sh
cargo build --release

coati --init
coati get /users -q page=1
coati post /users -d '{"name":"foo"}'
coati put https://api.example.com/users/1 -d @body.json
coati delete /users/1 -i
```

Options: `-b/--base-url` (`COATI_BASE_URL`) and `-k/--api-key` (`COATI_API_KEY`, sent as Bearer)
override the stored config, `-H`, `-q`, `-i`, `--raw`, `--timeout`.
Exit codes: 0 success, 1 HTTP error status, 2 request/usage error.

## Debugging

Install the CodeLLDB extension (`vadimcn.vscode-lldb`), set breakpoints, and press F5.
Launch configurations in `.vscode/launch.json`: `--init`, `get` (prompts for a path), and custom arguments.
They run in the integrated terminal so interactive prompts work.
