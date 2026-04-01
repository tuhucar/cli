---
name: tuhucar-shared
description: Shared rules and conventions for all tuhucar skills
---

# TuhuCar Shared Rules

## Prerequisites

Before using any tuhucar command:
1. Verify `tuhucar` is installed: run `tuhucar --version`
2. If not installed, guide the user to install:
   - `npm install -g @tuhucar/cli`
   - Or: `curl -fsSL https://raw.githubusercontent.com/tuhucar/tuhucar/main/install.sh | sh`
3. Verify configuration exists: run `tuhucar config show`
4. If config missing, run `tuhucar config init`

## Output Format Conventions

- When calling tuhucar for data processing (feeding into your reasoning): use `--format json`
- When displaying results directly to the user: use `--format markdown` (default)
- Always parse JSON output programmatically, never show raw JSON to users

## Error Handling

All tuhucar commands return a unified JSON envelope when `--format json` is used:

```json
{
  "data": { ... },
  "error": { "code": "...", "message": "...", "retryable": true/false, "suggestion": "..." },
  "meta": { "version": "...", "notices": [...] }
}
```

Decision matrix based on `error.code`:

| error.code | retryable | Action |
|-----------|-----------|--------|
| (no error) | - | Use `data` normally |
| `CAR_NOT_FOUND` | false | Read `suggestion`, ask user for more precise description |
| `NETWORK_ERROR` | true | Retry once automatically |
| `API_ERROR` (5xx) | true | Tell user to try again later |
| `API_ERROR` (4xx) | false | Read `suggestion` for corrective action |
| `CONFIG_MISSING` | false | Guide user to run `tuhucar config init` |
| `INVALID_ARGS` | false | Read `suggestion` for correct usage, fix parameters |

## Update Notices

After each command, check `meta.notices` for update notifications:
- If `type: "update"` is present, inform the user about the available update
- Include the update command from the `message` field

## Safety Rules

- Never modify the user's tuhucar configuration without asking
- Always use `--dry-run` first if unsure about a command's behavior
- Do not cache or store car_id values across conversations — always re-match
