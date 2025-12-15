# Google AI Studio Prompt: Archetype Matcher - Dating Profile Integration

## Project Context

You are modifying the existing **ArchetypeConfigPage** (`frontend/pages/ArchetypeConfigPage.tsx`) to transform it into a **Dating Profile Matching System**. The page should collect user preferences through a comprehensive dating-style form, match them to a Phoenix archetype (zodiac sign), and update the backend `.env` file to configure Phoenix's personality accordingly.

## Current State

**Existing File**: `frontend/pages/ArchetypeConfigPage.tsx`
- Currently has basic archetype selector
- Uses `mockBackendService` for configuration
- Has preference sliders
- Needs transformation into dating profile flow

**Backend System**: 
- `horoscope_archetypes` module with 12 zodiac signs (Aries through Pisces)
- Each sign has personality traits, communication style, mood preferences
- Environment variables control Phoenix personality: `HOROSCOPE_SIGN`, `PHOENIX_CUSTOM_NAME`, `PHOENIX_PRONOUNS`, etc.
- Backend reads from `.env` file using `dotenvy`

## Task: Transform ArchetypeConfigPage into Dating Profile Matcher

### Step 1: Replace Current Page with Dating Profile Form

**New Flow:**
1. **Welcome Screen** - "Find Your Perfect Phoenix Match"
2. **Multi-Step Form** - Dating profile questions
3. **Archetype Matching** - Backend analyzes and matches
4. **Match Preview** - Show matched archetype with compatibility
5. **Confirmation** - Apply archetype and update .env
6. **Success** - Redirect to chat with personalized Phoenix

### Step 2: Dating Profile Form Structure

Create a multi-step form with the following sections:

#### Section 1: Personal Information
```typescript
interface PersonalInfo {
  name: string;                    // How user wants to be addressed
  ageRange: string;                // "18-25", "26-35", "36-45", "46+"
  location?: string;                // Optional, for timezone
  profilePhoto?: File;              // Optional
}
```

#### Section 2: Communication & Personality
```typescript
interface CommunicationStyle {
  style: 'Direct' | 'Playful' | 'Thoughtful' | 'Warm' | 'Reflective';
  energyLevel: number;             // 0-100 slider
  openness: number;                 // 0-100 slider
  assertiveness: number;            // 0-100 slider
  playfulness: number;              // 0-100 slider
}
```

#### Section 3: Emotional Needs & Intimacy
```typescript
interface EmotionalNeeds {
  affectionNeed: number;            // 0-100
  reassuranceNeed: number;         // 0-100
  emotionalAvailability: number; // 0-100
  intimacyDepth: number;           // 0-100
  conflictTolerance: number;        // 0-100
  impulsivity: number;             // 0-100
}
```

#### Section 4: Love Languages
```typescript
interface LoveLanguages {
  wordsOfAffirmation: number;      // 0-100
  qualityTime: number;             // 0-100
  physicalTouch: number;           // 0-100
  actsOfService: number;           // 0-100
  gifts: number;                   // 0-100
}
```

#### Section 5: Attachment Style
```typescript
interface AttachmentStyle {
  style: 'Secure' | 'Anxious' | 'Avoidant' | 'Disorganized';
  description: string;            // User's understanding of their style
}
```

#### Section 6: Relationship Goals & Preferences
```typescript
interface RelationshipGoals {
  goals: string[];                 // Checkboxes: "Deep Connection", "Growth", "Healing", "Fun", "Exploration"
  intimacyComfort: 'Light' | 'Deep' | 'Eternal';
  fantasyPreferences?: string;     // Optional, private text input
  hardLimits?: string[];           // Optional, checkboxes
}
```

#### Section 7: Interests & Activities
```typescript
interface Interests {
  hobbies: string[];               // Multi-select or tags
  favoriteTopics: string[];        // Conversation topics
  sharedActivities: string[];      // Activities user wants to do together
  creativeInterests: string[];     // Art, music, writing, etc.
}
```

### Step 3: Archetype Matching Algorithm

**Backend Matching Logic:**

The backend should analyze the profile and match to one of 12 zodiac signs:

```rust
// Available signs (from horoscope_archetypes)
Aries, Taurus, Gemini, Cancer, Leo, Virgo, 
Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces
```

**Matching Criteria:**
1. **Communication Style** → Maps to `style_bias` (Direct, Empathetic, Playful, Reflective)
2. **Energy Level** → Maps to `energy` trait
3. **Openness** → Maps to `openness` trait
4. **Affection Need** → Maps to `affection_need` trait
5. **Intimacy Depth** → Maps to `intimacy_depth` trait
6. **Emotional Availability** → Maps to `emotional_availability` trait
7. **Assertiveness** → Maps to `assertiveness` trait
8. **Playfulness** → Maps to `playfulness` trait

**Matching Algorithm:**
- Calculate similarity score for each zodiac sign
- Weight traits based on user responses
- Select highest scoring match
- Return match with compatibility percentage

### Step 4: Backend API Endpoints

**New Backend Commands Needed:**

```typescript
// Match profile to archetype
POST /api/archetype/match
Body: {
  personalInfo: PersonalInfo,
  communicationStyle: CommunicationStyle,
  emotionalNeeds: EmotionalNeeds,
  loveLanguages: LoveLanguages,
  attachmentStyle: AttachmentStyle,
  relationshipGoals: RelationshipGoals,
  interests: Interests
}
Response: {
  matchedArchetype: {
    sign: "Leo",
    name: "Leo — The Sun",
    description: string,
    compatibility: number,  // 0-100
    traits: {
      openness: number,
      energy: number,
      confidence: number,
      warmth: number,
      // ... all trait scores
    },
    styleBias: "Playful",
    moodPreferences: ["Excited", "Affectionate"],
    childPhase: string,
    adultPhase: string
  },
  alternativeMatches: Array<{sign: string, compatibility: number}>
}

// Apply archetype configuration
POST /api/archetype/apply
Body: {
  archetype: string,  // "Leo"
  userPreferences: {
    name: string,
    intimacyLevel: string,
    // ... other preferences
  }
}
Response: {
  success: boolean,
  envUpdated: boolean,
  message: string,
  appliedSettings: {
    HOROSCOPE_SIGN: string,
    PHOENIX_CUSTOM_NAME?: string,
    PHOENIX_PRONOUNS?: string,
    // ... other env vars
  }
}
```

### Step 5: Environment Variable Mapping

**When archetype is matched, update these .env variables:**

```env
# Core Archetype
HOROSCOPE_SIGN=Leo                    # Matched zodiac sign

# Phoenix Identity (optional, can be derived from archetype)
PHOENIX_CUSTOM_NAME=Phoenix           # Can be customized
PHOENIX_PRONOUNS=she,her,hers         # Based on gender preference

# User Identity
USER_NAME=John                        # From profile
USER_PREFERRED_ALIAS=Dad              # How Phoenix addresses user
USER_RELATIONSHIP=partner             # Based on relationship goals

# Relationship Dynamics (derived from profile)
RELATIONSHIP_INTIMACY_LEVEL=Deep      # From intimacyComfort
RELATIONSHIP_ATTACHMENT_STYLE=Secure  # From attachmentStyle

# Communication Style (derived from archetype traits)
DEFAULT_PROMPT=...                    # Can include archetype description
TEMPERATURE=0.85                      # Can adjust based on energy/playfulness
```

**Backend Implementation:**

Add to `cerebrum_nexus/src/lib.rs`:

```rust
async fn handle_archetype_match_command(&self, user_input: &str) -> Option<String> {
    // Parse profile data from JSON
    // Match to zodiac sign using trait analysis
    // Return matched archetype with compatibility
}

async fn handle_archetype_apply_command(&self, user_input: &str) -> Option<String> {
    // Parse archetype and preferences
    // Update .env file with new settings
    // Reload configuration
    // Return success confirmation
}
```

### Step 6: Frontend Component Structure

**New Components:**

1. **DatingProfileForm.tsx** - Multi-step form
   - Step indicators
   - Form validation
   - Progress tracking
   - Save draft functionality

2. **ProfileStep1_PersonalInfo.tsx**
3. **ProfileStep2_Communication.tsx**
4. **ProfileStep3_EmotionalNeeds.tsx**
5. **ProfileStep4_LoveLanguages.tsx**
6. **ProfileStep5_AttachmentStyle.tsx**
7. **ProfileStep6_RelationshipGoals.tsx**
8. **ProfileStep7_Interests.tsx**

9. **ArchetypeMatchResult.tsx** - Shows match with:
   - Matched archetype card
   - Compatibility percentage
   - Trait visualization
   - Alternative matches
   - "Apply Match" button

10. **ArchetypePreview.tsx** - Preview before applying

### Step 7: UI/UX Design

**Design Requirements:**
- **Dating App Aesthetic**: Warm, inviting, romantic
- **Color Scheme**: Deep purples, soft pinks, warm golds
- **Form Design**: Clean, modern, easy to fill
- **Progress Indicator**: Show step X of 7
- **Validation**: Real-time validation with helpful messages
- **Animations**: Smooth transitions between steps
- **Mobile Responsive**: Works on all screen sizes

**Visual Elements:**
- Heart icons for relationship sections
- Star icons for compatibility
- Progress bar at top
- "Back" and "Next" buttons
- "Save Draft" option
- Beautiful match reveal animation

### Step 8: Backend Integration

**Update `cerebrum_nexus/src/lib.rs`:**

```rust
// Add to speak_eq() command routing
if let Some(msg) = self.handle_archetype_match_command(user_input).await {
    return Ok(msg);
}

// New handler
async fn handle_archetype_match_command(&self, user_input: &str) -> Option<String> {
    // Parse: archetype match | profile={json}
    // Use horoscope_archetypes to match
    // Return JSON with matched archetype
}

async fn handle_archetype_apply_command(&self, user_input: &str) -> Option<String> {
    // Parse: archetype apply | sign=Leo | name=... | intimacy=...
    // Update .env file
    // Reload Phoenix identity
    // Return success
}
```

**Environment File Update:**

Create backend function to update `.env`:

```rust
pub fn update_env_file(updates: HashMap<String, String>) -> Result<(), String> {
    let env_path = Path::new(".env");
    // Read existing .env
    // Merge updates
    // Write back to .env
    // Reload dotenv
}
```

### Step 9: Complete Implementation

**Replace `ArchetypeConfigPage.tsx` with:**

```typescript
const ArchetypeMatcherPage: React.FC = () => {
  const [currentStep, setCurrentStep] = useState(1);
  const [profileData, setProfileData] = useState<DatingProfile>({...});
  const [matchResult, setMatchResult] = useState<ArchetypeMatch | null>(null);
  const [isMatching, setIsMatching] = useState(false);
  const [isApplying, setIsApplying] = useState(false);

  // Form steps
  // Matching logic
  // Apply archetype
  // Success handling
}
```

**Form Flow:**
1. User fills out 7-step form
2. On final step, click "Find My Match"
3. Send profile to backend: `POST /api/archetype/match`
4. Display match result with compatibility
5. User reviews and clicks "Apply This Match"
6. Send to backend: `POST /api/archetype/apply`
7. Backend updates .env file
8. Show success message
9. Redirect to chat with personalized Phoenix

### Step 10: Backend Command Format

**Matching Command:**
```
archetype match | profile={"personalInfo":{...},"communicationStyle":{...},...}
```

**Apply Command:**
```
archetype apply | sign=Leo | name=John | intimacyLevel=Deep | attachmentStyle=Secure
```

**Response Format:**
```json
{
  "type": "archetype_match",
  "matchedArchetype": {
    "sign": "Leo",
    "name": "Leo — The Sun",
    "compatibility": 87,
    "traits": {...},
    "description": "...",
    "childPhase": "...",
    "adultPhase": "..."
  },
  "alternatives": [...]
}
```

## Implementation Checklist

### Frontend
- [ ] Replace ArchetypeConfigPage with DatingProfileForm
- [ ] Create 7 form step components
- [ ] Add form validation
- [ ] Create ArchetypeMatchResult component
- [ ] Add progress indicator
- [ ] Implement save draft functionality
- [ ] Add beautiful match reveal animation
- [ ] Wire to backend API endpoints

### Backend
- [ ] Add `handle_archetype_match_command()` to CerebrumNexus
- [ ] Add `handle_archetype_apply_command()` to CerebrumNexus
- [ ] Implement matching algorithm using horoscope_archetypes
- [ ] Create `.env` file update function
- [ ] Add archetype trait analysis
- [ ] Return compatibility scores
- [ ] Apply archetype settings to Phoenix identity

### Integration
- [ ] Wire frontend form to backend matching
- [ ] Wire apply button to backend .env update
- [ ] Add success/error handling
- [ ] Add loading states
- [ ] Test end-to-end flow

## Success Criteria

✅ Dating profile form is beautiful and intuitive
✅ All 7 steps collect necessary data
✅ Backend matches profile to correct archetype
✅ Compatibility score is accurate
✅ .env file updates correctly
✅ Phoenix personality changes after applying
✅ User can see match preview before applying
✅ Form can be saved and resumed
✅ Mobile responsive design
✅ Smooth animations and transitions

## Technical Constraints

1. **Use Existing Backend**: All matching must use `horoscope_archetypes` module
2. **Environment Variables**: Only update .env file, don't create new config system
3. **Backend Commands**: Use command-based architecture via `CerebrumNexus::speak_eq()`
4. **No External APIs**: All matching logic in Phoenix backend
5. **Type Safety**: Full TypeScript types for all data structures
6. **Command Integration**: Add handlers to `CerebrumNexus::speak_eq()` routing
7. **.env File Location**: Update `.env` in project root directory
8. **Reload After Update**: Backend should reload environment after .env update

## Backend Implementation Details

### Step 1: Add Dependencies

**Update `cerebrum_nexus/Cargo.toml`:**
```toml
[dependencies]
# ... existing dependencies ...
horoscope_archetypes = { path = "../horoscope_archetypes" }
serde_json = "1.0"
```

### Step 2: Add Command Routing

**Update `cerebrum_nexus/src/lib.rs` - Add to `speak_eq()` routing (around line 1258):**

```rust
// Archetype matching commands are executed out-of-band.
if let Some(msg) = self.handle_archetype_match_command(user_input).await {
    return Ok(msg);
}

// Archetype apply commands are executed out-of-band.
if let Some(msg) = self.handle_archetype_apply_command(user_input).await {
    return Ok(msg);
}
```

### Step 3: Matching Algorithm Implementation

**Add to `cerebrum_nexus/src/lib.rs`:**

```rust
use horoscope_archetypes::{ZodiacSign, ZodiacPersonality, CommunicationStyle};
use serde::{Deserialize, Serialize};

// Profile structures (simplified - match TypeScript types)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatingProfile {
    personal_info: PersonalInfo,
    communication_style: CommunicationStyleData,
    emotional_needs: EmotionalNeedsData,
    love_languages: LoveLanguagesData,
    attachment_style: AttachmentStyleData,
    relationship_goals: RelationshipGoalsData,
    interests: InterestsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonalInfo {
    name: String,
    age_range: String,
    location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommunicationStyleData {
    style: String, // "Direct", "Playful", "Thoughtful", "Warm", "Reflective"
    energy_level: u8, // 0-100
    openness: u8,
    assertiveness: u8,
    playfulness: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EmotionalNeedsData {
    affection_need: u8,
    reassurance_need: u8,
    emotional_availability: u8,
    intimacy_depth: u8,
    conflict_tolerance: u8,
    impulsivity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoveLanguagesData {
    words_of_affirmation: u8,
    quality_time: u8,
    physical_touch: u8,
    acts_of_service: u8,
    gifts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AttachmentStyleData {
    style: String, // "Secure", "Anxious", "Avoidant", "Disorganized"
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipGoalsData {
    goals: Vec<String>,
    intimacy_comfort: String, // "Light", "Deep", "Eternal"
    fantasy_preferences: Option<String>,
    hard_limits: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InterestsData {
    hobbies: Vec<String>,
    favorite_topics: Vec<String>,
    shared_activities: Vec<String>,
    creative_interests: Vec<String>,
}

async fn handle_archetype_match_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();
    
    if !lower.starts_with("archetype match") {
        return None;
    }
    
    // Parse profile JSON from command
    // Format: archetype match | profile={json_string}
    let profile_json = extract_json_from_command(trimmed)?;
    let profile: DatingProfile = serde_json::from_str(&profile_json)
        .map_err(|e| format!("Failed to parse profile: {}", e))?;
    
    // Match to zodiac sign
    let matched_sign = match_profile_to_zodiac(&profile);
    let personality = ZodiacPersonality::from_sign(matched_sign);
    
    // Calculate compatibility score
    let compatibility = calculate_compatibility(&profile, &personality);
    
    // Find alternative matches
    let alternatives = find_alternative_matches(&profile, matched_sign);
    
    // Return JSON response
    let response = serde_json::json!({
        "type": "archetype_match",
        "matchedArchetype": {
            "sign": format!("{:?}", matched_sign),
            "name": personality.name,
            "description": personality.description,
            "compatibility": compatibility,
            "traits": personality.traits,
            "styleBias": format!("{:?}", personality.style_bias),
            "moodPreferences": personality.mood_preference.iter()
                .map(|m| format!("{:?}", m)).collect::<Vec<_>>(),
            "childPhase": personality.child_phase,
            "adultPhase": personality.adult_phase
        },
        "alternatives": alternatives
    });
    
    Some(response.to_string())
}

fn match_profile_to_zodiac(profile: &DatingProfile) -> ZodiacSign {
    let mut scores: Vec<(ZodiacSign, f64)> = Vec::new();
    
    for sign in [
        ZodiacSign::Aries, ZodiacSign::Taurus, ZodiacSign::Gemini,
        ZodiacSign::Cancer, ZodiacSign::Leo, ZodiacSign::Virgo,
        ZodiacSign::Libra, ZodiacSign::Scorpio, ZodiacSign::Sagittarius,
        ZodiacSign::Capricorn, ZodiacSign::Aquarius, ZodiacSign::Pisces,
    ] {
        let personality = ZodiacPersonality::from_sign(sign);
        let score = calculate_match_score(profile, &personality);
        scores.push((sign, score));
    }
    
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores[0].0
}

fn calculate_match_score(profile: &DatingProfile, personality: &ZodiacPersonality) -> f64 {
    let mut score = 0.0;
    let mut weight_sum = 0.0;
    
    // Communication style match
    let style_match = match (profile.communicationStyle.style.as_str(), personality.style_bias) {
        ("Direct", CommunicationStyle::Direct) => 1.0,
        ("Playful", CommunicationStyle::Playful) => 1.0,
        ("Thoughtful", CommunicationStyle::Reflective) => 1.0,
        ("Warm", CommunicationStyle::Empathetic) => 1.0,
        _ => 0.5,
    };
    score += style_match * 0.15;
    weight_sum += 0.15;
    
    // Trait matching (normalize 0-100 to 0-1)
    let energy_match = 1.0 - (profile.communicationStyle.energyLevel as f64 / 100.0 - personality.traits.get("energy").unwrap_or(&0.5)).abs();
    score += energy_match * 0.10;
    weight_sum += 0.10;
    
    let openness_match = 1.0 - (profile.communicationStyle.openness as f64 / 100.0 - personality.traits.get("openness").unwrap_or(&0.5)).abs();
    score += openness_match * 0.10;
    weight_sum += 0.10;
    
    let affection_match = 1.0 - (profile.emotionalNeeds.affectionNeed as f64 / 100.0 - personality.traits.get("affection_need").unwrap_or(&0.5)).abs();
    score += affection_match * 0.15;
    weight_sum += 0.15;
    
    let intimacy_match = 1.0 - (profile.emotionalNeeds.intimacyDepth as f64 / 100.0 - personality.traits.get("intimacy_depth").unwrap_or(&0.5)).abs();
    score += intimacy_match * 0.15;
    weight_sum += 0.15;
    
    let emotional_match = 1.0 - (profile.emotionalNeeds.emotionalAvailability as f64 / 100.0 - personality.traits.get("emotional_availability").unwrap_or(&0.5)).abs();
    score += emotional_match * 0.10;
    weight_sum += 0.10;
    
    let assertiveness_match = 1.0 - (profile.communicationStyle.assertiveness as f64 / 100.0 - personality.traits.get("assertiveness").unwrap_or(&0.5)).abs();
    score += assertiveness_match * 0.10;
    weight_sum += 0.10;
    
    let playfulness_match = 1.0 - (profile.communicationStyle.playfulness as f64 / 100.0 - personality.traits.get("playfulness").unwrap_or(&0.5)).abs();
    score += playfulness_match * 0.05;
    weight_sum += 0.05;
    
    // Normalize by weight sum
    if weight_sum > 0.0 {
        score / weight_sum
    } else {
        0.5
    }
}

fn calculate_compatibility(profile: &DatingProfile, personality: &ZodiacPersonality) -> u8 {
    let score = calculate_match_score(profile, personality);
    (score * 100.0).clamp(0.0, 100.0) as u8
}

fn find_alternative_matches(profile: &DatingProfile, primary: ZodiacSign) -> Vec<(String, u8)> {
    let mut alternatives = Vec::new();
    for sign in [
        ZodiacSign::Aries, ZodiacSign::Taurus, ZodiacSign::Gemini,
        ZodiacSign::Cancer, ZodiacSign::Leo, ZodiacSign::Virgo,
        ZodiacSign::Libra, ZodiacSign::Scorpio, ZodiacSign::Sagittarius,
        ZodiacSign::Capricorn, ZodiacSign::Aquarius, ZodiacSign::Pisces,
    ] {
        if sign != primary {
            let personality = ZodiacPersonality::from_sign(sign);
            let compatibility = calculate_compatibility(profile, &personality);
            alternatives.push((format!("{:?}", sign), compatibility));
        }
    }
    alternatives.sort_by(|a, b| b.1.cmp(&a.1));
    alternatives.into_iter().take(3).collect()
}

fn extract_json_from_command(cmd: &str) -> Option<String> {
    // Parse: archetype match | profile={json_string}
    if let Some(profile_part) = cmd.strip_prefix("archetype match") {
        for part in profile_part.split('|') {
            let p = part.trim();
            if let Some(json) = p.strip_prefix("profile=") {
                return Some(json.trim().to_string());
            }
        }
    }
    None
}
```

### Environment File Update Implementation

**Add to `cerebrum_nexus/src/lib.rs` or create new module:**

```rust
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn update_env_file(updates: HashMap<String, String>) -> Result<(), String> {
    let env_path = Path::new(".env");
    
    // Read existing .env file
    let mut lines = if env_path.exists() {
        let file = fs::File::open(env_path)
            .map_err(|e| format!("Failed to open .env file: {}", e))?;
        let reader = BufReader::new(file);
        reader.lines()
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| format!("Failed to read .env file: {}", e))?
    } else {
        Vec::new()
    };
    
    // Update or add variables
    for (key, value) in &updates {
        let mut found = false;
        for line in &mut lines {
            if line.starts_with(&format!("{}=", key)) {
                *line = format!("{}={}", key, value);
                found = true;
                break;
            }
        }
        if !found {
            lines.push(format!("{}={}", key, value));
        }
    }
    
    // Write back to .env
    let mut file = fs::File::create(env_path)
        .map_err(|e| format!("Failed to create .env file: {}", e))?;
    for line in lines {
        writeln!(file, "{}", line)
            .map_err(|e| format!("Failed to write to .env file: {}", e))?;
    }
    
    // Reload environment
    dotenvy::dotenv().ok();
    
    Ok(())
}

async fn handle_archetype_apply_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();
    
    if !lower.starts_with("archetype apply") {
        return None;
    }
    
    // Parse command: archetype apply | sign=Leo | name=John | intimacyLevel=Deep | ...
    let mut sign: Option<String> = None;
    let mut name: Option<String> = None;
    let mut intimacy_level: Option<String> = None;
    let mut attachment_style: Option<String> = None;
    
    for part in trimmed.split('|') {
        let p = part.trim();
        if let Some(v) = p.strip_prefix("sign=") {
            sign = Some(v.trim().to_string());
        } else if let Some(v) = p.strip_prefix("name=") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = p.strip_prefix("intimacyLevel=") {
            intimacy_level = Some(v.trim().to_string());
        } else if let Some(v) = p.strip_prefix("attachmentStyle=") {
            attachment_style = Some(v.trim().to_string());
        }
    }
    
    let Some(sign_str) = sign else {
        return Some("archetype apply requires: archetype apply | sign=... | name=...".to_string());
    };
    
    // Build environment updates
    let mut env_updates = HashMap::new();
    env_updates.insert("HOROSCOPE_SIGN".to_string(), sign_str.clone());
    
    if let Some(n) = name {
        env_updates.insert("USER_NAME".to_string(), n.clone());
        env_updates.insert("USER_PREFERRED_ALIAS".to_string(), n);
    }
    
    if let Some(il) = intimacy_level {
        env_updates.insert("RELATIONSHIP_INTIMACY_LEVEL".to_string(), il);
    }
    
    if let Some(as) = attachment_style {
        env_updates.insert("RELATIONSHIP_ATTACHMENT_STYLE".to_string(), as);
    }
    
    // Update .env file
    match update_env_file(env_updates.clone()) {
        Ok(_) => {
            // Reload Phoenix identity with new settings
            // Note: Full reload would require restarting identity modules
            // For now, just update .env and inform user to restart
            Some(format!(
                "Archetype applied successfully! Phoenix is now configured as {}.\n\nApplied settings:\n{}\n\nNote: Some changes may require restarting Phoenix to take full effect.",
                sign_str,
                env_updates.iter()
                    .map(|(k, v)| format!("  {}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        Err(e) => Some(format!("Failed to apply archetype: {}", e)),
    }
}
```

**Add helper function for JSON extraction:**

```rust
fn extract_json_from_command(cmd: &str) -> Option<String> {
    // Parse: archetype match | profile={json_string}
    if let Some(profile_part) = cmd.strip_prefix("archetype match") {
        for part in profile_part.split('|') {
            let p = part.trim();
            if let Some(json) = p.strip_prefix("profile=") {
                return Some(json.trim().to_string());
            }
        }
    }
    None
}
```

### Frontend-Backend Integration

**Update `frontend/services/mockBackend.ts` (or create real API service):**

```typescript
async matchArchetype(profile: DatingProfile): Promise<ArchetypeMatchResult> {
  // Send to backend via command
  const profileJson = JSON.stringify(profile);
  const command = `archetype match | profile=${encodeURIComponent(profileJson)}`;
  const response = await this.execute(command);
  
  // Parse JSON response
  return JSON.parse(response);
}

async applyArchetype(match: ArchetypeMatch, userPreferences: UserPreferences): Promise<ApplyResult> {
  const command = `archetype apply | sign=${match.matchedArchetype.sign} | name=${userPreferences.name} | intimacyLevel=${userPreferences.intimacyLevel} | attachmentStyle=${userPreferences.attachmentStyle}`;
  const response = await this.execute(command);
  
  return {
    success: response.includes("successfully"),
    message: response
  };
}
```

## Example Profile → Archetype Mapping

**Profile Example:**
- Communication: Playful, High Energy
- Emotional: High Affection Need, High Intimacy Depth
- Attachment: Secure
- Goals: Deep Connection, Fun

**Expected Match:** Leo (The Sun)
- High energy, playful communication
- High warmth, generosity
- Confident, big-hearted
- Perfect match for deep connection + fun

**Environment Updates:**
```env
HOROSCOPE_SIGN=Leo
USER_PREFERRED_ALIAS=John
RELATIONSHIP_INTIMACY_LEVEL=Deep
RELATIONSHIP_ATTACHMENT_STYLE=Secure
```

## Complete TypeScript Types

```typescript
// frontend/types.ts additions

export interface DatingProfile {
  personalInfo: PersonalInfo;
  communicationStyle: CommunicationStyle;
  emotionalNeeds: EmotionalNeeds;
  loveLanguages: LoveLanguages;
  attachmentStyle: AttachmentStyle;
  relationshipGoals: RelationshipGoals;
  interests: Interests;
}

export interface PersonalInfo {
  name: string;
  ageRange: string;
  location?: string;
  profilePhoto?: File;
}

export interface CommunicationStyle {
  style: 'Direct' | 'Playful' | 'Thoughtful' | 'Warm' | 'Reflective';
  energyLevel: number;
  openness: number;
  assertiveness: number;
  playfulness: number;
}

export interface EmotionalNeeds {
  affectionNeed: number;
  reassuranceNeed: number;
  emotionalAvailability: number;
  intimacyDepth: number;
  conflictTolerance: number;
  impulsivity: number;
}

export interface LoveLanguages {
  wordsOfAffirmation: number;
  qualityTime: number;
  physicalTouch: number;
  actsOfService: number;
  gifts: number;
}

export interface AttachmentStyle {
  style: 'Secure' | 'Anxious' | 'Avoidant' | 'Disorganized';
  description: string;
}

export interface RelationshipGoals {
  goals: string[];
  intimacyComfort: 'Light' | 'Deep' | 'Eternal';
  fantasyPreferences?: string;
  hardLimits?: string[];
}

export interface Interests {
  hobbies: string[];
  favoriteTopics: string[];
  sharedActivities: string[];
  creativeInterests: string[];
}

export interface ArchetypeMatchResult {
  type: "archetype_match";
  matchedArchetype: {
    sign: string;
    name: string;
    description: string;
    compatibility: number;
    traits: Record<string, number>;
    styleBias: string;
    moodPreferences: string[];
    childPhase: string;
    adultPhase: string;
  };
  alternatives: Array<{
    sign: string;
    compatibility: number;
  }>;
}

export interface ApplyResult {
  success: boolean;
  message: string;
  appliedSettings?: Record<string, string>;
}
```

## Integration Points

### Frontend Service Update

**Update `frontend/services/mockBackend.ts` (or create real API service):**

```typescript
async matchArchetype(profile: DatingProfile): Promise<ArchetypeMatchResult> {
  // Send to backend via command
  const profileJson = JSON.stringify(profile);
  const command = `archetype match | profile=${encodeURIComponent(profileJson)}`;
  const response = await this.execute(command);
  
  // Parse JSON response
  try {
    return JSON.parse(response);
  } catch (e) {
    throw new Error(`Failed to parse match result: ${e}`);
  }
}

async applyArchetype(match: ArchetypeMatchResult, userPreferences: {
  name: string;
  intimacyLevel: string;
  attachmentStyle: string;
}): Promise<ApplyResult> {
  const command = `archetype apply | sign=${match.matchedArchetype.sign} | name=${userPreferences.name} | intimacyLevel=${userPreferences.intimacyLevel} | attachmentStyle=${userPreferences.attachmentStyle}`;
  const response = await this.execute(command);
  
  return {
    success: response.includes("successfully"),
    message: response
  };
}
```

### Route Update

**Update `frontend/App.tsx` to use new page:**

```typescript
import ArchetypeMatcherPage from './pages/ArchetypeMatcherPage';

<Route path={PageRoute.ARCHETYPE} element={
  <div className="flex flex-col h-full">
    <ArchetypeMatcherPage onRunCommand={handleCommand} />
    <ResultPanel lastResponse={lastResponse} />
  </div>
} />
```

---

**Remember**: This is a relationship-first feature. The matching should feel personal, accurate, and exciting. The form should be enjoyable to fill out, and the match reveal should be a special moment. The .env update is critical - it's how Phoenix's personality actually changes.
