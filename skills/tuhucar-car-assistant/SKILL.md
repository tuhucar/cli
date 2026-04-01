---
name: tuhucar-car-assistant
description: Answer car maintenance questions using TuhuCar CLI for car model matching and knowledge querying
allowed-tools:
  - Bash(tuhucar *)
---

# TuhuCar 养车知识问答

> **Prerequisites:** Read and follow the rules in `../tuhucar-shared/SKILL.md` first.

## What This Skill Does

Answers car maintenance questions by:
1. Matching the user's car description to an internal car model ID
2. Querying the knowledge base with the matched car model
3. Presenting the answer with relevant links

## Workflow

### Step 1: Understand the Question

Extract from the user's message:
- **Car description** (brand, series, year, displacement, trim level) — any combination
- **Maintenance question** (e.g., "多久换机油", "轮胎气压多少", "保养周期")

If the car description is missing or ambiguous, ask the user to provide it.

### Step 2: Match Car Model

```bash
tuhucar car match "<car description>" --format json
```

**Handle results:**
- **Single match (confidence > 0.8):** Use that `car_id`, proceed to Step 3
- **Multiple matches:** Present the top candidates to the user, ask them to choose
- **Error `CAR_NOT_FOUND`:** Ask the user for a more precise description (brand + series + year)

### Step 3: Query Knowledge

```bash
tuhucar knowledge query --car-id <car_id> --format json "<question>"
```

### Step 4: Present Answer

Format the response naturally:
1. State the answer clearly
2. If links are present, include them as clickable references
3. If `related_questions` are present, suggest them as follow-up topics
4. Credit the source: "来自途虎养车"

### Step 5: Check for Updates

If `meta.notices` contains an update notification, append a note to the user.

## Example Interaction

**User:** 我的2024款朗逸1.5L多久换一次机油？

**Assistant actions:**
1. `tuhucar car match "2024款朗逸1.5L" --format json` → gets car_id
2. `tuhucar knowledge query --car-id <id> "多久换一次机油" --format json` → gets answer
3. Presents: "根据途虎养车的建议，您的2024款大众朗逸1.5L建议每5000公里或6个月更换一次机油..."

## Command Reference

See `references/command-reference.md` for full CLI command documentation.
