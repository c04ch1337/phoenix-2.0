# Master Orchestrator Code Analysis Integration

## Overview

The Master Orchestrator now has **Full-Control and Unlimited Access** for code analysis. This provides deep semantic understanding, not just definition listing.

## Capabilities

### Full Access Features

1. **Deep Semantic Analysis**
   - Understands code purpose, intent, and context
   - Uses LLM for full semantic understanding
   - Analyzes patterns, algorithms, and data structures
   - Identifies security and performance considerations

2. **Full File System Access**
   - Can read any file, anywhere on the system
   - No restrictions on file paths
   - Can analyze entire codebases recursively

3. **Comprehensive Code Understanding**
   - Not just definition names - full context understanding
   - Cross-file dependency analysis
   - Code flow and execution path analysis
   - Quality metrics and code smells detection

## Usage

### In Cerebrum Nexus

Add to `cerebrum_nexus/src/lib.rs`:

```rust
use code_analysis::MasterOrchestratorCodeAnalysis;

// In CerebrumNexus struct
pub struct CerebrumNexus {
    // ... existing fields ...
    code_analyzer: Option<MasterOrchestratorCodeAnalysis>,
}

// In initialization
impl CerebrumNexus {
    pub fn awaken() -> Result<Self, String> {
        // ... existing initialization ...
        
        let code_analyzer = if let Some(llm) = llm_orchestrator.clone() {
            Some(MasterOrchestratorCodeAnalysis::new_with_llm(llm))
        } else {
            Some(MasterOrchestratorCodeAnalysis::new())
        };
        
        Ok(Self {
            // ... existing fields ...
            code_analyzer,
        })
    }
}
```

### Command Handler

Add to `speak_eq()` routing:

```rust
// Code analysis commands
if let Some(msg) = self.handle_code_analysis_command(user_input).await {
    return Ok(msg);
}
```

### Command Implementation

```rust
impl CerebrumNexus {
    async fn handle_code_analysis_command(&self, user_input: &str) -> Option<String> {
        let analyzer = self.code_analyzer.as_ref()?;
        
        // Parse commands
        if let Some(rest) = user_input.strip_prefix("code analyze ") {
            let path = rest.trim();
            match analyzer.analyze_file(Path::new(path)).await {
                Ok(analysis) => {
                    if let Ok(json) = serde_json::to_string_pretty(&analysis) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if let Some(rest) = user_input.strip_prefix("code list ") {
            let path = rest.trim();
            match analyzer.list_definitions(Path::new(path)).await {
                Ok(defs) => {
                    if let Ok(json) = serde_json::to_string_pretty(&defs) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if let Some(rest) = user_input.strip_prefix("code semantic ") {
            let path = rest.trim();
            match analyzer.deep_semantic_analysis(Path::new(path)).await {
                Ok(semantics) => {
                    if let Ok(json) = serde_json::to_string_pretty(&semantics) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if let Some(rest) = user_input.strip_prefix("code intent ") {
            let path = rest.trim();
            match analyzer.analyze_intent(Path::new(path)).await {
                Ok(intent) => {
                    if let Ok(json) = serde_json::to_string_pretty(&intent) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if let Some(rest) = user_input.strip_prefix("code dependencies ") {
            let path = rest.trim();
            match analyzer.analyze_dependencies(Path::new(path)).await {
                Ok(deps) => {
                    if let Ok(json) = serde_json::to_string_pretty(&deps) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if let Some(rest) = user_input.strip_prefix("code quality ") {
            let path = rest.trim();
            match analyzer.quality_metrics(Path::new(path)).await {
                Ok(quality) => {
                    if let Ok(json) = serde_json::to_string_pretty(&quality) {
                        return Some(json);
                    }
                }
                Err(e) => return Some(format!("Error: {}", e)),
            }
        } else if user_input == "code analyze help" || user_input == "code help" {
            return Some(
                "Code Analysis Commands (Master Orchestrator - Full Access):\n\
                - code analyze <path> - Full code analysis with deep semantic understanding\n\
                - code list <path> - List definition names only (partial access)\n\
                - code semantic <path> - Deep semantic analysis (purpose, patterns, algorithms)\n\
                - code intent <path> - Analyze code intent and purpose\n\
                - code dependencies <path> - Analyze dependencies\n\
                - code quality <path> - Get quality metrics\n\
                - code codebase <path> - Analyze entire codebase recursively\n\
                \n\
                Master Orchestrator has unlimited access - can analyze any file, anywhere."
                    .to_string(),
            );
        }
        
        None
    }
}
```

## Command Examples

### Full Analysis
```
code analyze C:\Users\JAMEYMILNER\AppData\Local\phoenix-2.0\cerebrum_nexus\src\lib.rs
```

### Definition List (Partial Access)
```
code list C:\Users\JAMEYMILNER\AppData\Local\phoenix-2.0\cerebrum_nexus\src\lib.rs
```

### Deep Semantic Analysis
```
code semantic C:\Users\JAMEYMILNER\AppData\Local\phoenix-2.0\cerebrum_nexus\src\lib.rs
```

### Intent Analysis
```
code intent C:\Users\JAMEYMILNER\AppData\Local\phoenix-2.0\cerebrum_nexus\src\lib.rs
```

### Codebase Analysis
```
code codebase C:\Users\JAMEYMILNER\AppData\Local\phoenix-2.0
```

## Access Levels

### Partial Access (Previous)
- Can list code definition names
- High-level structural overview
- Cannot perform deep semantic analysis
- Cannot understand full context

### Full Access (Current - Master Orchestrator)
- ✅ Can read any file, anywhere
- ✅ Deep semantic analysis with LLM
- ✅ Full context and intent understanding
- ✅ Cross-file dependency analysis
- ✅ Comprehensive codebase understanding
- ✅ Quality metrics and code smells
- ✅ Pattern and algorithm recognition

## Integration Status

- ✅ Code analysis module created
- ✅ Master Orchestrator integration module created
- ✅ Full file system access enabled
- ✅ LLM integration for deep semantic analysis
- ⏳ Command handler integration (to be added to cerebrum_nexus)
- ⏳ Frontend integration (optional)
