# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test --workspace

# Lint & Format
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Run locally
cargo run -p tuhucar-cli -- <subcommand>

# Run a single test
cargo test -p <crate-name> <test_name>
```

CI enforces `cargo fmt` and `cargo clippy -D warnings` (all warnings are errors).

## Architecture

Rust CLI tool (edition 2021) with a Cargo workspace of 4 crates:

```
tuhucar-cli  →  tuhucar-car  →  tuhucar-core
             →  tuhucar-knowledge  →  tuhucar-core
```

- **tuhucar-core**: Shared foundation — `Command` trait, `McpClient` (MCP JSON-RPC over Streamable HTTP), `Config` (TOML at `~/.tuhucar/config.toml`), `Response<T>` envelope, `ApiError`, `Render` trait, `OutputFormat`.
- **tuhucar-car**: Car model matching. Implements `Command` trait via `CarCommand`.
- **tuhucar-knowledge**: Knowledge querying. Implements `Command` trait via `KnowledgeCommand`.
- **tuhucar-cli**: Binary entry point. Clap-derived CLI with subcommands: `car`, `knowledge`, `config`, `skill`. Global flags: `--format`, `--dry-run`, `--verbose`.

### Key Patterns

- **Command trait**: All commands define `Input`/`Output` types (both `JsonSchema`), implement `execute()` and get `schema()` for free — enables LLM self-discovery.
- **Dual output**: Types implement `Render` trait providing both `to_json()` and `to_markdown()`. JSON for programmatic/LLM use, Markdown for humans.
- **MCP Client**: `McpClient` connects to upstream MCP server via Streamable HTTP. Implements `initialize` → `notifications/initialized` → `tools/call` flow. Upstream tools: `car_match`, `knowledge_query`.
- **Error envelope**: All errors become `ApiError` with `code`, `message`, `retryable`, `suggestion` fields. `Response<T>` skips absent fields via `skip_serializing_if`.
- **Skill system**: `skill install/uninstall` detects AI platforms (Claude Code, Cursor, Codex, OpenCode, Gemini) and symlinks skill definitions from `skills/` directory.

### CLI Command Tree

```
tuhucar car match <query>           # Match car model
tuhucar car schema                  # JSON schema for LLM
tuhucar knowledge query --car-id <id> <question>
tuhucar knowledge schema
tuhucar config init|show
tuhucar skill install|uninstall
```

## Distribution

- npm wrapper package in `npm/` — postinstall downloads the platform binary
- One-line installer in `scripts/install.sh`
- Release workflow builds 8 targets (macOS/Linux/Windows × architectures)
