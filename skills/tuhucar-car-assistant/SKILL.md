---
name: tuhucar-knowledge-assistant
description: Answer car maintenance and ownership questions by calling the TuhuCar CLI knowledge query
allowed-tools:
  - Bash(tuhucar *)
---

# TuhuCar 养车知识问答

> **Prerequisites:** Read and follow the rules in `../tuhucar-shared/SKILL.md` first.

## What This Skill Does

Routes the user's car-ownership / maintenance question to the TuhuCar knowledge gateway via the `tuhucar` CLI and renders the reply.

The current build of `tuhucar` only exposes **knowledge query** as a business command. There is no separate "match car" step — pass the question (with any car context the user gave you) directly to `tuhucar knowledge query`.

## Workflow

### Step 1: Understand the Question

Treat the user's message as the `question`. Inline any car context the user mentioned (brand / series / year / 排量 / 配置) into the question string itself, e.g. `"2024款大众朗逸1.5L 自动挡 全合成机油多久换一次？"`. Don't strip it out — the upstream model uses it for personalised answers.

If the user only asks a generic question without car context, ask them once for **brand / series / year** so the answer can be tailored. If they decline, just proceed with the generic question.

### Step 2: Call the CLI

For multi-turn dialogs, reuse the `session_id` returned from the previous reply so the gateway keeps conversation state:

```bash
# First turn
question=$(cat <<'EOF'
<question>
EOF
)
tuhucar --format json knowledge query -- "$question"

# Follow-up turn (reuse the session_id from the previous response)
follow_up=$(cat <<'EOF'
<follow-up question>
EOF
)
tuhucar --format json knowledge query --session-id "$session_id" -- "$follow_up"
```

Always use `--format json` when you need to parse the result. Use the markdown default only when piping straight to the user without any post-processing.
Do not interpolate raw user input into a command string or ask a nested shell to execute it.

### Step 3: Parse the Response

Successful JSON envelope:

```json
{
  "data": {
    "reply": "...markdown answer from the gateway...",
    "session_id": "1743672000000",
    "msg_id": "1743672000000-1"
  },
  "error": null,
  "meta": { "version": "0.1.0", "notices": [] }
}
```

- `data.reply` is already markdown — present it (or paraphrase it) to the user.
- Cache `data.session_id` **for the duration of the current conversation only** so follow-up turns can pass `--session-id`. Never persist it across conversations.
- `msg_id` is for tracing; you usually don't need to surface it.

### Step 4: Present the Answer

1. Show `data.reply` to the user. It's already structured markdown — preserve headings, bullets and emojis.
2. End with the source attribution: **来自途虎养车**.
3. If `meta.notices` contains an update notice, append it after the answer.

### Step 5: Error Handling

See `../tuhucar-shared/SKILL.md` for the full decision matrix. Most common cases:

| `error.code` | What to do |
|---|---|
| `MCP_ERROR` (retryable) | Retry once. If it still fails, surface the message. |
| `MCP_ERROR` with `参数错误` | Re-check the question text — gateway may have rejected empty / malformed input. |
| `CONFIG_MISSING` | Run `tuhucar config init`, then retry. |
| `NETWORK_ERROR` | Retry once, then ask the user to try again. |

## Example Interaction

**User:** 我的2024款朗逸1.5L，全合成机油多久换一次？

**Assistant actions:**
1. Store `2024款大众朗逸1.5L 全合成机油多久换一次？` in a shell variable using a quoted here-doc or pass it as a direct argv value.
2. Run `tuhucar --format json knowledge query -- "$question"`.
3. Read `data.reply` from the JSON envelope
4. Remember `data.session_id` for the conversation
5. Present the markdown reply to the user, append "来自途虎养车"

**Follow-up — User:** 那刹车油呢？

**Assistant actions:**
1. Store `那刹车油多久换一次？` in a shell variable using a quoted here-doc or pass it as a direct argv value.
2. Run `tuhucar --format json knowledge query --session-id "$session_id" -- "$follow_up"`.
3. Present the new `data.reply`

## Command Reference

See `references/command-reference.md` for the full CLI surface.
