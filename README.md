# TuhuCar CLI

[![CI](https://github.com/tuhucar/cli/actions/workflows/ci.yml/badge.svg)](https://github.com/tuhucar/cli/actions/workflows/ci.yml)
[![npm version](https://img.shields.io/npm/v/%40tuhucar%2Fcli)](https://www.npmjs.com/package/@tuhucar/cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

TuhuCar CLI is a production-oriented command-line client for Tuhu car-care knowledge workflows.
It provides a consistent interface for querying maintenance guidance, exposing machine-readable schemas for LLM tooling, and installing ready-to-use skills into popular AI coding assistants.

## Why TuhuCar CLI

- Purpose-built for car-care knowledge queries instead of generic chat wrappers
- Stable CLI surface with both `markdown` and JSON envelope output modes
- MCP-based upstream integration with schema introspection for agent workflows
- Built-in skill installation for Claude Code, Cursor, Codex, OpenCode, and Gemini CLI
- Multi-platform distribution via npm, release binaries, and shell installer

## Features

- `knowledge query` for maintenance and ownership questions
- `knowledge schema` for LLM introspection and tool wiring
- `config init` and `config show` for local runtime configuration
- `skill install` and `skill uninstall` for assistant integration
- `--dry-run` support for debugging and prompt/tool development
- Update notices for installed distributions

## Installation

### npm

```bash
npm install -g @tuhucar/cli
```

### Shell Installer

```bash
curl -fsSLO https://raw.githubusercontent.com/tuhucar/cli/main/scripts/install.sh
less install.sh
sh install.sh
```

### Homebrew

```bash
brew install tuhucar/tap/tuhucar
```

## Quick Start

```bash
# 1. Create default config
tuhucar config init

# 2. Ask a maintenance question
tuhucar knowledge query "2024款大众朗逸1.5L 全合成机油多久换一次？"

# 3. Continue a multi-turn session
tuhucar knowledge query --session-id <session_id> "那刹车油多久换一次？"

# 4. Inspect the machine-readable command schema
tuhucar knowledge schema
```

Configuration is stored at `~/.tuhucar/config.toml`.
If you need to point the CLI at another gateway temporarily, set `TUHUCAR_ENDPOINT` in the environment.

## Command Overview

### Global Options

| Option | Description | Default |
| --- | --- | --- |
| `--format json\|markdown` | Output mode for command results | `markdown` |
| `--dry-run` | Preview the outbound MCP tool call without sending it | `false` |
| `--verbose` | Enable more detailed runtime output | `false` |
| `--version` | Print version information | - |
| `--help` | Print help information | - |

### Knowledge Query

```bash
# Human-friendly output
tuhucar knowledge query "机油多久换一次"

# JSON envelope output
tuhucar --format json knowledge query "机油多久换一次"

# Multi-turn dialog
tuhucar knowledge query --session-id <session_id> "轮胎气压怎么判断？"

# Tool schema for agents
tuhucar knowledge schema
```

### Configuration

```bash
tuhucar config init
tuhucar config show
```

### Assistant Skills

```bash
tuhucar skill install
tuhucar skill uninstall
```

## AI Assistant Integration

TuhuCar CLI ships with embedded skill assets and platform-specific installation flows.

| Platform | Support |
| --- | --- |
| Claude Code | Yes |
| Cursor | Yes |
| Codex | Yes |
| OpenCode | Yes |
| Gemini CLI | Yes |

After installing the CLI, run `tuhucar skill install` to register the bundled skills on detected platforms.

## ClawHub Publishing

The ClawHub-ready publish source lives in `skills/clawhub/tuhucar-knowledge-assistant/`.
It is intentionally self-contained and separate from the bundled local assistant skills used by `tuhucar skill install`.

Example publish flow:

```bash
clawhub login

clawhub publish ./skills/clawhub/tuhucar-knowledge-assistant \
  --slug tuhucar-knowledge-assistant \
  --name "TuhuCar Knowledge Assistant" \
  --version 0.0.3 \
  --tags latest \
  --changelog "Address ClawHub security scan findings"
```

If you update the CLI behavior that the skill depends on, keep these files aligned:

- `skills/tuhucar-*` for bundled local assistant installs
- `skills/clawhub/tuhucar-knowledge-assistant/` for ClawHub publishing

## Architecture

The project is organized as a small Rust workspace with clear module boundaries.

| Path | Responsibility |
| --- | --- |
| `crates/tuhucar-core` | Shared config, MCP transport, error model, output envelope, update logic |
| `crates/tuhucar-knowledge` | Knowledge query models and command implementation |
| `crates/tuhucar-car` | Car matching support library retained in workspace modules |
| `crates/tuhucar-cli` | User-facing CLI binary and command dispatch |
| `skills/` | Assistant skill definitions and references |
| `skills/clawhub/` | Self-contained ClawHub publishable skill bundles |
| `npm/` | npm distribution package |
| `scripts/` | Installer and release support scripts |

## Development

### Prerequisites

- Rust stable
- Node.js 14+ for npm packaging workflows

### Common Commands

```bash
# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy with CI-level strictness
cargo clippy --workspace --all-targets -- -D warnings

# Verify formatting
cargo fmt --all -- --check

# Run the CLI locally
cargo run -p tuhucar-cli -- --help
```

## Release Model

- GitHub Releases publish platform binaries
- npm publishes `@tuhucar/cli`
- The shell installer downloads release artifacts directly from GitHub Releases

## Contributing

Issues and pull requests are welcome.
When changing CLI behavior, keep the README examples, integration tests, and schema-facing output aligned.

## License

MIT. See [LICENSE](./LICENSE).
