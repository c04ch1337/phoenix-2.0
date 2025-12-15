# Google AI Studio Prompt: Orchestration Page with GitHub Integration

## Project Context

You are creating/updating the **Orchestration Page** (`frontend/pages/OrchestrationPage.tsx`) to provide comprehensive management of Agents, Tools, and Skills with full GitHub integration. All creations must follow a strict GitHub-first workflow: **Create → Push to GitHub → Pull Down → Use/Assign**.

## Current State

**Existing System:**
- `agent_spawner` module handles agent creation and GitHub pushing
- `limb_extension_grafts` module manages tools
- `skill_system` module manages skills with folder-based loading
- `evolution_pipeline` module handles GitHub enforcement (PR workflow)
- Frontend has `SkillsPage.tsx` but no dedicated Orchestration page
- Backend uses command-based architecture: `CerebrumNexus::speak_eq()`

**GitHub Repositories:**
- **Skills**: `https://github.com/c04ch1337/phoenix-core-skills.git`
- **Agents**: `https://github.com/c04ch1337/phoenix-core-agents/`
- **Tools**: `https://github.com/c04ch1337/phoenix-core-tools/`

## Task: Create/Update Orchestration Page with GitHub Integration

### Step 1: Create OrchestrationPage Component

**New File**: `frontend/pages/OrchestrationPage.tsx`

**Page Structure:**
1. **Header Section** - "Orchestration Hub" with GitHub status indicator
2. **Three Main Tabs/Sections**:
   - **Agents Tab** - Agent creation, listing, GitHub sync
   - **Tools Tab** - Tool creation, listing, GitHub sync
   - **Skills Tab** - Skill creation, listing, assignment to agents, GitHub sync
3. **GitHub Status Panel** - Shows sync status, pending PRs, last pull/push times
4. **Activity Log** - Shows recent GitHub operations (push, pull, commit)

### Step 2: GitHub-First Workflow Implementation

**Core Principle**: All creations (Agents, Tools, Skills) MUST follow this workflow:

```
1. Create locally (generate code/JSON)
2. Commit to local git repository
3. Push to GitHub (create PR if MANDATE_GITHUB_CI=true)
4. Wait for CI + Human Approval (if required)
5. Pull down merged code
6. Add to local registry/repo
7. Use/Assign
```

**Workflow States:**
- `draft` - Created locally, not yet pushed
- `pushed` - Pushed to GitHub, PR created
- `pending_approval` - Waiting for CI/human approval
- `merged` - Merged to main, ready to pull
- `pulled` - Pulled down locally
- `active` - In use/assigned

### Step 3: Agents Section

**UI Components:**

#### Agent List View
```typescript
interface Agent {
  id: string;                    // UUID
  name: string;
  description: string;
  tier: 'Free' | 'Paid' | 'Enterprise';
  githubRepo: string;            // "c04ch1337/phoenix-core-agents/agent-name"
  githubUrl: string;              // Full GitHub URL
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;                 // PR URL if pending
  createdAt: string;
  lastSynced?: string;
}
```

**Features:**
- **List Agents**: Display all agents with status badges
- **Create Agent**: Form to create new agent
  - Name, Description, Tier selection
  - Generate code via LLM
  - Show preview before pushing
- **Push to GitHub**: Button to push agent to `phoenix-core-agents` repo
- **Pull from GitHub**: Button to pull latest from GitHub
- **View on GitHub**: Link to agent repository
- **Delete Agent**: Remove from local registry (with confirmation)

**Backend Commands:**
```typescript
// List agents
'agents list'

// Create agent (generates code, creates locally)
'agents create | name=... | description=... | tier=Free'

// Push agent to GitHub
'agents push | id=... | repo=phoenix-core-agents'

// Pull agent from GitHub
'agents pull | name=... | repo=phoenix-core-agents'

// Get agent status
'agents status | id=...'

// Delete agent
'agents delete | id=...'
```

### Step 4: Tools Section

**UI Components:**

#### Tool List View
```typescript
interface Tool {
  id: string;                    // UUID
  name: string;
  description: string;
  category: string;               // e.g., "utility", "automation", "analysis"
  githubRepo: string;             // "c04ch1337/phoenix-core-tools/tool-name"
  githubUrl: string;
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;
  createdAt: string;
  lastSynced?: string;
}
```

**Features:**
- **List Tools**: Display all tools with status badges
- **Create Tool**: Form to create new tool
  - Name, Description, Category
  - Generate code via LLM
  - Show preview before pushing
- **Push to GitHub**: Button to push tool to `phoenix-core-tools` repo
- **Pull from GitHub**: Button to pull latest from GitHub
- **View on GitHub**: Link to tool repository
- **Delete Tool**: Remove from local registry

**Backend Commands:**
```typescript
// List tools
'tools list'

// Create tool (generates code, creates locally)
'tools create | name=... | description=... | category=...'

// Push tool to GitHub
'tools push | id=... | repo=phoenix-core-tools'

// Pull tool from GitHub
'tools pull | name=... | repo=phoenix-core-tools'

// Get tool status
'tools status | id=...'

// Delete tool
'tools delete | id=...'
```

### Step 5: Skills Section

**UI Components:**

#### Skill List View
```typescript
interface Skill {
  id: string;                    // UUID
  name: string;
  category: string;               // "Intimacy", "Passion", "Fantasy", etc.
  description: string;
  version: string;
  githubRepo: string;             // "c04ch1337/phoenix-core-skills/skill-name.json"
  githubUrl: string;
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;
  assignedToAgents: string[];     // Agent IDs this skill is assigned to
  createdAt: string;
  lastSynced?: string;
}
```

**Features:**
- **List Skills**: Display all skills with category badges
- **Create Skill**: Form to create new skill
  - Name, Category, Description
  - Steps editor (JSON format)
  - Relationship context settings
  - Generate JSON structure
  - Show preview before pushing
- **Push to GitHub**: Button to push skill JSON to `phoenix-core-skills` repo
- **Pull from GitHub**: Button to pull latest from GitHub
- **Assign to Agent**: Multi-select to assign skills to agents
- **View on GitHub**: Link to skill file in repository
- **Delete Skill**: Remove from local registry

**Backend Commands:**
```typescript
// List skills
'skills list'

// Create skill (generates JSON, creates locally)
'skills create | name=... | category=... | description=...'

// Push skill to GitHub
'skills push | id=... | repo=phoenix-core-skills'

// Pull skill from GitHub
'skills pull | name=... | repo=phoenix-core-skills'

// Assign skill to agent
'skills assign | skillId=... | agentId=...'

// Unassign skill from agent
'skills unassign | skillId=... | agentId=...'

// Get skill status
'skills status | id=...'

// Delete skill
'skills delete | id=...'
```

### Step 6: GitHub Integration Panel

**Status Display:**
```typescript
interface GitHubStatus {
  skillsRepo: {
    url: string;
    lastPulled: string;
    lastPushed: string;
    pendingPRs: number;
    synced: boolean;
  };
  agentsRepo: {
    url: string;
    lastPulled: string;
    lastPushed: string;
    pendingPRs: number;
    synced: boolean;
  };
  toolsRepo: {
    url: string;
    lastPulled: string;
    lastPushed: string;
    pendingPRs: number;
    synced: boolean;
  };
}
```

**Features:**
- **Sync Status**: Show last pull/push times for each repo
- **Pending PRs**: Display count and links to pending PRs
- **Sync All**: Button to pull latest from all repos
- **GitHub Connection**: Show connection status (authenticated/not authenticated)

**Backend Commands:**
```typescript
// Get GitHub status
'github status'

// Pull all from GitHub
'github pull-all'

// Pull specific repo
'github pull | repo=phoenix-core-skills'

// List pending PRs
'github prs | repo=...'

// Sync status for all repos
'github sync-status'
```

### Step 7: Backend Command Handlers

**Add to `cerebrum_nexus/src/lib.rs`:**

#### Agent Commands
```rust
async fn handle_agent_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();

    // List agents
    if lower == "agents list" {
        // Query local agent registry
        // Return JSON list of agents
    }

    // Create agent
    if lower.starts_with("agents create") {
        // Parse: agents create | name=... | description=... | tier=...
        // Generate code via LLM
        // Create local agent structure
        // Return agent metadata
    }

    // Push agent to GitHub
    if lower.starts_with("agents push") {
        // Parse: agents push | id=... | repo=phoenix-core-agents
        // Use agent_spawner to push to GitHub
        // Create PR if MANDATE_GITHUB_CI=true
        // Return PR URL or success message
    }

    // Pull agent from GitHub
    if lower.starts_with("agents pull") {
        // Parse: agents pull | name=... | repo=phoenix-core-agents
        // Clone/pull from GitHub
        // Add to local registry
        // Return success message
    }

    // Get agent status
    if lower.starts_with("agents status") {
        // Parse: agents status | id=...
        // Check local status, GitHub PR status
        // Return status JSON
    }

    // Delete agent
    if lower.starts_with("agents delete") {
        // Parse: agents delete | id=...
        // Remove from local registry
        // Return success message
    }

    None
}
```

#### Tool Commands
```rust
async fn handle_tool_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();

    // List tools
    if lower == "tools list" {
        // Query limb_extension_grafts
        // Return JSON list of tools
    }

    // Create tool
    if lower.starts_with("tools create") {
        // Parse: tools create | name=... | description=... | category=...
        // Generate code via LLM
        // Create local tool structure
        // Return tool metadata
    }

    // Push tool to GitHub
    if lower.starts_with("tools push") {
        // Parse: tools push | id=... | repo=phoenix-core-tools
        // Use evolution_pipeline to push to GitHub
        // Create PR if MANDATE_GITHUB_CI=true
        // Return PR URL or success message
    }

    // Pull tool from GitHub
    if lower.starts_with("tools pull") {
        // Parse: tools pull | name=... | repo=phoenix-core-tools
        // Clone/pull from GitHub
        // Add to local tool registry
        // Return success message
    }

    // Get tool status
    if lower.starts_with("tools status") {
        // Parse: tools status | id=...
        // Check local status, GitHub PR status
        // Return status JSON
    }

    // Delete tool
    if lower.starts_with("tools delete") {
        // Parse: tools delete | id=...
        // Remove from local registry
        // Return success message
    }

    None
}
```

#### Skill Commands (Extended)
```rust
async fn handle_skill_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();

    // Existing skill commands (list, run, etc.)
    // ... existing implementation ...

    // Create skill
    if lower.starts_with("skills create") {
        // Parse: skills create | name=... | category=... | description=...
        // Generate skill JSON structure
        // Create local skill file
        // Return skill metadata
    }

    // Push skill to GitHub
    if lower.starts_with("skills push") {
        // Parse: skills push | id=... | repo=phoenix-core-skills
        // Read skill JSON from local
        // Push to GitHub repo (create PR if required)
        // Return PR URL or success message
    }

    // Pull skill from GitHub
    if lower.starts_with("skills pull") {
        // Parse: skills pull | name=... | repo=phoenix-core-skills
        // Clone/pull from GitHub
        // Load skill JSON into skill_system
        // Return success message
    }

    // Assign skill to agent
    if lower.starts_with("skills assign") {
        // Parse: skills assign | skillId=... | agentId=...
        // Link skill to agent in registry
        // Update agent's skills.json
        // Return success message
    }

    // Unassign skill from agent
    if lower.starts_with("skills unassign") {
        // Parse: skills unassign | skillId=... | agentId=...
        // Unlink skill from agent
        // Return success message
    }

    // Get skill status
    if lower.starts_with("skills status") {
        // Parse: skills status | id=...
        // Check local status, GitHub PR status
        // Return status JSON
    }

    // Delete skill
    if lower.starts_with("skills delete") {
        // Parse: skills delete | id=...
        // Remove from local registry
        // Return success message
    }

    None
}
```

#### GitHub Commands
```rust
async fn handle_github_command(&self, user_input: &str) -> Option<String> {
    let trimmed = user_input.trim();
    let lower = trimmed.to_ascii_lowercase();

    // Get GitHub status
    if lower == "github status" {
        // Check sync status for all three repos
        // Return status JSON
    }

    // Pull all repos
    if lower == "github pull-all" {
        // Pull latest from phoenix-core-skills
        // Pull latest from phoenix-core-agents
        // Pull latest from phoenix-core-tools
        // Return success message
    }

    // Pull specific repo
    if lower.starts_with("github pull") {
        // Parse: github pull | repo=phoenix-core-skills
        // Pull latest from specified repo
        // Return success message
    }

    // List pending PRs
    if lower.starts_with("github prs") {
        // Parse: github prs | repo=...
        // Query GitHub API for open PRs
        // Return PR list JSON
    }

    // Sync status
    if lower == "github sync-status" {
        // Check last pull/push times for all repos
        // Return sync status JSON
    }

    None
}
```

### Step 8: Frontend Service Methods

**Update `frontend/services/mockBackend.ts` (or create real API service):**

```typescript
// Agent methods
async listAgents(): Promise<Agent[]> {
  const response = await this.execute('agents list');
  return JSON.parse(response);
}

async createAgent(name: string, description: string, tier: string): Promise<Agent> {
  const command = `agents create | name=${name} | description=${description} | tier=${tier}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pushAgentToGitHub(agentId: string): Promise<{ prUrl?: string; success: boolean }> {
  const command = `agents push | id=${agentId} | repo=phoenix-core-agents`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pullAgentFromGitHub(agentName: string): Promise<{ success: boolean; message: string }> {
  const command = `agents pull | name=${agentName} | repo=phoenix-core-agents`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async getAgentStatus(agentId: string): Promise<AgentStatus> {
  const command = `agents status | id=${agentId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async deleteAgent(agentId: string): Promise<{ success: boolean }> {
  const command = `agents delete | id=${agentId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

// Tool methods
async listTools(): Promise<Tool[]> {
  const response = await this.execute('tools list');
  return JSON.parse(response);
}

async createTool(name: string, description: string, category: string): Promise<Tool> {
  const command = `tools create | name=${name} | description=${description} | category=${category}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pushToolToGitHub(toolId: string): Promise<{ prUrl?: string; success: boolean }> {
  const command = `tools push | id=${toolId} | repo=phoenix-core-tools`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pullToolFromGitHub(toolName: string): Promise<{ success: boolean; message: string }> {
  const command = `tools pull | name=${toolName} | repo=phoenix-core-tools`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async getToolStatus(toolId: string): Promise<ToolStatus> {
  const command = `tools status | id=${toolId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async deleteTool(toolId: string): Promise<{ success: boolean }> {
  const command = `tools delete | id=${toolId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

// Skill methods (extended)
async listSkills(): Promise<Skill[]> {
  const response = await this.execute('skills list');
  return JSON.parse(response);
}

async createSkill(name: string, category: string, description: string, steps: any[]): Promise<Skill> {
  const skillJson = JSON.stringify({ name, category, description, steps });
  const command = `skills create | skill=${encodeURIComponent(skillJson)}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pushSkillToGitHub(skillId: string): Promise<{ prUrl?: string; success: boolean }> {
  const command = `skills push | id=${skillId} | repo=phoenix-core-skills`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async pullSkillFromGitHub(skillName: string): Promise<{ success: boolean; message: string }> {
  const command = `skills pull | name=${skillName} | repo=phoenix-core-skills`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async assignSkillToAgent(skillId: string, agentId: string): Promise<{ success: boolean }> {
  const command = `skills assign | skillId=${skillId} | agentId=${agentId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async unassignSkillFromAgent(skillId: string, agentId: string): Promise<{ success: boolean }> {
  const command = `skills unassign | skillId=${skillId} | agentId=${agentId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async getSkillStatus(skillId: string): Promise<SkillStatus> {
  const command = `skills status | id=${skillId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async deleteSkill(skillId: string): Promise<{ success: boolean }> {
  const command = `skills delete | id=${skillId}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

// GitHub methods
async getGitHubStatus(): Promise<GitHubStatus> {
  const response = await this.execute('github status');
  return JSON.parse(response);
}

async pullAllFromGitHub(): Promise<{ success: boolean; message: string }> {
  const response = await this.execute('github pull-all');
  return JSON.parse(response);
}

async pullRepoFromGitHub(repo: string): Promise<{ success: boolean; message: string }> {
  const command = `github pull | repo=${repo}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async listPendingPRs(repo: string): Promise<PendingPR[]> {
  const command = `github prs | repo=${repo}`;
  const response = await this.execute(command);
  return JSON.parse(response);
}

async getSyncStatus(): Promise<SyncStatus> {
  const response = await this.execute('github sync-status');
  return JSON.parse(response);
}
```

### Step 9: TypeScript Types

**Add to `frontend/types.ts`:**

```typescript
// Orchestration types
export interface Agent {
  id: string;
  name: string;
  description: string;
  tier: 'Free' | 'Paid' | 'Enterprise';
  githubRepo: string;
  githubUrl: string;
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;
  createdAt: string;
  lastSynced?: string;
}

export interface Tool {
  id: string;
  name: string;
  description: string;
  category: string;
  githubRepo: string;
  githubUrl: string;
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;
  createdAt: string;
  lastSynced?: string;
}

export interface Skill {
  id: string;
  name: string;
  category: string;
  description: string;
  version: string;
  githubRepo: string;
  githubUrl: string;
  status: 'draft' | 'pushed' | 'pending_approval' | 'merged' | 'pulled' | 'active';
  prUrl?: string;
  assignedToAgents: string[];
  createdAt: string;
  lastSynced?: string;
}

export interface AgentStatus {
  id: string;
  status: string;
  localExists: boolean;
  githubExists: boolean;
  prUrl?: string;
  prStatus?: 'open' | 'merged' | 'closed';
  lastSynced?: string;
}

export interface ToolStatus {
  id: string;
  status: string;
  localExists: boolean;
  githubExists: boolean;
  prUrl?: string;
  prStatus?: 'open' | 'merged' | 'closed';
  lastSynced?: string;
}

export interface SkillStatus {
  id: string;
  status: string;
  localExists: boolean;
  githubExists: boolean;
  prUrl?: string;
  prStatus?: 'open' | 'merged' | 'closed';
  lastSynced?: string;
  assignedToAgents: string[];
}

export interface GitHubStatus {
  skillsRepo: RepoStatus;
  agentsRepo: RepoStatus;
  toolsRepo: RepoStatus;
}

export interface RepoStatus {
  url: string;
  lastPulled: string;
  lastPushed: string;
  pendingPRs: number;
  synced: boolean;
}

export interface PendingPR {
  number: number;
  title: string;
  url: string;
  status: 'open' | 'merged' | 'closed';
  createdAt: string;
}

export interface SyncStatus {
  skillsRepo: SyncInfo;
  agentsRepo: SyncInfo;
  toolsRepo: SyncInfo;
}

export interface SyncInfo {
  lastPulled: string;
  lastPushed: string;
  needsSync: boolean;
}
```

### Step 10: UI/UX Design

**Design Principles:**
- **GitHub-First Visual Cues**: Status badges clearly show GitHub workflow state
- **Three-Column Layout**: Agents | Tools | Skills tabs
- **GitHub Status Panel**: Always visible at top, shows sync status
- **Activity Feed**: Shows recent GitHub operations
- **Color Coding**:
  - 🟡 Draft (yellow) - Created locally, not pushed
  - 🔵 Pushed (blue) - On GitHub, PR created
  - 🟠 Pending (orange) - Waiting for approval
  - 🟢 Merged (green) - Merged, ready to pull
  - ⚪ Pulled (gray) - Pulled down locally
  - ✅ Active (green) - In use/assigned

**Component Structure:**
```typescript
<OrchestrationPage>
  <GitHubStatusPanel />
  <Tabs>
    <Tab label="Agents">
      <AgentList />
      <CreateAgentForm />
      <AgentActions />
    </Tab>
    <Tab label="Tools">
      <ToolList />
      <CreateToolForm />
      <ToolActions />
    </Tab>
    <Tab label="Skills">
      <SkillList />
      <CreateSkillForm />
      <SkillAssignmentPanel />
      <SkillActions />
    </Tab>
  </Tabs>
  <ActivityLog />
</OrchestrationPage>
```

### Step 11: Backend GitHub Integration Functions

**Add to `cerebrum_nexus/src/lib.rs`:**

#### Agent Push to GitHub
```rust
async fn push_agent_to_github(&self, agent_id: &str) -> Result<String, String> {
    // 1. Get agent from local registry
    // 2. Use agent_spawner to push to phoenix-core-agents repo
    // 3. Create PR if MANDATE_GITHUB_CI=true
    // 4. Return PR URL or success message
    
    let spawner = self.reproductive_system.lock().await;
    // Implementation using agent_spawner::spawn_agent logic
    // But instead of creating new repo, push to existing phoenix-core-agents repo
}
```

#### Tool Push to GitHub
```rust
async fn push_tool_to_github(&self, tool_id: &str) -> Result<String, String> {
    // 1. Get tool from limb_extension_grafts
    // 2. Use evolution_pipeline to push to phoenix-core-tools repo
    // 3. Create PR if MANDATE_GITHUB_CI=true
    // 4. Return PR URL or success message
}
```

#### Skill Push to GitHub
```rust
async fn push_skill_to_github(&self, skill_id: &str) -> Result<String, String> {
    // 1. Get skill from skill_system
    // 2. Serialize to JSON
    // 3. Push to phoenix-core-skills repo (create PR if required)
    // 4. Return PR URL or success message
}
```

#### Pull from GitHub Functions
```rust
async fn pull_agent_from_github(&self, agent_name: &str) -> Result<String, String> {
    // 1. Clone/pull from phoenix-core-agents repo
    // 2. Load agent code
    // 3. Add to local agent registry
    // 4. Return success message
}

async fn pull_tool_from_github(&self, tool_name: &str) -> Result<String, String> {
    // 1. Clone/pull from phoenix-core-tools repo
    // 2. Load tool code
    // 3. Add to local tool registry
    // 4. Return success message
}

async fn pull_skill_from_github(&self, skill_name: &str) -> Result<String, String> {
    // 1. Clone/pull from phoenix-core-skills repo
    // 2. Load skill JSON
    // 3. Add to skill_system library
    // 4. Return success message
}
```

### Step 12: GitHub Repository Structure

**Expected Repository Structures:**

#### phoenix-core-agents/
```
phoenix-core-agents/
  ├── agent-name-1/
  │   ├── src/
  │   │   ├── main.rs
  │   │   ├── generated.rs
  │   │   └── template_agent.rs
  │   ├── Cargo.toml
  │   ├── README.md
  │   └── agent.json
  ├── agent-name-2/
  │   └── ...
  └── README.md
```

#### phoenix-core-tools/
```
phoenix-core-tools/
  ├── tool-name-1/
  │   ├── src/
  │   │   └── lib.rs
  │   ├── Cargo.toml
  │   └── README.md
  ├── tool-name-2/
  │   └── ...
  └── README.md
```

#### phoenix-core-skills/
```
phoenix-core-skills/
  ├── intimate/
  │   ├── passionate_connection.json
  │   └── ...
  ├── passion/
  │   ├── desire_expression.json
  │   └── ...
  ├── fantasy/
  │   ├── roleplay_scenario.json
  │   └── ...
  └── README.md
```

### Step 13: Integration with Existing Systems

**Agent Spawner Integration:**
- Use `agent_spawner::AgentSpawner` for agent creation
- Modify to support pushing to existing `phoenix-core-agents` repo instead of creating new repos
- Use `evolution_pipeline::GitHubEnforcer` for PR workflow

**Tool System Integration:**
- Use `limb_extension_grafts::LimbExtensionGrafts` for tool management
- Integrate with `evolution_pipeline` for GitHub pushing
- Support both procedural and code-based tools

**Skill System Integration:**
- Use `skill_system::SkillSystem` for skill management
- Leverage `skill_system::folder_loader` for loading from GitHub
- Support skill assignment to agents via agent registry

### Step 14: Route Configuration

**Update `frontend/App.tsx`:**

```typescript
import OrchestrationPage from './pages/OrchestrationPage';

// Add to PageRoute enum in types.ts
export enum PageRoute {
  // ... existing routes ...
  ORCHESTRATION = '/orchestration',
}

// Add route
<Route path={PageRoute.ORCHESTRATION} element={
  <div className="flex flex-col h-full">
    <OrchestrationPage onRunCommand={handleCommand} />
    <ResultPanel lastResponse={lastResponse} />
  </div>
} />
```

**Update `frontend/components/Layout.tsx`** to include Orchestration link in navigation.

### Step 15: Error Handling & User Feedback

**Error States:**
- **GitHub Connection Failed**: Show error, suggest checking GITHUB_PAT
- **PR Creation Failed**: Show error, suggest checking permissions
- **Pull Failed**: Show error, suggest checking repo access
- **Merge Conflict**: Show conflict resolution UI
- **CI Failed**: Show CI error details, link to GitHub Actions

**Success Feedback:**
- **Pushed Successfully**: Show success toast, PR link
- **Pulled Successfully**: Show success toast, list of new items
- **Assigned Successfully**: Show confirmation

### Step 16: GitHub Workflow Enforcement

**Enforcement Rules:**
1. **MANDATE_GITHUB_CI=true**: All creations must go through PR workflow
2. **REQUIRE_HUMAN_PR_APPROVAL=true**: All PRs require human approval
3. **AUTO_MERGE_ON_APPROVAL=false**: Manual merge required (recommended)
4. **PR_APPROVAL_TIMEOUT_HOURS=24**: Timeout for waiting for approval

**Workflow States:**
- User creates Agent/Tool/Skill → Status: `draft`
- User clicks "Push to GitHub" → Status: `pushed`, PR created
- System polls for PR status → Status: `pending_approval`
- Human approves PR → Status: `merged`
- User clicks "Pull from GitHub" → Status: `pulled`
- User activates/assigns → Status: `active`

### Step 17: Skill Assignment to Agents

**Assignment Flow:**
1. User selects skill(s) from skill list
2. User selects agent(s) from agent list
3. Click "Assign Skills to Agents"
4. Backend updates agent's `skills.json` file
5. Push updated agent to GitHub
6. Pull updated agent back down
7. Skill is now available to that agent

**Backend Implementation:**
```rust
async fn assign_skill_to_agent(&self, skill_id: &str, agent_id: &str) -> Result<String, String> {
    // 1. Get skill from skill_system
    // 2. Get agent from agent registry
    // 3. Update agent's skills.json to include skill_id
    // 4. Save agent metadata
    // 5. Return success
}
```

### Step 18: Complete Example Workflow

**Example: Creating and Deploying an Agent**

1. **User fills out form:**
   - Name: "data-processor"
   - Description: "Processes CSV files and generates reports"
   - Tier: Free

2. **Click "Create Agent":**
   - Backend generates Rust code via LLM
   - Creates local agent structure
   - Status: `draft`

3. **Click "Push to GitHub":**
   - Backend commits agent code
   - Creates branch: `phoenix-creation/agent-data-processor-{uuid}`
   - Pushes to `phoenix-core-agents` repo
   - Creates PR: "Spawn agent: data-processor"
   - Status: `pushed`, PR URL shown

4. **Wait for Approval:**
   - System polls PR status
   - CI runs (if enabled)
   - Status: `pending_approval`

5. **Human Approves PR:**
   - PR is merged to main
   - Status: `merged`

6. **Click "Pull from GitHub":**
   - Backend pulls latest from `phoenix-core-agents`
   - Loads agent into local registry
   - Status: `pulled`

7. **Agent is Ready:**
   - Agent appears in active agents list
   - Can be used in orchestration
   - Status: `active`

### Step 19: UI Component Examples

**Agent Card Component:**
```typescript
const AgentCard: React.FC<{ agent: Agent }> = ({ agent }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'draft': return 'bg-yellow-900/30 border-yellow-700';
      case 'pushed': return 'bg-blue-900/30 border-blue-700';
      case 'pending_approval': return 'bg-orange-900/30 border-orange-700';
      case 'merged': return 'bg-green-900/30 border-green-700';
      case 'pulled': return 'bg-gray-900/30 border-gray-700';
      case 'active': return 'bg-green-900/30 border-green-500';
      default: return 'bg-gray-900/30 border-gray-700';
    }
  };

  return (
    <div className={`p-4 rounded-lg border ${getStatusColor(agent.status)}`}>
      <div className="flex justify-between items-start mb-2">
        <h3 className="text-white font-semibold">{agent.name}</h3>
        <span className="text-xs px-2 py-1 rounded bg-gray-800 text-gray-300">
          {agent.status}
        </span>
      </div>
      <p className="text-gray-400 text-sm mb-3">{agent.description}</p>
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-500">Tier: {agent.tier}</span>
        <div className="flex gap-2">
          {agent.status === 'draft' && (
            <button onClick={() => pushToGitHub(agent.id)}>
              Push to GitHub
            </button>
          )}
          {agent.status === 'merged' && (
            <button onClick={() => pullFromGitHub(agent.name)}>
              Pull from GitHub
            </button>
          )}
          {agent.prUrl && (
            <a href={agent.prUrl} target="_blank" rel="noopener noreferrer">
              View PR
            </a>
          )}
          <a href={agent.githubUrl} target="_blank" rel="noopener noreferrer">
            View on GitHub
          </a>
        </div>
      </div>
    </div>
  );
};
```

**GitHub Status Panel:**
```typescript
const GitHubStatusPanel: React.FC<{ status: GitHubStatus }> = ({ status }) => {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg p-4 mb-6">
      <h3 className="text-lg font-semibold text-white mb-4">GitHub Sync Status</h3>
      <div className="grid grid-cols-3 gap-4">
        <RepoStatusCard 
          label="Skills" 
          repo={status.skillsRepo}
          repoUrl="https://github.com/c04ch1337/phoenix-core-skills"
        />
        <RepoStatusCard 
          label="Agents" 
          repo={status.agentsRepo}
          repoUrl="https://github.com/c04ch1337/phoenix-core-agents"
        />
        <RepoStatusCard 
          label="Tools" 
          repo={status.toolsRepo}
          repoUrl="https://github.com/c04ch1337/phoenix-core-tools"
        />
      </div>
      <button 
        onClick={() => pullAllFromGitHub()}
        className="mt-4 w-full bg-blue-900/40 hover:bg-blue-900/60 text-blue-200 py-2 rounded-lg"
      >
        Sync All Repositories
      </button>
    </div>
  );
};
```

### Step 20: Backend Implementation Details

**GitHub Repository Paths:**
- Skills: `c04ch1337/phoenix-core-skills`
- Agents: `c04ch1337/phoenix-core-agents`
- Tools: `c04ch1337/phoenix-core-tools`

**Environment Variables Required:**
```bash
GITHUB_PAT=your_token
GITHUB_USERNAME=c04ch1337
GITHUB_REPO_OWNER=c04ch1337
GITHUB_AGENTS_REPO=phoenix-core-agents
GITHUB_TOOLS_REPO=phoenix-core-tools
# Note: Skills repo uses phoenix-core-skills (no separate env var needed)
MANDATE_GITHUB_CI=true
REQUIRE_HUMAN_PR_APPROVAL=true
AUTO_MERGE_ON_APPROVAL=false
PR_APPROVAL_TIMEOUT_HOURS=24
```

**GitHub API Integration:**
- Use `octocrab` crate for GitHub API calls
- Use `git2` crate for git operations
- Use `evolution_pipeline::GitHubEnforcer` for PR workflow

### Step 21: Complete Command Reference

**Agent Commands:**
```
agents list
agents create | name=... | description=... | tier=Free|Paid|Enterprise
agents push | id=... | repo=phoenix-core-agents
agents pull | name=... | repo=phoenix-core-agents
agents status | id=...
agents delete | id=...
```

**Tool Commands:**
```
tools list
tools create | name=... | description=... | category=...
tools push | id=... | repo=phoenix-core-tools
tools pull | name=... | repo=phoenix-core-tools
tools status | id=...
tools delete | id=...
```

**Skill Commands (Extended):**
```
skills list
skills create | name=... | category=... | description=... | steps=...
skills push | id=... | repo=phoenix-core-skills
skills pull | name=... | repo=phoenix-core-skills
skills assign | skillId=... | agentId=...
skills unassign | skillId=... | agentId=...
skills status | id=...
skills delete | id=...
```

**GitHub Commands:**
```
github status
github pull-all
github pull | repo=phoenix-core-skills|phoenix-core-agents|phoenix-core-tools
github prs | repo=...
github sync-status
```

### Step 22: Response Format Specifications

**Agent List Response:**
```json
{
  "type": "agents_list",
  "agents": [
    {
      "id": "uuid",
      "name": "data-processor",
      "description": "...",
      "tier": "Free",
      "githubRepo": "c04ch1337/phoenix-core-agents/data-processor",
      "githubUrl": "https://github.com/c04ch1337/phoenix-core-agents/tree/main/data-processor",
      "status": "pulled",
      "createdAt": "2024-01-01T00:00:00Z",
      "lastSynced": "2024-01-01T12:00:00Z"
    }
  ]
}
```

**Push Response:**
```json
{
  "type": "push_result",
  "success": true,
  "prUrl": "https://github.com/c04ch1337/phoenix-core-agents/pull/123",
  "message": "Agent pushed to GitHub. PR #123 created."
}
```

**Pull Response:**
```json
{
  "type": "pull_result",
  "success": true,
  "message": "Pulled 3 new agents from GitHub.",
  "items": ["agent-1", "agent-2", "agent-3"]
}
```

**GitHub Status Response:**
```json
{
  "type": "github_status",
  "skillsRepo": {
    "url": "https://github.com/c04ch1337/phoenix-core-skills",
    "lastPulled": "2024-01-01T12:00:00Z",
    "lastPushed": "2024-01-01T11:00:00Z",
    "pendingPRs": 2,
    "synced": true
  },
  "agentsRepo": {
    "url": "https://github.com/c04ch1337/phoenix-core-agents",
    "lastPulled": "2024-01-01T12:00:00Z",
    "lastPushed": "2024-01-01T11:00:00Z",
    "pendingPRs": 1,
    "synced": true
  },
  "toolsRepo": {
    "url": "https://github.com/c04ch1337/phoenix-core-tools",
    "lastPulled": "2024-01-01T12:00:00Z",
    "lastPushed": "2024-01-01T11:00:00Z",
    "pendingPRs": 0,
    "synced": true
  }
}
```

### Step 23: Integration with Existing Pages

**Update SkillsPage.tsx:**
- Add link to Orchestration page for GitHub management
- Show GitHub sync status for skills
- Add "Push to GitHub" button for individual skills

**Update ApprovalsPage.tsx:**
- Show PRs from all three repos (skills, agents, tools)
- Unified approval queue

### Step 24: Local Storage & State Management

**Persist State:**
- Store agent/tool/skill lists in local state
- Cache GitHub status (refresh every 5 minutes)
- Store last sync times
- Remember user preferences (default repo, auto-sync, etc.)

### Step 25: Error Recovery

**Handle Common Errors:**
- **GitHub Authentication Failed**: Show setup instructions
- **Repository Not Found**: Offer to create repo
- **Merge Conflicts**: Show conflict resolution UI
- **Network Errors**: Retry with exponential backoff
- **Permission Denied**: Show permission requirements

---

## Summary

This prompt creates a comprehensive Orchestration page that:

1. ✅ **Manages Agents** - Create, push to GitHub, pull down, use
2. ✅ **Manages Tools** - Create, push to GitHub, pull down, use
3. ✅ **Manages Skills** - Create, push to GitHub, pull down, assign to agents
4. ✅ **GitHub Integration** - Full push/pull workflow for all three repos
5. ✅ **PR Workflow** - Enforces GitHub-first creation with PR approval
6. ✅ **Status Tracking** - Visual status indicators for workflow states
7. ✅ **Skill Assignment** - Assign skills to agents with GitHub sync
8. ✅ **Unified Interface** - Single page for all orchestration needs

**Key GitHub Repositories:**
- Skills: `https://github.com/c04ch1337/phoenix-core-skills.git`
- Agents: `https://github.com/c04ch1337/phoenix-core-agents/`
- Tools: `https://github.com/c04ch1337/phoenix-core-tools/`

**Workflow Enforcement:**
- All creations must be pushed to GitHub before use
- PR workflow enforced when `MANDATE_GITHUB_CI=true`
- Human approval required when `REQUIRE_HUMAN_PR_APPROVAL=true`
- Pull down after merge before activation

**Remember**: This is a GitHub-first system. Nothing is activated until it's been pushed to GitHub, approved (if required), merged, and pulled back down. This ensures version control, collaboration, and safety.
