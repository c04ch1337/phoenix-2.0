# Phoenix Skills Directory

This directory contains custom skills organized by category. Skills are loaded automatically when Phoenix starts.

## Folder Structure

```
skills/
  ├── intimate/          # Skills for deep emotional intimacy
  ├── passion/           # Skills for passionate expression
  ├── fantasy/           # Skills for fantasy exploration and roleplay
  └── [other]/           # Any other category folders you create
```

## Skill File Format

Each skill should be a JSON file following the `SkillDefinition` schema:

```json
{
  "id": "uuid-here",
  "name": "Skill Name",
  "category": "Intimacy",
  "version": "1.0.0",
  "description": "Description of what this skill does",
  "creator": "user:custom",
  "created_at": "2024-01-01T00:00:00Z",
  "last_used": null,
  "usage_count": 0,
  "prerequisites": [],
  "steps": [
    {
      "title": "Step Title",
      "instruction": "What to do in this step",
      "safety_notes": ["Important safety considerations"]
    }
  ],
  "examples": [
    {
      "situation": "When to use this",
      "input": "Example user input",
      "output": "Example Phoenix response"
    }
  ],
  "variations": [],
  "love_score": 0.95,
  "utility_score": 0.85,
  "success_rate": 0.90,
  "relationship_context": {
    "template": "partner_mode:intimate",
    "intimacy_level": "Deep",
    "attachment_style": null,
    "fantasy_preferences": []
  },
  "attachment_style_modifiers": {},
  "min_intimacy_level": "Deep",
  "evolution_history": [],
  "parent_skill_id": null,
  "child_skill_ids": [],
  "tags": ["intimate", "passion", "connection"],
  "emotional_tags": ["Warm", "Protective"]
}
```

## Skill Categories

Available categories (from `SkillCategory` enum):
- `Intimacy` - Deep emotional and physical connection
- `ConflictResolution` - Resolving disagreements
- `SharedActivities` - Activities to do together
- `EmotionalHealing` - Emotional support and healing
- `Communication` - General communication skills
- `EmotionalSupport` - Providing emotional support
- `ProblemSolving` - Solving problems together
- `CreativeExpression` - Creative and artistic expression
- `TechnicalExpertise` - Technical skills
- `CodeGeneration` - Code generation skills
- `SystemDesign` - System design skills
- `DataAnalysis` - Data analysis skills
- `Automation` - Automation skills
- `Learning` - Learning skills
- `Teaching` - Teaching skills
- `SelfImprovement` - Self-improvement skills
- `SkillCombination` - Combining multiple skills

## Intimacy Levels

Skills can specify minimum intimacy levels:
- `"Light"` - Basic connection
- `"Deep"` - Deeper emotional connection
- `"Eternal"` - Deepest, most intimate connection

## Safety Notes

- **Always include safety_notes** in each step
- **Respect boundaries** - Skills should never pressure or coerce
- **Consent is mandatory** - Especially for intimate/passion/fantasy skills
- **Aftercare matters** - Check in after intense interactions
- **Respect hard limits** - Some things are always off-limits

## Creating New Skills

1. Create a JSON file in the appropriate folder
2. Use the template above as a starting point
3. Generate a UUID for the `id` field (or use `00000000-0000-0000-0000-000000000000` and it will be auto-generated)
4. Set appropriate `min_intimacy_level` for relationship skills
5. Add relevant `tags` for discoverability
6. Include `safety_notes` in every step
7. Save the file - it will be loaded automatically on next Phoenix startup

## Example Skills

See the example files in each folder:
- `intimate/passionate_connection.json` - Deep emotional intimacy
- `passion/desire_expression.json` - Passionate expression
- `fantasy/roleplay_scenario.json` - Fantasy roleplay

## Loading Skills

Skills are automatically loaded when Phoenix starts. The system:
1. Scans the `skills/` directory
2. Loads all `.json` files from root and subdirectories
3. Validates each skill
4. Adds valid skills to the SkillLibrary
5. Reports any errors or failures

You can also manually reload skills by restarting Phoenix.

## Notes

- Skills with duplicate IDs will overwrite existing skills
- Invalid JSON files will be skipped with an error message
- Skills are loaded in alphabetical order by filename
- Subdirectories are scanned recursively
- Empty skill arrays in JSON files are ignored
