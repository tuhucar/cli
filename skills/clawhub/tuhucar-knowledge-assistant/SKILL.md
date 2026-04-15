---
name: tuhucar-knowledge-assistant
description: Use when answering car maintenance, service interval, oil, brake fluid, tire pressure, or ownership questions through the TuhuCar CLI knowledge gateway.
homepage: https://github.com/tuhucar/cli
metadata: {"openclaw":{"requires":{"bins":["tuhucar"]}}}
---

# TuhuCar Knowledge Assistant

Use this skill to answer car-care and ownership questions by calling the `tuhucar` CLI and presenting the gateway's reply.

## Prerequisites

Before using any `tuhucar` command:

1. Verify the CLI is installed: `tuhucar --version`
2. If it is missing, guide the user to install it:
   - `npm install -g @tuhucar/cli`
   - `curl -fsSL https://raw.githubusercontent.com/tuhucar/cli/main/scripts/install.sh | sh`
3. Verify configuration: `tuhucar config show`
4. If config is missing, run `tuhucar config init` or set `TUHUCAR_ENDPOINT`

## Workflow

### Step 1: Build the question

Treat the user's message as the `question`. Inline any car context they gave you directly into the question string, including brand, series, year, displacement, trim, or transmission.

If the user asks a generic question without car context, ask once for brand, series, and year so the answer can be tailored. If they decline, continue with the generic question.

### Step 2: Call the CLI

Use `--format json` whenever you need to parse the response:

```bash
# First turn
tuhucar knowledge query --format json "<question>"

# Follow-up turn in the same conversation
tuhucar knowledge query --format json --session-id <session_id> "<follow-up question>"
```

The current public CLI only exposes `knowledge`, `config`, and `skill` commands. Do not invent a `car` command or a separate car-match step.

### Step 3: Parse the JSON envelope

Every JSON response uses this envelope:

```json
{
  "data": { ... },
  "error": { "code": "...", "message": "...", "retryable": true, "suggestion": "..." },
  "meta": { "version": "0.1.0", "notices": [] }
}
```

Exactly one of `data` or `error` is populated.

On success, use `data.reply` as the answer body. It is already markdown.

`data.session_id` is conversation-scoped. Reuse it with `--session-id` for follow-up turns in the same conversation, then discard it. Do not persist it across conversations.

### Step 4: Present the answer

1. Show `data.reply` to the user and preserve its markdown structure.
2. End with `来自途虎养车`.
3. If `meta.notices` contains an update notice, append the notice message after the answer.

## Error Handling

| `error.code` | Retryable | Action |
|---|---|---|
| `MCP_ERROR` | usually | Retry once. If it still fails, surface `error.message`. |
| `NETWORK_ERROR` | yes | Retry once, then ask the user to try again. |
| `CONFIG_MISSING` | no | Run `tuhucar config init` or set `TUHUCAR_ENDPOINT`. |
| `INVALID_ARGS` | no | Read `error.suggestion`, fix the command shape, and retry. |
| `API_ERROR` with 5xx semantics | yes | Ask the user to try again later. |
| `API_ERROR` with 4xx semantics | no | Use `error.suggestion` to correct the request. |

If you are unsure what a command will do, run it with `--dry-run` first.

## Output Conventions

- Use `--format json` for programmatic parsing.
- Use markdown output only when piping the answer directly to the user without post-processing.
- Never show the raw JSON envelope to the user. Extract `data.*` first.
- Never modify `~/.tuhucar/config.toml` without the user's approval.

## Example

**User:** 我的2024款朗逸1.5L，全合成机油多久换一次？

**Assistant actions:**
1. Run `tuhucar knowledge query --format json "2024款大众朗逸1.5L 全合成机油多久换一次？"`
2. Read `data.reply`
3. Remember `data.session_id` for this conversation
4. Present the markdown reply and append `来自途虎养车`

## Command Reference

See `{baseDir}/references/command-reference.md` for the full CLI surface.
