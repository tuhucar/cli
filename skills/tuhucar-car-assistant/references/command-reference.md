# TuhuCar CLI Command Reference

## Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--format json\|markdown` | Output format | `markdown` |
| `--dry-run` | Preview request without sending | off |
| `--verbose` | Detailed output | off |
| `--version` | Show version | - |
| `--help` | Show help | - |

## Commands

### `tuhucar car match <query>`

Match a car description to internal car model IDs.

**Arguments:**
- `query` (required): Car description, e.g. "2024款朗逸1.5L自动舒适版"

**JSON Output (success):**
```json
{
  "data": {
    "candidates": [
      {
        "car_id": "12345",
        "brand": "大众",
        "series": "朗逸",
        "year": "2024",
        "displacement": "1.5L",
        "model": "自动舒适版",
        "confidence": 0.95
      }
    ],
    "total_count": 1
  }
}
```

**Possible errors:** `CAR_NOT_FOUND`, `NETWORK_ERROR`, `CONFIG_MISSING`

### `tuhucar car schema`

Output the car match command's JSON Schema for LLM introspection. Does not require configuration.

### `tuhucar knowledge query --car-id <id> <question>`

Query car maintenance knowledge for a specific car model.

**Arguments:**
- `--car-id` (required): Five-level car model ID from `car match`
- `question` (required): Maintenance question

**JSON Output (success):**
```json
{
  "data": {
    "answer": "建议每5000公里或6个月更换一次机油...",
    "links": [
      {
        "title": "预约保养",
        "url": "https://m.tuhu.cn/maintenance",
        "link_type": "H5"
      }
    ],
    "related_questions": [
      "机油品牌推荐",
      "保养费用参考"
    ]
  }
}
```

**Possible errors:** `NETWORK_ERROR`, `API_ERROR`, `CONFIG_MISSING`

### `tuhucar knowledge schema`

Output the knowledge query command's JSON Schema. Does not require configuration.

### `tuhucar config init`

Create default configuration at `~/.tuhucar/config.toml`.

### `tuhucar config show`

Display current configuration.

### `tuhucar skill install`

Detect installed AI platforms and register tuhucar skills.

### `tuhucar skill uninstall`

Remove tuhucar skills from AI platforms.
