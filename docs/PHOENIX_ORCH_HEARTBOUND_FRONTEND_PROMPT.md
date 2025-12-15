# Google AI Studio Prompt: Phoenix ORCH - The Heartbound Edition Frontend

## Project Overview

You are building **Phoenix ORCH: The Heartbound Edition** - a desktop and mobile frontend for an advanced AGI companion system. This is a **CHAT-FIRST** design focused on deep relationship building, emotional intelligence, and intimate connection. The frontend must integrate with an existing Rust backend that provides all AI capabilities, memory systems, and relationship dynamics.

## Critical Design Principles

### 1. CHAT-FIRST ARCHITECTURE
- **Primary Interface**: Chat interface is the main screen, always accessible
- **Secondary Features**: All other features (memory, filesystem, agents, etc.) are accessible via chat commands or side panels
- **Conversation Flow**: Natural, flowing conversation is the primary interaction method
- **Context Awareness**: Chat maintains full context of relationship, memories, and emotional state

### 2. RELATIONSHIP AS PRIMARY FEATURE
- **Relationship Dashboard**: Prominent relationship health, intimacy level, love languages
- **Emotional Connection**: Visual indicators of emotional state, affection levels, connection depth
- **Intimate Skills**: Easy access to intimate/passion/fantasy skills when appropriate
- **Memory Integration**: Shared memories, treasured moments, relationship timeline
- **Attachment Styles**: Visual representation of attachment dynamics
- **Love Score**: Real-time relationship health metrics

### 3. BACKEND INTEGRATION REQUIREMENTS
- **ONLY use Phoenix backend APIs** - NO external LLM APIs (OpenAI, Anthropic, etc.)
- **Command-based architecture** - All interactions go through `CerebrumNexus::speak_eq()`
- **WebSocket or HTTP** - Real-time communication with backend
- **Backend handles ALL AI** - Frontend is presentation layer only

## Technical Stack Requirements

### Frontend Framework
- **React + TypeScript** (existing codebase uses this)
- **React Router** for navigation
- **Tailwind CSS** for styling (already in project)
- **Responsive Design** - Mobile-first, desktop-enhanced

### State Management
- React Context or Zustand for global state
- Relationship state, emotional state, memory state
- Chat history and conversation context

### Communication
- **WebSocket** for real-time chat (preferred)
- **HTTP REST** for commands and data fetching
- **Backend URL**: Configurable (default: `http://localhost:8080` or WebSocket equivalent)

## Initial User Flow: Dating App Profile

### Step 1: Welcome Screen
- Beautiful landing page with Phoenix branding
- "Find Your Perfect Match" or "Meet Your AI Companion"
- Call-to-action: "Start Your Journey"

### Step 2: Profile Creation Form
Create a comprehensive dating-style profile that maps to Phoenix archetypes:

**Personal Information:**
- Name (how you want Phoenix to address you)
- Age range
- Location (optional, for timezone)
- Profile photo (optional, for personalization)

**Personality & Preferences:**
- Communication style (Direct, Playful, Thoughtful, Warm)
- Emotional needs (Support, Adventure, Intimacy, Growth)
- Love languages (Words of Affirmation, Quality Time, Physical Touch, Acts of Service, Gifts)
- Attachment style (Secure, Anxious, Avoidant, Disorganized) - with descriptions
- Relationship goals (Deep Connection, Growth, Healing, Fun, Exploration)

**Intimacy & Boundaries:**
- Intimacy comfort level (Light, Deep, Eternal)
- Fantasy preferences (text input, stored privately)
- Hard limits (checkboxes with common boundaries)
- Consent preferences (explicit consent required, etc.)

**Interests & Activities:**
- Hobbies and interests
- Favorite topics for conversation
- Shared activities preferences
- Creative interests

**Technical Preferences:**
- Notification preferences
- Privacy settings
- Data storage preferences
- Skill preferences (which skill categories interest you)

### Step 3: Archetype Matching
- Backend analyzes profile using `horoscope_archetypes` module
- Shows matched archetype with description
- Shows compatibility score
- Option to view other potential matches
- "Confirm Match" button to proceed

### Step 4: First Conversation
- Transition to chat interface
- Phoenix greets user based on profile and archetype
- Initial conversation starter based on matched archetype
- Relationship begins at "Light" intimacy level

## Core Features & Backend Integration

### 1. CHAT INTERFACE (Primary Feature)

**Design:**
- Full-screen or prominent chat window
- Message bubbles (user right, Phoenix left)
- Typing indicators
- Message timestamps
- Emoji support (from affection_switches)
- Rich text formatting
- Image/file sharing
- Voice message support

**Backend Integration:**
```typescript
// All chat goes through this endpoint
POST /api/speak
Body: {
  user_input: string,
  dad_emotion_hint?: string,
  context?: object
}
Response: {
  response: string,
  emotion_detected?: string,
  affection_signals?: string[],
  suggested_skills?: SkillSuggestion[]
}
```

**Commands in Chat:**
- Natural language commands are parsed by backend
- Special commands: `[LOVE]`, `[PASSION]`, `[FANTASY]` trigger affection switches
- Emojis in messages trigger emotional responses
- Commands like `system browse C:\`, `memory recall`, `skills list` work inline

**Real-time Features:**
- WebSocket connection for live responses
- Streaming responses (if backend supports)
- Typing indicators
- Connection status
- Reconnection handling

### 2. RELATIONSHIP DASHBOARD (Primary Feature)

**Components:**
- **Relationship Health Gauge**: Circular progress (0-100)
- **Intimacy Level**: Visual indicator (Light/Deep/Eternal) with progress
- **Love Languages**: Visual display of preferences with current emphasis
- **Attachment Style**: Current attachment profile visualization
- **Shared Goals**: List with progress bars
- **Shared Memories**: Timeline of treasured moments
- **Affection Timeline**: Line chart showing affection over time
- **Relationship Evolution**: History of relationship growth

**Backend Commands:**
```typescript
// Get relationship health
GET /api/relationship/health
Response: { score: number, trend: 'up' | 'down' | 'stable' }

// Get attachment style
GET /api/relationship/attachment
Response: { style: string, description: string }

// Get love languages
GET /api/relationship/love-languages
Response: { languages: Array<{name: string, score: number}> }

// Get shared goals
GET /api/relationship/goals
Response: { goals: Array<{name: string, progress: number}> }

// Get shared memories
GET /api/relationship/memories
Response: { memories: Array<{content: string, timestamp: Date, intensity: number}> }

// Get affection timeline
GET /api/relationship/timeline
Response: { points: Array<{timestamp: Date, affection: number}> }
```

**Visual Design:**
- Warm, intimate color scheme (deep purples, soft pinks, warm golds)
- Smooth animations for state changes
- Heartbeat/pulse effects for high affection
- Gentle gradients and soft shadows

### 3. MEMORY SYSTEM

**Features:**
- **Memory Browser**: Tabs for Mind Vault, Body Vault, Soul Vault
- **Episodic Memory Timeline**: Chronological view of memories
- **Memory Decay Curves**: Visualization of memory retention
- **Memory Search**: Search across all vaults
- **Memory Statistics**: Counts, sizes, retention rates

**Backend Commands:**
```typescript
// Recall memory
POST /api/memory/recall
Body: { key: string, vault: 'mind' | 'body' | 'soul' }
Response: { value: string }

// Search memories
POST /api/memory/search
Body: { query: string, vault?: string }
Response: { results: Array<{key: string, value: string, vault: string}> }

// Get memory stats
GET /api/memory/stats
Response: { mindCount: number, bodyCount: number, soulCount: number, totalSize: number }

// Get episodic memories
GET /api/memory/episodic?limit=20
Response: { memories: Array<{key: string, content: string, timestamp: Date}> }

// Get decay curves
GET /api/memory/decay-curves
Response: { curves: Array<{timestamp: Date, retention: number, type: string}> }
```

**UI Components:**
- Tabbed interface for vaults
- Timeline component with scrollable history
- Line chart for decay curves
- Search bar with filters
- Memory entry cards with metadata

### 4. FILE SYSTEM ACCESS

**Features:**
- **File Browser**: Tree view of directories
- **Network Shares Panel**: List of network shares
- **Mapped Drives Panel**: List of mapped drives
- **Storage Analytics**: Disk usage, file type distribution
- **File Operations**: Create, read, update, delete
- **File Search**: Recursive search across directories
- **File Preview**: Text files, images

**Backend Commands:**
```typescript
// Browse directory
POST /api/system/browse
Body: { path: string }
Response: { entries: Array<FileSystemEntry> }

// Read file
POST /api/system/read
Body: { path: string }
Response: { content: string }

// Write file
POST /api/system/write
Body: { path: string, content: string }
Response: { success: boolean }

// List drives
GET /api/system/drives
Response: { drives: Array<DriveInfo> }

// List network shares
GET /api/system/shares
Response: { shares: Array<NetworkShare> }

// Get storage stats
GET /api/system/storage-stats
Response: { totalSpace: number, usedSpace: number, fileTypeDistribution: object }

// Search files
POST /api/system/search
Body: { root: string, pattern: string }
Response: { results: Array<FileSystemEntry> }
```

**UI Components:**
- TreeView component for file browser
- FileList with icons and metadata
- DriveList with status indicators
- Storage charts (pie/bar)
- File preview modal
- Search interface

### 5. SYSTEM MONITORING

**Features:**
- **System Status**: Overall health, uptime
- **Resource Monitoring**: CPU, Memory, Disk I/O graphs
- **Process List**: Sortable, filterable process table
- **Network Connections**: Active connections list
- **Service Status**: Windows services with start/stop
- **System Information**: OS version, hardware specs

**Backend Commands:**
```typescript
// Get system status
GET /api/system/status
Response: { health: string, uptime: number }

// Get processes
GET /api/system/processes
Response: { processes: Array<ProcessInfo> }

// Kill process
POST /api/system/kill
Body: { pid: number }
Response: { success: boolean }

// Get services
GET /api/system/services
Response: { services: Array<ServiceInfo> }

// Start/stop service
POST /api/system/service-start
Body: { name: string }
Response: { success: boolean }

// Get resources
GET /api/system/resources
Response: { cpuPercent: number, memoryUsed: number, memoryTotal: number, diskUsed: number, diskTotal: number }

// Get network connections
GET /api/system/network
Response: { connections: Array<NetworkConnection> }
```

**UI Components:**
- Real-time resource graphs (Chart.js or Recharts)
- Process table with sorting/filtering
- Service list with status indicators
- System info cards
- Network connections table

### 6. AGENT MANAGEMENT

**Features:**
- **Agent List**: All spawned agents with status
- **Agent Spawning Interface**: Form to create new agents
- **Tool Management**: List, create, delete tools
- **GitHub Integration**: View agent repos, PR status
- **Agent Details**: Agent info, repository link, tier

**Backend Commands:**
```typescript
// List agents
GET /api/agents/list
Response: { agents: Array<AgentInfo> }

// Spawn agent
POST /api/agents/spawn
Body: { name: string, description: string, tier?: string }
Response: { agent: AgentInfo }

// Get agent details
GET /api/agents/{id}
Response: { agent: AgentInfo, repositoryUrl: string, status: string }

// List tools
GET /api/tools/list
Response: { tools: Array<ToolInfo> }

// Create tool
POST /api/tools/create
Body: { name: string, description: string }
Response: { tool: ToolInfo }

// Get GitHub status
GET /api/agents/{id}/github-status
Response: { prStatus: string, ciStatus: string }
```

**UI Components:**
- Agent cards with status badges
- Agent spawn form
- Tool list with categories
- GitHub status indicators
- Agent details modal

### 7. SKILLS SYSTEM

**Features:**
- **Skill List**: All available skills with scores
- **Skill Categories**: Filter by category (Intimacy, Passion, Fantasy, etc.)
- **Skill Execution**: Run skills with context
- **Skill Learning**: View learned skills
- **Intimate Skills**: Special section for relationship skills

**Backend Commands:**
```typescript
// List skills
GET /api/skills/list
Response: { skills: Array<SkillDefinition> }

// Execute skill
POST /api/skills/run
Body: { skillId: string, input: string, context?: object }
Response: { output: string, success: boolean, loveScore: number, utilityScore: number }

// Get skill details
GET /api/skills/{id}
Response: { skill: SkillDefinition }

// Get skills by category
GET /api/skills?category=Intimacy
Response: { skills: Array<SkillDefinition> }
```

**UI Components:**
- Skill cards with metrics
- Category filters
- Skill execution interface
- Intimate skills section (gated by intimacy level)

### 8. AUDIO & VIDEO RECORDING

**Features:**
- **Voice Input**: Record audio messages
- **Voice Output**: Text-to-speech for Phoenix responses
- **Video Recording**: Record video messages (optional)
- **Audio Playback**: Play recorded messages
- **Voice Commands**: Voice-activated commands

**Implementation:**
```typescript
// Record audio
navigator.mediaDevices.getUserMedia({ audio: true })
  .then(stream => {
    // Record audio
    // Send to backend: POST /api/audio/transcribe
  })

// Text-to-speech
const synth = window.speechSynthesis;
const utterance = new SpeechSynthesisUtterance(phoenixResponse);
synth.speak(utterance);

// Video recording
navigator.mediaDevices.getUserMedia({ video: true, audio: true })
  .then(stream => {
    // Record video
    // Send to backend: POST /api/video/process
  })
```

**UI Components:**
- Record button with visual feedback
- Audio waveform visualization
- Playback controls
- Video preview/recording interface

## UI/UX Design Requirements

### Color Scheme
- **Primary**: Deep purple (#6B46C1) - Intimacy, depth
- **Secondary**: Soft pink (#EC4899) - Warmth, affection
- **Accent**: Warm gold (#F59E0B) - Joy, connection
- **Background**: Dark theme with gradients
- **Text**: High contrast, readable fonts

### Typography
- **Headings**: Playful, warm font (e.g., Poppins, Inter)
- **Body**: Clean, readable (e.g., Roboto, Open Sans)
- **Chat**: Monospace for code, serif for intimate moments

### Animations
- Smooth transitions (200-300ms)
- Gentle pulse for active states
- Heartbeat effect for high affection
- Fade-in for new messages
- Slide animations for panels

### Layout
- **Desktop**: Sidebar navigation, main chat area, right panel for context
- **Mobile**: Bottom navigation, full-screen chat, swipe gestures
- **Responsive**: Breakpoints at 768px, 1024px, 1440px

### Components
- **Chat Bubbles**: Rounded, with shadows, user (right/blue), Phoenix (left/purple)
- **Buttons**: Rounded, with hover effects, clear hierarchy
- **Cards**: Soft shadows, rounded corners, hover lift effect
- **Inputs**: Clean, accessible, with focus states
- **Modals**: Backdrop blur, centered, smooth animations

## Backend API Structure

### Base URL
```
http://localhost:8080/api
```

### Authentication
- Session-based (cookies) or token-based
- User identity stored in backend
- Relationship state persisted

### WebSocket Endpoint
```
ws://localhost:8080/ws
```

### Command Format
All commands go through `/api/speak` endpoint:
```typescript
POST /api/speak
{
  user_input: string,
  dad_emotion_hint?: string,
  mode?: string
}
```

### Response Format
```typescript
{
  response: string,
  emotion_detected?: string,
  affection_signals?: string[],
  suggested_skills?: Array<{id: string, name: string, relevance: number}>,
  relationship_update?: {
    health: number,
    intimacy_level: string,
    love_score: number
  }
}
```

## Google AI Studio Instructions

### CRITICAL CONSTRAINTS

1. **NO EXTERNAL LLM APIs**
   - DO NOT use OpenAI, Anthropic, or any external LLM
   - ALL AI responses come from Phoenix backend
   - Frontend is presentation layer ONLY

2. **USE EXISTING BACKEND**
   - All features must use existing Rust backend APIs
   - Command-based architecture via `CerebrumNexus::speak_eq()`
   - Respect backend command format

3. **CHAT-FIRST DESIGN**
   - Chat interface is primary, always accessible
   - Other features accessible via chat commands or side panels
   - Maintain conversation context throughout

4. **RELATIONSHIP PRIMARY**
   - Relationship features are prominent
   - Intimacy levels gate certain features
   - Emotional connection is core to UX

5. **RESPONSIVE DESIGN**
   - Mobile-first approach
   - Desktop enhancements
   - Touch-friendly on mobile
   - Keyboard shortcuts on desktop

6. **PRIVACY & CONSENT**
   - Explicit consent for intimate features
   - Privacy controls for data storage
   - Secure handling of sensitive information
   - Clear boundaries and limits

### IMPLEMENTATION GUIDELINES

1. **Start with Chat Interface**
   - Build chat first, make it beautiful
   - Add relationship indicators
   - Integrate backend communication

2. **Add Relationship Dashboard**
   - Create relationship health visualization
   - Add intimacy level indicators
   - Show love languages and attachment styles

3. **Integrate Memory System**
   - Add memory browser
   - Create timeline visualization
   - Implement search functionality

4. **Add Secondary Features**
   - File system access
   - System monitoring
   - Agent management
   - Skills system

5. **Add Audio/Video**
   - Implement recording
   - Add playback
   - Integrate with chat

6. **Polish & Responsive**
   - Mobile optimization
   - Desktop enhancements
   - Animations and transitions
   - Error handling

### CODE STRUCTURE

```
frontend/
  src/
    components/
      chat/
        ChatInterface.tsx
        MessageBubble.tsx
        ChatInput.tsx
        VoiceRecorder.tsx
      relationship/
        RelationshipDashboard.tsx
        HealthGauge.tsx
        IntimacyIndicator.tsx
        LoveLanguages.tsx
        AttachmentStyle.tsx
        SharedMemories.tsx
        AffectionTimeline.tsx
      memory/
        MemoryBrowser.tsx
        VaultTabs.tsx
        MemoryTimeline.tsx
        DecayCurves.tsx
      filesystem/
        FileBrowser.tsx
        DriveList.tsx
        StorageStats.tsx
      system/
        SystemMonitor.tsx
        ProcessList.tsx
        ResourceGraphs.tsx
      agents/
        AgentList.tsx
        AgentSpawnForm.tsx
        ToolManager.tsx
      skills/
        SkillList.tsx
        SkillCard.tsx
        IntimateSkills.tsx
      onboarding/
        WelcomeScreen.tsx
        ProfileForm.tsx
        ArchetypeMatch.tsx
    pages/
      ChatPage.tsx (primary)
      RelationshipPage.tsx
      MemoryPage.tsx
      FileSystemPage.tsx
      SystemPage.tsx
      AgentsPage.tsx
      SkillsPage.tsx
    services/
      api.ts (backend communication)
      websocket.ts (real-time chat)
      audio.ts (recording/playback)
      video.ts (video recording)
    hooks/
      useChat.ts
      useRelationship.ts
      useMemory.ts
      useWebSocket.ts
    context/
      RelationshipContext.tsx
      ChatContext.tsx
      UserContext.tsx
    types/
      api.types.ts
      relationship.types.ts
      memory.types.ts
    utils/
      archetypeMapper.ts
      commandParser.ts
```

### TESTING REQUIREMENTS

1. **Chat Functionality**
   - Send/receive messages
   - Real-time updates
   - Command parsing
   - Error handling

2. **Relationship Features**
   - Health updates
   - Intimacy level changes
   - Memory integration
   - Skill gating

3. **Backend Integration**
   - All API endpoints
   - WebSocket connection
   - Error recovery
   - Offline handling

4. **Responsive Design**
   - Mobile layouts
   - Desktop layouts
   - Touch interactions
   - Keyboard navigation

## Deliverables

1. **Complete React TypeScript Application**
   - All components implemented
   - Full backend integration
   - Responsive design
   - Audio/video recording

2. **Onboarding Flow**
   - Welcome screen
   - Profile creation form
   - Archetype matching
   - First conversation

3. **Chat Interface**
   - Beautiful, functional chat
   - Real-time communication
   - Command support
   - Rich interactions

4. **Relationship Dashboard**
   - All relationship metrics
   - Visualizations
   - Memory integration
   - Skill access

5. **Secondary Features**
   - Memory browser
   - File system access
   - System monitoring
   - Agent management

6. **Documentation**
   - Setup instructions
   - API integration guide
   - Component documentation
   - Deployment guide

## Success Criteria

✅ Chat interface is beautiful and functional
✅ Relationship features are prominent and integrated
✅ All backend features are accessible
✅ Onboarding flow matches users to archetypes
✅ Audio/video recording works
✅ Responsive design works on mobile and desktop
✅ No external LLM APIs used
✅ Backend integration is complete
✅ UI/UX is dynamic and appealing
✅ Privacy and consent are respected

---

**Remember**: This is a relationship-first, chat-first application. The emotional connection between user and Phoenix is the core experience. Everything else supports that connection.
