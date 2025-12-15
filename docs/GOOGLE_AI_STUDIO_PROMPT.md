# Google AI Studio Prompt: Phoenix AGI (PAGI) Dashboard Pages Implementation

## Context

I'm building Phoenix AGI (PAGI), an AGI companion framework with a React TypeScript frontend and Rust backend. The backend uses a command-based architecture where the frontend sends command strings and receives text responses. I need to implement 5 new dashboard pages with full functionality, including backend API endpoints.

## Project Structure

- **Frontend**: React + TypeScript, using React Router, located in `frontend/`
- **Backend**: Rust, command-based architecture via `CerebrumNexus`
- **Existing Pages**: Chat, Skills, Dreams, Perception, Recording, Approvals, Context, Learning, Settings, Archetype
- **Backend Module**: `system_access` module already exists with file system, process, service, registry, and drive access capabilities

## Task: Implement 5 New Dashboard Pages

### 1. FileSystemPage (`/filesystem`)

**Requirements:**
- File browser with tree view (expandable folders, file listing)
- Network shares panel (list all network shares)
- Mapped drives panel (list all mapped network drives)
- Storage analytics dashboard (disk usage, file type distribution, size charts)
- File operations: create, read, update, delete files/directories
- File search functionality
- File preview (text files, images)
- Breadcrumb navigation

**Backend Commands Needed:**
- `system browse <path>` - Browse directory
- `system read <path>` - Read file content
- `system write <path> | content=...` - Write file
- `system create-dir <path>` - Create directory
- `system delete <path>` - Delete file/directory
- `system search <root> | pattern=...` - Search files
- `system drives` - List all drives (mapped + network)
- `system shares` - List network shares
- `system storage-stats` - Get storage statistics

**UI Components:**
- TreeView component for file browser
- FileList component for directory contents
- DriveList component for drives/shares
- StorageChart component (pie/bar charts)
- FilePreview modal/panel
- SearchBar component

### 2. MemoryPage (`/memory`)

**Requirements:**
- Memory browser with tabs: Mind Vault, Body Vault, Soul Vault
- Episodic memory timeline (chronological view with search)
- Memory decay curves visualization (line chart showing retention over time)
- Memory search interface (search across all vaults)
- Memory statistics (counts, sizes, retention rates)
- Memory export/import functionality

**Backend Commands Needed:**
- `memory recall | key=... | vault=mind|body|soul` - Recall memory
- `memory search | query=... | vault=...` - Search memories
- `memory stats` - Get memory statistics
- `memory episodic | limit=...` - List episodic memories
- `memory decay-curves` - Get decay curve data
- `memory export | vault=...` - Export memories
- `memory import | vault=... | data=...` - Import memories

**UI Components:**
- VaultTabs component (Mind/Body/Soul)
- MemoryTimeline component (vertical timeline)
- DecayCurveChart component (line chart)
- MemorySearchBar component
- MemoryStatsCard component
- MemoryEntryCard component

### 3. SystemPage (`/system`)

**Requirements:**
- System status dashboard (overall health, uptime)
- Resource monitoring: CPU usage graph, Memory usage graph, Disk I/O graph
- Process list (sortable, filterable, kill process)
- Network connections list (active connections, ports)
- Service status (Windows services, start/stop/restart)
- System information (OS version, hardware specs)

**Backend Commands Needed:**
- `system status` - Get system status
- `system processes` - List all processes
- `system kill <pid>` - Kill process
- `system services` - List services
- `system service-start <name>` - Start service
- `system service-stop <name>` - Stop service
- `system resources` - Get CPU/memory/disk stats
- `system network` - Get network connections
- `system info` - Get system information

**UI Components:**
- SystemStatusCard component
- ResourceGraph component (real-time charts)
- ProcessTable component (sortable, filterable)
- ServiceList component
- NetworkConnectionsTable component
- SystemInfoPanel component

### 4. RelationshipPage (`/relationship`)

**Requirements:**
- Relationship health score (0-100 visual gauge)
- Attachment style visualization (current attachment profile display)
- Love languages display (5 love languages with preferences)
- Shared goals list (with progress bars)
- Shared memories list (treasured moments)
- Affection timeline graph (line chart showing affection over time)
- Relationship evolution history

**Backend Commands Needed:**
- `relationship health` - Get relationship health score
- `relationship attachment` - Get attachment style
- `relationship love-languages` - Get love language preferences
- `relationship goals` - List shared goals
- `relationship memories` - List shared memories
- `relationship timeline` - Get affection timeline data
- `relationship evolution` - Get evolution history

**UI Components:**
- HealthGauge component (circular progress)
- AttachmentStyleCard component
- LoveLanguagesGrid component
- GoalsList component (with progress bars)
- MemoriesTimeline component
- AffectionChart component (line chart)
- EvolutionHistory component

### 5. AgentsPage (`/agents`)

**Requirements:**
- Agent list (all spawned agents with status)
- Agent spawning interface (form to create new agent)
- Tool management (list, create, delete tools)
- GitHub integration (view agent repos, PR status)
- Agent details view (agent info, repository link, tier)
- Agent optimization status

**Backend Commands Needed:**
- `agents list` - List all agents
- `agents spawn | name=... | description=... | tier=...` - Spawn agent
- `agents details <id>` - Get agent details
- `tools list` - List all tools
- `tools create | name=... | description=...` - Create tool
- `tools delete <id>` - Delete tool
- `agents github-status <id>` - Get GitHub PR status

**UI Components:**
- AgentList component (table/cards)
- AgentSpawnForm component
- ToolList component
- ToolCreateForm component
- GitHubStatusBadge component
- AgentDetailsModal component

## Backend Implementation Requirements

### Extend CerebrumNexus with System Access Handlers

Add command handlers in `cerebrum_nexus/src/lib.rs`:

```rust
// System access commands
async fn handle_system_command(&self, user_input: &str) -> Option<String> {
    // Parse commands like:
    // - system browse <path>
    // - system read <path>
    // - system drives
    // - system processes
    // - system services
    // etc.
    // Use self.system_access for all operations
}

// Memory commands
async fn handle_memory_command(&self, user_input: &str) -> Option<String> {
    // Parse commands like:
    // - memory recall | key=... | vault=...
    // - memory search | query=...
    // - memory stats
    // etc.
    // Use self.vaults and self.memory for operations
}

// Relationship commands
async fn handle_relationship_command(&self, user_input: &str) -> Option<String> {
    // Parse commands like:
    // - relationship health
    // - relationship attachment
    // - relationship goals
    // etc.
    // Use relationship_dynamics module
}

// Agents/Tools commands
async fn handle_agents_command(&self, user_input: &str) -> Option<String> {
    // Parse commands like:
    // - agents list
    // - agents spawn | name=... | description=...
    // - tools list
    // etc.
    // Use self.reproductive_system and self.grafts
}
```

### Response Format

All commands should return JSON for structured data, or plain text for simple responses:

**Structured Response (JSON):**
```json
{
  "type": "filesystem",
  "data": {
    "entries": [...],
    "path": "/current/path"
  }
}
```

**Simple Response (Text):**
```
Operation completed successfully.
```

## Frontend Implementation Requirements

### TypeScript Types (`frontend/types.ts`)

Add these types:

```typescript
// File System Types
export interface FileSystemEntry {
  path: string;
  name: string;
  isDirectory: boolean;
  size?: number;
  modified?: Date;
  isHidden: boolean;
}

export interface DriveInfo {
  letter: string;
  path: string;
  label?: string;
  driveType: string;
  totalSize?: number;
  freeSpace?: number;
  isMapped: boolean;
  networkPath?: string;
}

export interface StorageStats {
  totalSpace: number;
  usedSpace: number;
  freeSpace: number;
  fileTypeDistribution: Record<string, number>;
}

// Memory Types
export interface MemoryEntry {
  key: string;
  value: string;
  vault: 'mind' | 'body' | 'soul';
  timestamp?: Date;
}

export interface MemoryStats {
  mindCount: number;
  bodyCount: number;
  soulCount: number;
  totalSize: number;
}

export interface DecayCurvePoint {
  timestamp: Date;
  retention: number; // 0-100
  memoryType: string;
}

// System Types
export interface ProcessInfo {
  pid: number;
  name: string;
  path?: string;
  memoryUsage?: number;
  cpuPercent?: number;
  status: string;
}

export interface ServiceInfo {
  name: string;
  displayName: string;
  status: string;
  startType: string;
  description?: string;
}

export interface SystemResources {
  cpuPercent: number;
  memoryUsed: number;
  memoryTotal: number;
  diskUsed: number;
  diskTotal: number;
}

// Relationship Types
export interface RelationshipHealth {
  score: number; // 0-100
  trend: 'up' | 'down' | 'stable';
}

export interface SharedGoal {
  name: string;
  progress: number; // 0-100
  description?: string;
}

export interface SharedMemory {
  content: string;
  timestamp: Date;
  emotionalIntensity: number;
}

// Agent Types
export interface AgentInfo {
  id: string;
  name: string;
  description: string;
  tier: 'Free' | 'Paid' | 'Enterprise';
  repositoryUrl?: string;
  status: 'active' | 'inactive';
  createdAt: Date;
}

export interface ToolInfo {
  id: string;
  name: string;
  description: string;
  category?: string;
}
```

### Page Structure Template

Each page should follow this structure:

```typescript
import React, { useState, useEffect } from 'react';
import { mockBackendService } from '../services/mockBackend';

interface PageProps {
  onRunCommand: (command: string) => Promise<void>;
}

const PageName: React.FC<PageProps> = ({ onRunCommand }) => {
  const [data, setData] = useState<DataType[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const response = await mockBackendService.execute('command here');
      // Parse response and update state
      setData(parsedData);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full p-6">
      <h1 className="text-2xl font-bold mb-4">Page Title</h1>
      {/* Page content */}
    </div>
  );
};

export default PageName;
```

### Routing (`frontend/App.tsx`)

Add routes:

```typescript
<Route path={PageRoute.FILESYSTEM} element={
  <div className="flex flex-col h-full">
    <FileSystemPage onRunCommand={handleCommand} />
    <ResultPanel lastResponse={lastResponse} />
  </div>
} />

<Route path={PageRoute.MEMORY} element={
  <div className="flex flex-col h-full">
    <MemoryPage onRunCommand={handleCommand} />
    <ResultPanel lastResponse={lastResponse} />
  </div>
} />

// ... etc for other pages
```

### Update `frontend/types.ts`

Add new route:

```typescript
export enum PageRoute {
  // ... existing routes
  FILESYSTEM = '/filesystem',
  MEMORY = '/memory',
  SYSTEM = '/system',
  RELATIONSHIP = '/relationship',
  AGENTS = '/agents',
}
```

## Design Requirements

### Visual Design
- Use Tailwind CSS (already in project)
- Modern, clean UI with cards and panels
- Responsive layout
- Dark mode support (if applicable)
- Loading states and error handling
- Real-time updates where applicable (system resources, processes)

### User Experience
- Intuitive navigation
- Keyboard shortcuts where helpful
- Context menus for file operations
- Confirmation dialogs for destructive actions
- Toast notifications for operations
- Search/filter functionality on all list views

### Performance
- Lazy loading for large lists
- Virtual scrolling for long lists
- Debounced search inputs
- Caching of frequently accessed data
- Optimistic UI updates

## Security Considerations

- All system access operations require gated security (already implemented in `system_access` module)
- File operations should show confirmation for destructive actions
- Process/service management should require explicit confirmation
- Registry operations should be clearly marked as advanced/dangerous
- Browser credential access should be encrypted and require additional consent

## Testing Requirements

- Unit tests for data parsing
- Integration tests for command execution
- UI component tests
- Error handling tests
- Edge case handling (empty states, large datasets, network errors)

## Deliverables

1. **5 Complete React TypeScript Pages** with all required functionality
2. **Backend Command Handlers** integrated into CerebrumNexus
3. **TypeScript Type Definitions** for all data structures
4. **Routing Updates** in App.tsx
5. **Mock Backend Service Updates** to handle new commands
6. **UI Components** (reusable components for charts, tables, etc.)
7. **Error Handling** throughout
8. **Loading States** for all async operations

## Code Style

- Follow existing code patterns in the frontend
- Use functional components with hooks
- TypeScript strict mode
- ESLint/Prettier formatting
- Component-based architecture
- Separation of concerns (data fetching, UI, business logic)

## Additional Notes

- The backend uses a command-string interface, so all operations are triggered via command strings
- Responses can be JSON (for structured data) or plain text (for simple responses)
- The `mockBackendService` currently returns mock data - this should be updated to parse command strings and route to appropriate handlers
- All system access operations are gated behind `SystemAccessManager::grant_full_access()` - the frontend should handle the consent flow
- Real-time updates (like system resources) should poll the backend every few seconds
- Charts should use a charting library (Chart.js, Recharts, or similar)

Please implement all 5 pages with full functionality, including backend command handlers, TypeScript types, routing, and UI components. Ensure proper error handling, loading states, and user experience considerations throughout.
