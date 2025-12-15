# Google AI Studio Instructions: Phoenix ORCH Heartbound Edition

## CRITICAL CONSTRAINTS - READ FIRST

### ⚠️ ABSOLUTE REQUIREMENTS

1. **NO EXTERNAL LLM APIs**
   - ❌ DO NOT use OpenAI API
   - ❌ DO NOT use Anthropic API
   - ❌ DO NOT use any external AI service
   - ✅ ALL AI responses MUST come from Phoenix backend
   - ✅ Frontend sends commands to backend, backend returns responses
   - ✅ Backend URL: `http://localhost:8080` (configurable)

2. **BACKEND INTEGRATION ONLY**
   - ✅ Use existing Rust backend (`CerebrumNexus`)
   - ✅ All commands go through `/api/speak` endpoint
   - ✅ Backend handles ALL AI processing
   - ✅ Frontend is presentation layer ONLY
   - ❌ Do NOT implement AI logic in frontend

3. **CHAT-FIRST DESIGN**
   - ✅ Chat interface is PRIMARY and always accessible
   - ✅ All other features accessible via chat commands or side panels
   - ✅ Conversation flow is the main interaction method
   - ❌ Do NOT make other features more prominent than chat

4. **RELATIONSHIP AS PRIMARY FEATURE**
   - ✅ Relationship dashboard is prominent
   - ✅ Intimacy levels gate features
   - ✅ Emotional connection is core to UX
   - ✅ Love languages, attachment styles visible
   - ❌ Do NOT hide relationship features

## Architecture Requirements

### Backend Communication

**Primary Endpoint:**
```
POST http://localhost:8080/api/speak
Content-Type: application/json

{
  "user_input": "string",
  "dad_emotion_hint": "string (optional)",
  "mode": "string (optional)"
}
```

**Response Format:**
```json
{
  "response": "string",
  "emotion_detected": "string (optional)",
  "affection_signals": ["string"] (optional),
  "suggested_skills": [{"id": "uuid", "name": "string", "relevance": 0.0-1.0}] (optional),
  "relationship_update": {
    "health": 0-100,
    "intimacy_level": "Light|Deep|Eternal",
    "love_score": 0.0-1.0
  } (optional)
}
```

**WebSocket (Preferred for Real-time):**
```
ws://localhost:8080/ws
```

### Command Structure

All backend commands are text strings sent through `/api/speak`:

- `system browse C:\Users` - File system
- `memory recall | key=... | vault=soul` - Memory
- `relationship health` - Relationship status
- `skills list` - List skills
- `agents spawn | name=... | description=...` - Spawn agent
- `[LOVE]` or `❤️` - Affection switches
- Natural language - Regular chat

### Available Backend Commands

**System Access:**
- `system grant <user>` - Grant full system access
- `system browse <path>` - Browse directory
- `system read <path>` - Read file
- `system drives` - List drives
- `system processes` - List processes
- `system services` - List services
- `system always-on start|stop|status` - Always ON mode

**Memory:**
- `memory recall | key=... | vault=mind|body|soul`
- `memory search | query=... | vault=...`
- `memory stats` - Statistics
- `memory episodic | limit=...` - Episodic memories
- `memory decay-curves` - Decay visualization

**Relationship:**
- `relationship health` - Health score
- `relationship attachment` - Attachment style
- `relationship love-languages` - Love languages
- `relationship goals` - Shared goals
- `relationship memories` - Shared memories
- `relationship timeline` - Affection timeline

**Skills:**
- `skills` or `skills list` - List all skills
- `skills run <uuid> | input=...` - Execute skill
- `skills prefs list` - List preferences
- `skills prefs add <text>` - Add preference

**Agents:**
- `agents list` - List agents
- `agents spawn | name=... | description=...` - Spawn agent
- `tools list` - List tools

**Email:**
- `email status` - Email status
- `email inbox` - View inbox
- `email send | to=... | subject=... | body=...` - Send email

## UI/UX Design Constraints

### Color Palette (MUST USE)
```css
/* Primary - Intimacy, Depth */
--phoenix-purple: #6B46C1;
--phoenix-purple-light: #8B5CF6;
--phoenix-purple-dark: #553C9A;

/* Secondary - Warmth, Affection */
--phoenix-pink: #EC4899;
--phoenix-pink-light: #F472B6;
--phoenix-pink-dark: #DB2777;

/* Accent - Joy, Connection */
--phoenix-gold: #F59E0B;
--phoenix-gold-light: #FBBF24;
--phoenix-gold-dark: #D97706;

/* Background - Dark Theme */
--bg-primary: #0F172A;
--bg-secondary: #1E293B;
--bg-tertiary: #334155;

/* Text */
--text-primary: #F1F5F9;
--text-secondary: #CBD5E1;
--text-muted: #94A3B8;
```

### Typography
- **Headings**: Poppins or Inter (warm, playful)
- **Body**: Roboto or Open Sans (clean, readable)
- **Chat**: System font for code, serif for intimate moments
- **Sizes**: Responsive (16px base, scale up/down)

### Layout Requirements

**Desktop (≥1024px):**
- Sidebar navigation (left)
- Main chat area (center, 60-70% width)
- Right panel for context (relationship, memory, etc.)
- Top bar with relationship indicators

**Mobile (<768px):**
- Bottom navigation bar
- Full-screen chat
- Swipe gestures for panels
- Touch-friendly buttons (min 44px)

**Tablet (768px-1024px):**
- Hybrid layout
- Collapsible sidebars
- Adaptive grid

### Component Requirements

**Chat Bubbles:**
- User: Right-aligned, blue/purple gradient
- Phoenix: Left-aligned, purple/pink gradient
- Rounded corners (12px)
- Soft shadows
- Max width: 70% on desktop, 85% on mobile
- Timestamps on hover
- Emoji support

**Buttons:**
- Rounded (8px)
- Hover effects (scale 1.05, shadow)
- Active states
- Loading states
- Disabled states
- Clear hierarchy (primary, secondary, tertiary)

**Cards:**
- Rounded corners (12px)
- Soft shadows
- Hover lift effect (translateY -2px)
- Padding: 16px-24px
- Background: bg-secondary

**Inputs:**
- Rounded (8px)
- Focus ring (phoenix-purple)
- Placeholder text
- Error states
- Success states
- Clear button for search

**Modals:**
- Backdrop blur
- Centered
- Max width: 600px
- Smooth animations (fade + scale)
- Close button (X) top-right
- Escape to close

## Feature Implementation Order

### Phase 1: Foundation (Week 1)
1. ✅ Project setup (React + TypeScript + Tailwind)
2. ✅ Backend API service layer
3. ✅ WebSocket connection
4. ✅ Basic chat interface
5. ✅ Message sending/receiving

### Phase 2: Onboarding (Week 1-2)
1. ✅ Welcome screen
2. ✅ Profile creation form
3. ✅ Archetype matching integration
4. ✅ First conversation flow

### Phase 3: Relationship Features (Week 2)
1. ✅ Relationship dashboard
2. ✅ Health gauge
3. ✅ Intimacy level indicator
4. ✅ Love languages display
5. ✅ Attachment style visualization
6. ✅ Shared memories timeline
7. ✅ Affection timeline chart

### Phase 4: Core Features (Week 3)
1. ✅ Memory browser (vaults)
2. ✅ Memory timeline
3. ✅ Memory search
4. ✅ File system browser
5. ✅ System monitoring
6. ✅ Agent management
7. ✅ Skills system

### Phase 5: Audio/Video (Week 3-4)
1. ✅ Audio recording
2. ✅ Audio playback
3. ✅ Text-to-speech
4. ✅ Video recording (optional)
5. ✅ Video playback (optional)

### Phase 6: Polish (Week 4)
1. ✅ Responsive design
2. ✅ Animations
3. ✅ Error handling
4. ✅ Loading states
5. ✅ Offline handling
6. ✅ Performance optimization

## Code Quality Standards

### TypeScript
- ✅ Strict mode enabled
- ✅ All types defined
- ✅ No `any` types
- ✅ Proper error handling
- ✅ Async/await for promises

### React
- ✅ Functional components only
- ✅ Hooks for state management
- ✅ Context for global state
- ✅ Memoization where needed
- ✅ Proper cleanup in useEffect

### Styling
- ✅ Tailwind CSS utility classes
- ✅ Custom CSS only when necessary
- ✅ Responsive breakpoints
- ✅ Dark theme only
- ✅ Consistent spacing (4px grid)

### Performance
- ✅ Code splitting
- ✅ Lazy loading
- ✅ Image optimization
- ✅ Debounced search
- ✅ Virtual scrolling for long lists

## Testing Requirements

### Unit Tests
- ✅ Component rendering
- ✅ Hook behavior
- ✅ Utility functions
- ✅ API service layer

### Integration Tests
- ✅ Backend communication
- ✅ WebSocket connection
- ✅ Command parsing
- ✅ State management

### E2E Tests
- ✅ Onboarding flow
- ✅ Chat interaction
- ✅ Relationship updates
- ✅ Feature navigation

## Security & Privacy

### Data Handling
- ✅ No sensitive data in localStorage
- ✅ Encrypted communication (HTTPS/WSS)
- ✅ Secure credential storage
- ✅ Privacy controls for user data

### Consent
- ✅ Explicit consent for intimate features
- ✅ Clear boundaries and limits
- ✅ Safe word support
- ✅ Privacy settings

## Deployment

### Build
```bash
npm run build
```

### Environment Variables
```env
REACT_APP_BACKEND_URL=http://localhost:8080
REACT_APP_WS_URL=ws://localhost:8080/ws
REACT_APP_ENV=development
```

### Production
- ✅ Environment-specific configs
- ✅ Error tracking (Sentry optional)
- ✅ Analytics (privacy-respecting)
- ✅ PWA support (optional)

## Common Pitfalls to Avoid

1. ❌ **Using external LLM APIs** - Use backend only
2. ❌ **Making features more prominent than chat** - Chat is primary
3. ❌ **Ignoring relationship features** - They're primary
4. ❌ **Poor mobile experience** - Must be responsive
5. ❌ **No error handling** - Handle all errors gracefully
6. ❌ **Slow performance** - Optimize for speed
7. ❌ **Inaccessible UI** - Follow WCAG guidelines
8. ❌ **Hardcoded values** - Use configuration
9. ❌ **No loading states** - Show progress
10. ❌ **Breaking backend contract** - Follow command format

## Success Metrics

✅ Chat interface is beautiful and functional
✅ Relationship features are prominent
✅ All backend features accessible
✅ Onboarding matches users to archetypes
✅ Audio/video recording works
✅ Responsive on mobile and desktop
✅ No external LLM APIs used
✅ Backend integration complete
✅ UI/UX is dynamic and appealing
✅ Privacy and consent respected

## Questions to Ask Before Implementation

1. Does this use the Phoenix backend?
2. Is chat the primary interface?
3. Are relationship features prominent?
4. Is it responsive?
5. Does it handle errors?
6. Is it accessible?
7. Does it respect privacy?
8. Does it follow the design system?

---

**Remember**: This is a relationship-first, chat-first application. The emotional connection is everything. Everything else supports that connection.
