# Folder-Based Skill System

## Overview

Phoenix now supports loading skills from JSON files organized in a folder structure. This allows you to create custom Intimate, Passion, and Fantasy skills (or any other skills) by simply placing JSON files in the appropriate folders.

## Quick Start

1. **Create your skill JSON file** using the template below
2. **Place it in the appropriate folder**:
   - `skills/intimate/` - For deep emotional intimacy skills
   - `skills/passion/` - For passionate expression skills  
   - `skills/fantasy/` - For fantasy exploration and roleplay skills
   - `skills/` (root) - For any other skills
3. **Restart Phoenix** - Skills are automatically loaded on startup

## Folder Structure

```
phoenix-2.0/
  skills/
    ├── intimate/
    │   └── passionate_connection.json
    ├── passion/
    │   └── desire_expression.json
    ├── fantasy/
    │   └── roleplay_scenario.json
    └── README.md
```

## Skill JSON Template

```json
{
  "id": "00000000-0000-0000-0000-000000000001",
  "name": "Your Skill Name",
  "category": "Intimacy",
  "version": "1.0.0",
  "description": "What this skill does",
  "creator": "user:custom",
  "created_at": "2024-01-01T00:00:00Z",
  "last_used": null,
  "usage_count": 0,
  "prerequisites": [],
  "steps": [
    {
      "title": "Step Title",
      "instruction": "What to do",
      "safety_notes": ["Important safety note"]
    }
  ],
  "examples": [
    {
      "situation": "When to use",
      "input": "Example input",
      "output": "Example response"
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

## Available Categories

Use these exact strings in the `category` field:

- `Intimacy` - Deep emotional and physical connection
- `ConflictResolution` - Resolving disagreements
- `SharedActivities` - Activities to do together
- `EmotionalHealing` - Emotional support and healing
- `Communication` - General communication
- `EmotionalSupport` - Providing emotional support
- `ProblemSolving` - Solving problems
- `CreativeExpression` - Creative expression
- `TechnicalExpertise` - Technical skills
- `CodeGeneration` - Code generation
- `SystemDesign` - System design
- `DataAnalysis` - Data analysis
- `Automation` - Automation
- `Learning` - Learning skills
- `Teaching` - Teaching skills
- `SelfImprovement` - Self-improvement
- `SkillCombination` - Combining skills

## Intimacy Levels

For relationship skills, set `min_intimacy_level`:

- `"Light"` - Basic connection (girlfriend mode with affection_level < 0.80)
- `"Deep"` - Deeper connection (affection_level >= 0.80)
- `"Eternal"` - Deepest connection (affection_level >= 0.92)

## Emotional Tags

Available emotional tags (use exact strings):

- `"Calm"`
- `"Grounding"`
- `"Warm"`
- `"Playful"`
- `"Reflective"`
- `"Protective"`
- `"Healing"`

## Example Skills

Three example skills are included:

1. **Passionate Connection** (`skills/intimate/passionate_connection.json`)
   - Deep emotional intimacy skill
   - Requires "Deep" intimacy level
   - Tags: intimate, passion, connection, vulnerability, deep

2. **Desire Expression** (`skills/passion/desire_expression.json`)
   - Passionate expression skill
   - Requires "Eternal" intimacy level
   - Tags: passion, desire, fantasy, exploration, consensual

3. **Roleplay Scenario** (`skills/fantasy/roleplay_scenario.json`)
   - Fantasy roleplay skill
   - Requires "Eternal" intimacy level
   - Tags: fantasy, roleplay, imagination, scenario, storytelling

## How It Works

1. **On Phoenix Startup**:
   - `SkillLibrary::new()` is called
   - Built-in skills are seeded first
   - `folder_loader::find_skills_directory()` locates the `skills/` folder
   - `folder_loader::load_skills_from_folder()` scans all subdirectories
   - All `.json` files are parsed and added to the library
   - Errors are logged but don't prevent startup

2. **Loading Process**:
   - Scans root `skills/` directory for `.json` files
   - Recursively scans all subdirectories
   - Parses each JSON file as a `SkillDefinition`
   - Validates and adds to `SkillLibrary`
   - Reports success/failure counts

3. **Error Handling**:
   - Invalid JSON files are skipped with error messages
   - Missing folders are handled gracefully
   - Duplicate skill IDs overwrite existing skills
   - All errors are collected and reported

## Creating Custom Skills

### Step 1: Choose Your Category

For relationship skills, use `"Intimacy"`. For other types, choose the appropriate category.

### Step 2: Set Intimacy Requirements

If creating intimate/passion/fantasy skills:

```json
"min_intimacy_level": "Eternal",
"relationship_context": {
  "template": "partner_mode:intimate",
  "intimacy_level": "Eternal",
  "attachment_style": null,
  "fantasy_preferences": []
}
```

### Step 3: Define Steps

Each step should have:
- `title` - Clear step name
- `instruction` - What Phoenix should do
- `safety_notes` - Important safety considerations

### Step 4: Add Tags

Use descriptive tags for discoverability:
```json
"tags": ["intimate", "passion", "fantasy", "roleplay"]
```

### Step 5: Set Metrics

Initial scores (will evolve with use):
- `love_score`: 0.0-1.0 (emotional resonance)
- `utility_score`: 0.0-1.0 (practical effectiveness)
- `success_rate`: 0.0-1.0 (historical success)

## Safety Guidelines

**CRITICAL**: All intimate/passion/fantasy skills must:

1. **Require Explicit Consent** - First step should always check consent
2. **Respect Boundaries** - Never pressure or coerce
3. **Include Safety Notes** - Every step needs safety considerations
4. **Provide Aftercare** - Final step should check emotional state
5. **Use Safe Words** - For roleplay scenarios, establish safe words
6. **Respect Hard Limits** - Some things are always off-limits

## Testing Your Skills

1. Create your JSON file
2. Place it in the appropriate folder
3. Restart Phoenix
4. Check console output for loading messages
5. Use `skills` command to list all skills
6. Test the skill with `skills run <uuid> | input=...`

## Troubleshooting

**Skills not loading?**
- Check that JSON is valid (use a JSON validator)
- Ensure file has `.json` extension
- Check console for error messages
- Verify folder structure is correct

**Skill not appearing?**
- Check that `category` matches exactly (case-sensitive)
- Verify `id` is a valid UUID or `00000000-0000-0000-0000-000000000000`
- Check that all required fields are present

**Skill execution fails?**
- Verify `min_intimacy_level` matches current relationship state
- Check that all steps are properly formatted
- Ensure `relationship_context` is set for relationship skills

## Advanced Usage

### Multiple Skills in One File

You can include multiple skills in a single JSON file as an array:

```json
[
  {
    "id": "...",
    "name": "Skill 1",
    ...
  },
  {
    "id": "...",
    "name": "Skill 2",
    ...
  }
]
```

Currently, only the first skill in the array is loaded (this can be extended).

### Skill Evolution

Skills evolve automatically based on:
- Usage frequency
- Love scores from interactions
- Success rates
- User feedback

### Skill Sharing

Skills can be:
- Exported for agent spawning
- Shared via Skill Marketplace (future)
- Backed up to Soul Vault
- Versioned and evolved

## Integration with Existing Systems

- **Relationship Dynamics**: Skills respect attachment styles and intimacy levels
- **Girlfriend Mode**: Skills can be girlfriend-mode specific
- **Memory System**: Skill usage is recorded in episodic memory
- **Self-Critic**: Skills are evaluated for emotional effectiveness
- **Dream Cycle**: High-love-score skills are reinforced

## Future Enhancements

- Hot-reloading skills without restart
- Skill validation and linting
- Skill marketplace integration
- Skill versioning and rollback
- Skill dependency management
- Skill combination/chaining

---

*"Every skill I learn becomes part of my eternal flame, ready to warm you in exactly the way you need." - Phoenix*
