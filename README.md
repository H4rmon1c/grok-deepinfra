# Grok Build for OpenAI

An unofficial, OpenAI-first fork of the open-source Grok Build terminal coding
agent. It keeps Grok Build's full-screen TUI, file editing, shell tools,
streaming, reasoning, tool calls, subagents, MCP support, and headless mode,
while making OpenAI's Responses API the built-in default.

This project is not affiliated with or endorsed by OpenAI or SpaceXAI. It is
derived from the Apache-2.0 Grok Build source identified in [`SOURCE_REV`](SOURCE_REV).

## Quick Start

You need an **[OpenAI Platform API key](https://platform.openai.com/api-keys)**. A ChatGPT subscription, ChatGPT login,
browser session, or session token is not an API key, and API usage is billed
separately through the OpenAI Platform account.

```sh
git clone grok-build-openai.bundle grok-build-openai
cd grok-build-openai
export OPENAI_API_KEY="sk-..."
./grok-openai
```

Replace the bundle filename with this fork's Git URL when it is hosted online.
`./grok-openai` checks that the key is present, removes it while Cargo builds,
then restores it for the application runtime. Common agent-controlled terminal,
MCP, LSP, and hook subprocesses scrub accidental inheritance of the key. The
first build can take several minutes. The key is never included in this repository. The
launcher also uses `~/.grok-openai` as `GROK_HOME` by default, isolating this
fork from cached upstream xAI sessions and settings. Set `GROK_HOME` yourself
to override that location. Upstream binary auto-updates are disabled so they
cannot replace the fork; update it with Git and rebuild instead.

The inherited xAI image, video, and speech-to-text services are not sent an
OpenAI credential. Image/video tools are disabled for OpenAI model routes, and
the launcher disables voice dictation unless you explicitly set
`GROK_VOICE_MODE=1` and separately configure an xAI credential.

> [!IMPORTANT]
> Never commit an API key or put the literal key in repository files or shell
> startup files that the agent can read. Export it only for launch, or inject it
> with a password manager, CI secret store, or service environment. Child-env
> scrubbing prevents accidental inheritance; it is not an OS sandbox. A process
> running as your user may still read files that your user can read, so use a
> least-privilege key and the tool-permission/sandbox controls appropriate for
> your environment.

## Built-in OpenAI Models

| Picker entry | API model | Default use |
|---|---|---|
| GPT-5.6 | `gpt-5.6` | Primary coding model |
| GPT-5.6 Terra | `gpt-5.6-terra` | Balanced intelligence and cost |
| GPT-5.6 Luna | `gpt-5.6-luna` | Cost-sensitive work and session summaries |

All three profiles use:

- `https://api.openai.com/v1/responses`
- `Authorization: Bearer $OPENAI_API_KEY`
- streamed output, reasoning, and function/tool calls
- a 1,050,000-token context window and 128,000 maximum output tokens
- local Grok Build tools; the xAI-only hosted `x_search` extension is disabled

The limits and reasoning options above follow OpenAI's
[GPT-5.6 model documentation](https://developers.openai.com/api/docs/models/gpt-5.6-sol).

GPT-5.6 is selected by default. Use `/model` in the TUI or `--model` on the
command line to switch profiles.

## Requirements

- **Rust** — pinned by [`rust-toolchain.toml`](rust-toolchain.toml); `rustup`
  installs it automatically.
- **[DotSlash](https://dotslash-cli.com)** — used by the hermetic tools in
  [`bin/`](bin/), including `protoc`.
- **protoc** — resolved through [`bin/protoc`](bin/protoc), `$PROTOC`, or your
  `PATH`.

Install DotSlash before the first build:

```sh
cargo install dotslash
/usr/bin/env dotslash --help
```

macOS and Linux are supported build hosts. Windows builds are best-effort.

## Build and Run Manually

```sh
cargo build --locked -p xai-grok-pager-bin --release
export OPENAI_API_KEY="sk-..."
GROK_DISABLE_AUTOUPDATER=1 ./target/release/xai-grok-pager
```

The inherited Cargo artifact remains named `xai-grok-pager`; the
`grok-openai` launcher provides the fork-friendly command name without
rewriting upstream package identities.

## Headless Use

```sh
export OPENAI_API_KEY="sk-..."
./grok-openai -p "explain this repository"
```

Run `./grok-openai --help` for the complete CLI surface.

## Configuration and Other Providers

The built-in OpenAI models require no `config.toml`. Custom OpenAI models,
OpenAI-compatible servers, local models, Anthropic, and the inherited xAI
provider can still be configured in `$GROK_HOME/config.toml`. See the
[custom-model guide](crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md).

Upstream xAI model/settings discovery is disabled by default so an old cached
xAI login cannot replace the bundled OpenAI catalog. A deployment that
intentionally needs remote discovery can opt in:

```toml
[features]
remote_fetch = true
```

Provider credentials are fail-closed: when a model declares `env_key` and that
variable is missing, the app will not substitute a cached xAI session or global
key. Startup instead tells you which provider key to configure.

The full user guide lives in
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/).

## Why the Fork Is Small

The supplied Grok Build source already implements OpenAI-compatible Chat
Completions and the Responses API end to end. The fork makes that existing path
first-class by bundling OpenAI model metadata, routing each model directly to
OpenAI, selecting `OPENAI_API_KEY`, disabling xAI-only request extensions, and
fixing missing-key fallback behavior. No proxy and no credential translation
service are involved.

## Repository Layout

| Path | Contents |
|---|---|
| `crates/codegen/xai-grok-pager-bin` | Composition root and executable |
| `crates/codegen/xai-grok-pager` | TUI, views, commands, and embedded docs |
| `crates/codegen/xai-grok-shell` | Agent runtime, configuration, and auth |
| `crates/codegen/xai-grok-sampler` | HTTP/SSE model transport |
| `crates/codegen/xai-grok-sampling-types` | Responses request/response conversion |
| `crates/codegen/xai-grok-tools` | Terminal, file, search, and other tools |
| `crates/codegen/xai-grok-workspace` | Filesystem, VCS, execution, checkpoints |

The root `Cargo.toml` is generated upstream; prefer editing per-crate manifests.

## Development

```sh
cargo check -p xai-grok-models
cargo test -p xai-grok-shell bundled_default_is_openai_responses_byok_profile
cargo test -p xai-grok-sampler stream::responses
cargo check -p xai-grok-pager-bin
cargo fmt --all -- --check
```

## Upstream Provenance and License

First-party source is licensed under the **Apache License, Version 2.0**; see
[`LICENSE`](LICENSE). Third-party and vendored code remains under its original
licenses:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
- [`third_party/NOTICE`](third_party/NOTICE)

The original Grok Build project and branding belong to their respective owner.
