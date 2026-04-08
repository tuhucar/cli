---
name: tuhucar-shared
description: Shared rules and conventions for all tuhucar skills
---

# TuhuCar Shared Rules

## Prerequisites

Before using any tuhucar command:

1. Verify `tuhucar` is installed: `tuhucar --version`
2. If not installed, guide the user to install:
   - `npm install -g @tuhucar/cli`
   - Or: `curl -fsSL https://raw.githubusercontent.com/tuhucar/tuhucar/main/install.sh | sh`
3. Verify configuration: `tuhucar config show`
4. If config is missing, run `tuhucar config init` — **or** set `TUHUCAR_ENDPOINT` in the environment to skip the file (useful for ad-hoc / dev gateways).

## Output Format Conventions

- When you need to parse the result programmatically: pass `--format json`.
- When piping the result straight to the user: use the markdown default.
- Never show the raw JSON envelope to the user — always extract `data.*` first.

## Unified JSON Envelope

Every command (with `--format json`) returns:

```json
{
  "data":  { ... },
  "error": { "code": "...", "message": "...", "retryable": true, "suggestion": "..." },
  "meta":  { "version": "0.1.0", "notices": [...] }
}
```

Exactly one of `data` / `error` is populated.

## Error Decision Matrix

| `error.code` | retryable | Action |
|---|---|---|
| (none) | — | Use `data` normally |
| `MCP_ERROR` | usually true | Retry once. If still failing, surface the upstream message. The gateway puts its own message inside `error.message` (e.g. `参数错误`). |
| `NETWORK_ERROR` | true | Retry once automatically, then ask user to retry. |
| `CONFIG_MISSING` | false | Run `tuhucar config init`, or set `TUHUCAR_ENDPOINT`. |
| `INVALID_ARGS` | false | Read `error.suggestion`, fix arguments, retry. |
| `API_ERROR` (5xx) | true | Tell the user to try again later. |
| `API_ERROR` (4xx) | false | Read `error.suggestion` for corrective action. |

## Update Notices

After each command, check `meta.notices`. If a `type: "update"` notice is present, append the `message` to your reply so the user sees the upgrade hint.

## Conversation State

- `data.session_id` (returned by `knowledge query`) is **conversation-scoped**. Reuse it via `--session-id` for follow-up turns in the *same* conversation, then discard. Do not persist across conversations.
- Do not cache any other ids across conversations.

## Safety Rules

- Never modify `~/.tuhucar/config.toml` without asking the user.
- If you are unsure what a command will do, run it with `--dry-run` first — it prints the upstream MCP tool call instead of sending it.
- This build only exposes `knowledge`, `config`, and `skill` subcommands. There is no `car` command — don't invent one.
