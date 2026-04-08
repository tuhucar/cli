# TuhuCar CLI Command Reference

The current build only exposes one business command (`knowledge query`) plus local utility commands (`config`, `skill`).

## Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--format json\|markdown` | Output format | `markdown` |
| `--dry-run` | Preview the upstream tool call without sending it | off |
| `--verbose` | Verbose output | off |
| `--version` | Show version | - |
| `--help` | Show help | - |

## Environment

| Variable | Effect |
|---|---|
| `TUHUCAR_ENDPOINT` | Override the MCP gateway endpoint at runtime (takes precedence over `~/.tuhucar/config.toml`). Useful for pointing at the test gateway without editing config. |

## `tuhucar knowledge query <question> [--session-id <id>]`

Send a maintenance / ownership question to the TuhuCar knowledge gateway.

**Arguments:**
- `question` (positional, required) — natural-language question. Inline car context (brand / series / year / 排量 / 配置) into this string when known.
- `--session-id <id>` (optional) — reuse a session id from a previous reply to continue a multi-turn dialog. If omitted, a new session is created automatically.

**JSON envelope (success):**

```json
{
  "data": {
    "reply": "...markdown answer...",
    "session_id": "1743672000000",
    "msg_id": "1743672000000-1"
  },
  "error": null,
  "meta": { "version": "0.1.0", "notices": [] }
}
```

**Possible errors:**
- `MCP_ERROR` — gateway rejected the call or returned a non-success code. Body holds the upstream message.
- `CONFIG_MISSING` — no `~/.tuhucar/config.toml`; run `tuhucar config init` (or set `TUHUCAR_ENDPOINT`).
- `NETWORK_ERROR` — transport-level failure (timeout, DNS, etc.).

## `tuhucar knowledge schema`

Print the JSON Schema for the knowledge query input/output and the wire envelope. Useful for LLM self-discovery. Does not require config.

```bash
tuhucar knowledge schema
```

## `tuhucar config init`

Create the default configuration at `~/.tuhucar/config.toml`. Endpoint defaults to the production MCP gateway; override with `TUHUCAR_ENDPOINT` or by editing the file directly.

## `tuhucar config show`

Print the current configuration.

## `tuhucar skill install` / `tuhucar skill uninstall`

Detect installed AI platforms (Claude Code, Cursor, Codex, OpenCode, Gemini) and register / unregister the tuhucar skill files. Local-only operation, no network.

## Notes for LLM callers

- Always pass `--format json` when you need to parse the result programmatically.
- The `reply` field is already markdown — render it as-is when surfacing the answer.
- `session_id` is conversation-scoped state. Reuse it for follow-up turns in the same conversation, then discard.
- This build does **not** include any car-match command. Don't try `tuhucar car ...` — it doesn't exist in the released binary.
