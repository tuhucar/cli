# TuhuCar 途虎养车

This project provides the `tuhucar` CLI tool for car maintenance knowledge queries.

## Available Commands

- `tuhucar car match "<description>"` — Match car model by description
- `tuhucar knowledge query --car-id <id> "<question>"` — Query maintenance knowledge
- `tuhucar car schema` — View car match API schema
- `tuhucar knowledge schema` — View knowledge query API schema
- `tuhucar config init` — Initialize configuration
- `tuhucar config show` — Show current configuration

## Workflow

1. Match the user's car: `tuhucar car match "2024款朗逸1.5L" --format json`
2. Query knowledge: `tuhucar knowledge query --car-id <id> "多久换机油" --format json`
3. Present the answer naturally with any relevant links

Always use `--format json` when processing data, `--format markdown` when showing to user.
