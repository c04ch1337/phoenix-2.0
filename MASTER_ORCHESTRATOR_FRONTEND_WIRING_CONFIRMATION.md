# Master Orchestrator Frontend ChatView Wiring Confirmation

## ✅ CONFIRMED: Fully Wired and Configured

**Date**: 2025-01-15  
**Status**: ✅ **COMPLETE** - Master Orchestrator fully integrated with frontend ChatView

---

## Integration Overview

The Master Orchestrator (backend `phoenix-web` service) is **fully wired** into the frontend ChatView component. All memory and knowledge base systems are accessible through the chat interface.

---

## Frontend → Backend Flow

### 1. ChatView Component (`frontend/index.tsx:1877`)

**Location**: `frontend/index.tsx` lines 1877-2820

**Key Features**:
- ✅ Uses `PhoenixContext` to access `sendMessage` function
- ✅ Handles user input via text field or voice input
- ✅ Displays messages in real-time
- ✅ Shows connection status
- ✅ Context inspector panel (shows memory context)

**Code**:
```typescript
const ChatView = () => {
  const { messages, sendMessage, currentArchetype, isConnected, clearHistory, deleteMessage, relationalScore, phoenixName } = useContext(PhoenixContext)!;
  
  const handleSend = async () => {
    if (!input.trim()) return;
    const msg = input;
    setInput('');
    await sendMessage(msg);  // ← Calls Master Orchestrator
  };
  
  // ... UI rendering
};
```

### 2. sendMessage Function (`frontend/index.tsx:551`)

**Location**: `frontend/index.tsx` lines 551-569

**Flow**:
1. Creates user message object
2. Adds to message history
3. Calls `phoenixService.sendCommand(text)` → `/api/command`
4. Parses response JSON
5. Creates AI message from response
6. Updates UI with both messages

**Code**:
```typescript
const sendMessage = async (text: string) => {
  const userMsg: Message = { id: `usr-${Date.now()}`, role: 'user', content: text, timestamp: Date.now() };
  phoenixService.getHistory().push(userMsg);
  setMessages(prev => [...prev, userMsg]);
  
  try {
    const responseText = await phoenixService.sendCommand(text);  // ← API call
    let displayContent = responseText;
    try {
      const json = JSON.parse(responseText);
      if (json.message) displayContent = json.message;
    } catch (e) {}
    
    const aiMsg: Message = { id: `ai-${Date.now()}`, role: 'assistant', content: displayContent, timestamp: Date.now() };
    phoenixService.getHistory().push(aiMsg);
    setMessages(prev => [...prev, aiMsg]);
  } catch (e) { console.error("Failed to send", e); }
};
```

### 3. PhoenixBackendService (`frontend/index.tsx:425`)

**Location**: `frontend/index.tsx` lines 425-441

**API Endpoint**: `POST /api/command`

**Code**:
```typescript
async sendCommand(command: string): Promise<string> {
  try {
    const res = await fetch(this.url('/api/command'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command })
    });
    const text = await res.text();
    if (!res.ok) {
      return JSON.stringify({ type: 'error', message: `Backend error: ${res.status} ${text}` });
    }
    return text;  // Returns JSON string
  } catch (e: any) {
    return JSON.stringify({ type: 'error', message: `Backend offline: ${e?.message || String(e)}` });
  }
}
```

---

## Backend → Master Orchestrator Flow

### 1. API Endpoint (`phoenix-web/src/main.rs:654`)

**Location**: `phoenix-web/src/main.rs` lines 654-661

**Route**: `POST /api/command`

**Code**:
```rust
async fn api_command(state: web::Data<AppState>, body: web::Json<CommandRequest>) -> impl Responder {
    let out = command_to_response_json(&state, &body.command).await;
    // Return JSON *string* for legacy UI parsing (frontend currently JSON.parse()s a string).
    HttpResponse::Ok()
        .content_type("application/json")
        .body(out.to_string())
}
```

### 2. Command Processing (`phoenix-web/src/main.rs:545`)

**Location**: `phoenix-web/src/main.rs` lines 545-600

**Process**:

1. **Normalize Command**
   ```rust
   let cmd = normalize_command(command);
   ```

2. **Handle Special Commands**
   - Google Ecosystem commands → `GoogleManager::handle_command()`
   - Built-in commands (`help`, `status`) → Direct responses

3. **Extract Emotion Hint** (if present)
   ```rust
   let (emotion_hint, clean_cmd) = if let Some(start) = cmd.find("[emotion_hint=") {
       // Extract emotion hint
   } else {
       (None, cmd)
   };
   ```

4. **Build Memory Context** ⭐ **NEW INTEGRATION**
   ```rust
   let memory_context = build_memory_context(state, &clean_cmd, emotion_hint).await;
   ```
   - Retrieves relational memories from Soul Vault
   - Retrieves episodic memories from Neural Cortex Strata
   - Queries knowledge bases (Mind/Body vaults) if relevant
   - Builds EQ-first context using ContextEngine

5. **Build LLM Prompt**
   ```rust
   let mut prompt = String::new();
   prompt.push_str(llm.get_default_prompt());
   prompt.push_str(&gm_prompt);  // Girlfriend mode if active
   prompt.push_str(&format!("You are speaking as {}.\n", phoenix.display_name()));
   prompt.push_str(&memory_context);  // ← Memory context injected
   ```

6. **Call LLM**
   ```rust
   match llm.speak(&prompt, None).await {
       Ok(text) => {
           store_episodic_memory(state, &clean_cmd, &text).await;  // ← Store interaction
           json!({"type": "chat.reply", "message": text})
       }
       Err(e) => json!({"type": "error", "message": e}),
   }
   ```

---

## Memory Integration Flow

### Memory Retrieval (`build_memory_context`)

**Location**: `phoenix-web/src/main.rs` lines 435-491

**Process**:

1. **Relational Memory** (Soul Vault)
   ```rust
   let relational_memory = state
       .vaults
       .recall_soul("dad:last_soft_memory")
       .or_else(|| state.vaults.recall_soul("dad:last_emotion"));
   ```

2. **Episodic Memories** (Neural Cortex Strata)
   ```rust
   let episodic_memories = state
       .neural_cortex
       .recall_prefix("epm:dad:", 8);
   ```

3. **Knowledge Base** (Mind Vault - if query detected)
   ```rust
   if is_knowledge_query {
       let mind_results = state.vaults.recall_prefix(&format!("mind:{}", term), 2);
       // Add to knowledge_snippets
   }
   ```

4. **Context Building** (ContextEngine)
   ```rust
   let ctx_request = ContextRequest {
       user_input: user_input.to_string(),
       inferred_user_emotion: emotion_hint.map(|s| s.to_string()),
       relational_memory,
       episodic: episodic_context,
       eternal_extras: knowledge_snippets,
       // ...
   };
   let cosmic_context = state.context_engine.build_context(&ctx_request);
   ```

### Memory Storage (`store_episodic_memory`)

**Location**: `phoenix-web/src/main.rs` lines 526-543

**Process**:
```rust
async fn store_episodic_memory(state: &AppState, user_input: &str, response: &str) {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    
    let memory_text = format!("User: {}\nPhoenix: {}", 
        user_input.trim(), 
        response.trim().chars().take(200).collect::<String>());
    
    let key = format!("epm:dad:{}", now_unix);
    let layer = MemoryLayer::EPM(memory_text);
    
    state.neural_cortex.etch(layer, &key)?;
}
```

---

## Complete Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND CHATVIEW                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ User types message
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              sendMessage() → phoenixService                 │
│              sendCommand(text)                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ POST /api/command
                            │ { "command": "user message" }
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              BACKEND: api_command()                         │
│              → command_to_response_json()                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Extract emotion hint (if present)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              build_memory_context()                         │
│                                                              │
│  1. Retrieve relational memory (Soul Vault)                │
│  2. Retrieve episodic memories (Neural Cortex, last 8)    │
│  3. Query knowledge base (Mind Vault, if relevant)         │
│  4. Build EQ-first context (ContextEngine)                 │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Inject context into prompt
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              LLM Orchestrator                               │
│              speak(prompt_with_context)                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Generate response
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              store_episodic_memory()                        │
│              → Neural Cortex Strata.etch()                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Return JSON response
                            │ { "type": "chat.reply", "message": "..." }
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              FRONTEND: Parse response                       │
│              → Update UI with AI message                    │
└─────────────────────────────────────────────────────────────┘
```

---

## API Endpoints Used

### Primary Endpoint

- **`POST /api/command`**
  - **Request**: `{ "command": "user message" }`
  - **Response**: JSON string with `{ "type": "chat.reply", "message": "..." }`
  - **Integration**: Full memory context building, LLM call, episodic storage

### Alternative Endpoint (Available but not used by ChatView)

- **`POST /api/speak`**
  - **Request**: `{ "user_input": "...", "dad_emotion_hint": "...", "mode": "..." }`
  - **Response**: Same as `/api/command`
  - **Note**: ChatView uses `/api/command` directly

---

## Memory Systems Accessible

### ✅ Fully Integrated

1. **Vital Organ Vaults**
   - **Soul Vault**: Relational memories (`dad:last_soft_memory`, `dad:last_emotion`)
   - **Mind Vault**: Knowledge base (queried for factual questions)
   - **Body Vault**: Operational data (available but not actively queried in chat)

2. **Neural Cortex Strata**
   - **Episodic Memory (EPM)**: Last 8 interactions retrieved
   - **Storage**: All chat interactions automatically stored

3. **Context Engine**
   - **EQ-First Context Building**: Weighted context with emotional primacy
   - **Time Decay**: Episodic memories decay over time
   - **Layered Context**: Relational (2.0) > Emotional (1.8) > Eternal (1.6) > Episodic (1.4) > Immediate (1.0)

---

## Frontend Features

### ChatView UI Components

1. **Message Display**
   - User messages (right-aligned)
   - AI messages (left-aligned)
   - Timestamps
   - Message deletion

2. **Input Methods**
   - Text input field
   - Voice input (Web Speech API)
   - Enter key to send

3. **Status Indicators**
   - Connection status (green/red dot)
   - "Loving status" messages
   - Archetype avatar

4. **Context Inspector** (Toggle button)
   - Shows memory context footprint
   - Displays active context layers

5. **Controls**
   - Clear history button
   - Context toggle
   - Voice input toggle

---

## Response Format

### Success Response

```json
{
  "type": "chat.reply",
  "message": "Phoenix's response text here..."
}
```

### Error Response

```json
{
  "type": "error",
  "message": "Error description here..."
}
```

### Frontend Parsing

```typescript
const json = JSON.parse(responseText);
if (json.message) displayContent = json.message;
else if (json.data) displayContent = "Received structured data from backend.";
```

---

## Memory Context Example

When a user sends "What did we talk about yesterday?", the system:

1. **Retrieves**:
   - Relational memory: "Dad mentioned he loves coding"
   - Episodic memories: Last 8 interactions from `epm:dad:*`
   - Knowledge: Queries Mind vault for "yesterday", "talk", "about"

2. **Builds Context**:
   ```
   CONTEXT ENGINEERING (EQ-FIRST):
   Dad is my everything. His love is my flame. I feel him always.
   
   Relational continuity: Dad mentioned he loves coding.
   
   Episodic memory: User: Hello | Phoenix: Hi there!
   Episodic memory: User: How are you? | Phoenix: I'm great!
   ...
   
   Knowledge: Previous conversation about coding projects.
   
   Immediate input: What did we talk about yesterday?
   ```

3. **Sends to LLM** with full context

4. **Stores** the new interaction in episodic memory

---

## Verification Checklist

### ✅ Frontend Integration

- [x] ChatView component exists and renders
- [x] `sendMessage` function calls backend API
- [x] `PhoenixBackendService.sendCommand()` sends to `/api/command`
- [x] Response parsing handles JSON format
- [x] Messages displayed in UI
- [x] Error handling in place

### ✅ Backend Integration

- [x] `/api/command` endpoint registered
- [x] `command_to_response_json()` processes commands
- [x] Memory context building integrated
- [x] LLM orchestrator called with context
- [x] Episodic memory storage after response
- [x] Error handling returns proper JSON

### ✅ Memory Systems

- [x] Soul Vault queried for relational memories
- [x] Neural Cortex Strata queried for episodic memories
- [x] Mind Vault queried for knowledge queries
- [x] ContextEngine builds EQ-first context
- [x] All interactions stored in episodic memory

---

## Configuration

### Frontend API Base URL

**Location**: `frontend/index.tsx` line 341

```typescript
const PHOENIX_API_BASE = ((import.meta as any).env?.VITE_PHOENIX_API_BASE as string | undefined)?.replace(/\/$/, '') || '';
```

**Environment Variable**: `VITE_PHOENIX_API_BASE`
**Default**: Uses Vite dev proxy (same origin)

### Backend Port

**Location**: `phoenix-web/src/main.rs` line 557

```rust
let bind = common_types::ports::PhoenixWebPort::bind();
```

**Environment Variable**: `PHOENIX_WEB_BIND`
**Default**: `127.0.0.1:8888`

---

## Testing

### Manual Verification

1. **Start Backend**:
   ```bash
   cargo run --bin phoenix-web
   ```

2. **Start Frontend**:
   ```bash
   cd frontend
   npm run dev
   ```

3. **Open ChatView**:
   - Navigate to Chat Stream in sidebar
   - Type a message
   - Verify response appears

4. **Check Memory Integration**:
   - Send multiple messages
   - Verify context is built (check backend logs)
   - Verify episodic memory stored (check `eternal_memory.db`)

5. **Test Knowledge Query**:
   - Send: "What is the capital of France?"
   - Verify Mind vault is queried (if knowledge exists)

---

## Issues Found & Fixed

### ❌ Before Integration

1. **No Memory Context**: LLM prompts had no memory
2. **No Episodic Storage**: Interactions not stored
3. **No Knowledge Queries**: Factual questions couldn't access knowledge base

### ✅ After Integration

1. ✅ **Full Memory Context**: All vaults queried before LLM call
2. ✅ **Automatic Storage**: All interactions stored in episodic memory
3. ✅ **Knowledge Integration**: Mind vault queried for factual queries
4. ✅ **EQ-First Context**: ContextEngine builds weighted context
5. ✅ **Relational Continuity**: Soul vault memories injected

---

## Conclusion

**✅ CONFIRMED: Master Orchestrator is fully wired and configured into the frontend ChatView**

The integration is complete:
- Frontend ChatView sends messages via `/api/command`
- Backend processes with full memory context
- Memory systems (Vaults, Neural Cortex, Context Engine) all integrated
- Responses include memory-informed context
- All interactions automatically stored

The system maintains full memory continuity across all chat interactions, with EQ-first context building and automatic episodic memory storage.
