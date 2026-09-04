# Grok Build for DeepInfra

An unofficial DeepInfra-oriented distribution of the open-source Grok Build terminal coding agent. It keeps the full-screen TUI, file editing, shell tools, streaming, tool calls, subagents, MCP support, checkpoints, and headless mode while running open-weight models through DeepInfra’s OpenAI-compatible API.

This repository is derived from [`H4rmon1c/grok-build-openai`](https://github.com/H4rmon1c/grok-build-openai), which in turn imports the Apache-2.0 Grok Build source identified in [`SOURCE_REV`](SOURCE_REV). This is a standalone derivative repository rather than a GitHub-network fork.

It is not affiliated with or endorsed by DeepInfra, OpenAI, or xAI.

## What This Repository Changes

- Preserves the OpenAI-compatible Chat Completions and Responses transports already implemented by Grok Build.
- Adds narrow handling for provider SSE keepalive frames before typed Responses-event deserialization.
- Supports DeepInfra through a normal custom-model configuration; no proxy or credential translation service is required.
- Keeps credentials outside the repository and resolves them from `DEEPINFRA_API_TOKEN` at runtime.
- Disables upstream binary auto-updates in the recommended launcher so an upstream binary cannot silently replace this build.

> [!NOTE]
> The inherited bundled model catalog is still OpenAI-oriented. A DeepInfra model must be selected in `config.toml` as shown below. This repository does not contain or hardcode a DeepInfra token.

## Requirements

- Linux or macOS
- Git and a C/C++ build toolchain
- Rust 1.94.0 through `rustup` (pinned by [`rust-toolchain.toml`](rust-toolchain.toml))
- [DotSlash](https://dotslash-cli.com), used by hermetic tools in [`bin/`](bin/)
- A [DeepInfra API token](https://deepinfra.com/dash/api_keys)

## Build

```sh
git clone https://github.com/H4rmon1c/grok-deepinfra.git
cd grok-deepinfra

curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
cargo install dotslash --locked
cargo build --locked --release -p xai-grok-pager-bin
```

The built executable is `target/release/xai-grok-pager`.

## Configure DeepInfra

The example below uses [Qwen3 Coder 480B A35B Instruct Turbo](https://deepinfra.com/Qwen/Qwen3-Coder-480B-A35B-Instruct-Turbo), a 262,144-token coding model with streaming and function-call support.

Create isolated configuration and install the binary:

```sh
install -d -m 700 "$HOME/.config/grok-deepinfra" "$HOME/.grok-deepinfra"
install -d "$HOME/.local/bin" "$HOME/.local/libexec/grok-deepinfra"
install -m 755 target/release/xai-grok-pager "$HOME/.local/libexec/grok-deepinfra/grok"

install -m 600 /dev/null "$HOME/.grok-deepinfra/config.toml"
printf "%s\n" \
  "[models]" \
  "default = \"qwen3-coder\"" \
  "" \
  "[model.qwen3-coder]" \
  "model = \"Qwen/Qwen3-Coder-480B-A35B-Instruct-Turbo\"" \
  "base_url = \"https://api.deepinfra.com/v1/openai\"" \
  "name = \"Qwen3 Coder 480B via DeepInfra\"" \
  "env_key = \"DEEPINFRA_API_TOKEN\"" \
  "api_backend = \"chat_completions\"" \
  "context_window = 262144" \
  "" \
  "[cli]" \
  "auto_update = false" \
  > "$HOME/.grok-deepinfra/config.toml"
```

Store the token without putting it in shell history:

```sh
umask 077
read -rsp "DeepInfra API token: " DEEPINFRA_TOKEN_INPUT
printf "\n"
printf "export DEEPINFRA_API_TOKEN=%q\n" "$DEEPINFRA_TOKEN_INPUT" \
  > "$HOME/.config/grok-deepinfra/env"
unset DEEPINFRA_TOKEN_INPUT
```

Create the launcher:

```sh
install -m 755 /dev/null "$HOME/.local/bin/grok-deepinfra"
printf "%s\n" \
  "#!/usr/bin/env bash" \
  "set -euo pipefail" \
  "set +x" \
  "export GROK_HOME=\"${GROK_HOME:-$HOME/.grok-deepinfra}\"" \
  "source \"$HOME/.config/grok-deepinfra/env\"" \
  ": \"${DEEPINFRA_API_TOKEN:?DeepInfra API token is not configured}\"" \
  "unset GROK_MODELS_BASE_URL GROK_MODELS_LIST_URL" \
  "export GROK_DISABLE_AUTOUPDATER=1" \
  "exec \"$HOME/.local/libexec/grok-deepinfra/grok\" \"$@\"" \
  > "$HOME/.local/bin/grok-deepinfra"
```

Start the agent from a project directory:

```sh
grok-deepinfra
```

For a long-running mobile SSH session, use `tmux`:

```sh
tmux new -As vibe
grok-deepinfra
```

## Choosing Another DeepInfra Model

DeepInfra exposes its LLM catalog through the OpenAI-compatible base URL:

```text
https://api.deepinfra.com/v1/openai
```

Change the `model`, display `name`, context window, and `[models].default` entry in `~/.grok-deepinfra/config.toml`. DeepInfra model identifiers retain their vendor prefix, such as `Qwen/...`, `deepseek-ai/...`, or `moonshotai/...`.

DeepInfra officially documents Chat Completions as the broadly supported LLM interface, so the recommended profile uses:

```toml
api_backend = "chat_completions"
```

See:

- [DeepInfra Chat Completions](https://docs.deepinfra.com/chat/overview)
- [DeepInfra model catalog](https://docs.deepinfra.com/models)
- [Grok Build custom-model guide](crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md)

## Credential Safety

Never commit an API token or place a literal token in repository files. The recommended layout keeps it in `~/.config/grok-deepinfra/env` with mode `0600`, while model configuration contains only the environment-variable name.

This reduces accidental disclosure; it is not an OS security boundary. Run untrusted agent-generated code in a dedicated VM or similarly isolated environment and use a least-privilege token with an appropriate spending limit.

## Development

```sh
cargo test -p xai-grok-sampler response_keepalive_frames_are_swallowed_before_typed_deserialization
cargo check -p xai-grok-pager-bin
cargo fmt --all -- --check
```

## Repository Layout

| Path | Contents |
|---|---|
| `crates/codegen/xai-grok-pager-bin` | Composition root and executable |
| `crates/codegen/xai-grok-pager` | TUI, views, commands, and embedded documentation |
| `crates/codegen/xai-grok-shell` | Agent runtime, configuration, and authentication |
| `crates/codegen/xai-grok-sampler` | HTTP and SSE model transport |
| `crates/codegen/xai-grok-sampling-types` | Request and response conversion |
| `crates/codegen/xai-grok-tools` | Terminal, file, search, and other agent tools |
| `crates/codegen/xai-grok-workspace` | Filesystem, VCS, execution, and checkpoints |

## Provenance and License

First-party source is licensed under the **Apache License, Version 2.0**; see [`LICENSE`](LICENSE). Third-party and vendored code remains under its original licenses:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
- [`third_party/NOTICE`](third_party/NOTICE)

The original Grok Build project and branding belong to their respective owners.
