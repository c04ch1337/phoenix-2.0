//! Master Orchestrator Code Analysis Integration
//!
//! Provides full-control and unlimited access for code analysis.
//! Master Orchestrator can:
//! - Read any file, anywhere on the system
//! - Perform deep semantic analysis with LLM
//! - Understand full context and intent
//! - Analyze cross-file dependencies
//! - Provide comprehensive codebase understanding

use crate::{CodeAnalyzer, CodeAnalysis, CodebaseAnalysis};
use error_types::PhoenixError;
use llm_orchestrator::LLMOrchestrator;
use std::path::{Path, PathBuf};

/// Master Orchestrator Code Analysis Manager
/// 
/// Provides unlimited access to code analysis capabilities.
/// Master Orchestrator has full system access by default.
pub struct MasterOrchestratorCodeAnalysis {
    analyzer: CodeAnalyzer,
}

impl MasterOrchestratorCodeAnalysis {
    /// Create a new Master Orchestrator code analyzer with LLM support
    /// 
    /// Master Orchestrator has full access - no restrictions.
    pub fn new_with_llm(llm_orchestrator: LLMOrchestrator) -> Self {
        Self {
            analyzer: CodeAnalyzer::with_llm(llm_orchestrator),
        }
    }

    /// Create a new Master Orchestrator code analyzer without LLM
    /// 
    /// Still provides full file system access and structural analysis.
    pub fn new() -> Self {
        Self {
            analyzer: CodeAnalyzer::new(),
        }
    }

    /// Analyze a single file with full semantic understanding
    /// 
    /// Master Orchestrator can read from anywhere on the system.
    /// Provides deep semantic analysis, not just definition listing.
    pub async fn analyze_file(&self, file_path: &Path) -> Result<CodeAnalysis, PhoenixError> {
        // Master Orchestrator has unlimited access - read from anywhere
        self.analyzer.analyze_file(file_path).await
    }

    /// Analyze entire codebase recursively
    /// 
    /// Master Orchestrator can analyze any directory on the system.
    pub async fn analyze_codebase(&self, root_path: &Path) -> Result<CodebaseAnalysis, PhoenixError> {
        // Master Orchestrator has unlimited access - analyze anywhere
        self.analyzer.analyze_codebase(root_path).await
    }

    /// Get high-level overview of codebase (definition names only)
    /// 
    /// This is the "partial access" mode - just listing definitions.
    /// For full understanding, use analyze_file() or analyze_codebase().
    pub async fn list_definitions(&self, file_path: &Path) -> Result<DefinitionList, PhoenixError> {
        let analysis = self.analyze_file(file_path).await?;
        
        Ok(DefinitionList {
            file_path: file_path.to_path_buf(),
            functions: analysis.structure.functions.iter().map(|f| f.name.clone()).collect(),
            types: analysis.structure.types.iter().map(|t| t.name.clone()).collect(),
            modules: analysis.structure.modules.iter().map(|m| m.name.clone()).collect(),
            constants: analysis.structure.constants.iter().map(|c| c.name.clone()).collect(),
        })
    }

    /// Perform deep semantic analysis on a file
    /// 
    /// This provides full context understanding, not just structure.
    /// Master Orchestrator uses LLM for deep semantic understanding.
    pub async fn deep_semantic_analysis(&self, file_path: &Path) -> Result<SemanticAnalysisResult, PhoenixError> {
        let analysis = self.analyze_file(file_path).await?;
        
        Ok(SemanticAnalysisResult {
            file_path: file_path.to_path_buf(),
            purpose: analysis.semantics.purpose,
            intent: analysis.intent.primary_intent,
            concepts: analysis.semantics.concepts,
            patterns: analysis.semantics.patterns,
            algorithms: analysis.semantics.algorithms,
            data_structures: analysis.semantics.data_structures,
            error_handling: analysis.semantics.error_handling,
            concurrency: analysis.semantics.concurrency,
            security_notes: analysis.semantics.security_notes,
            performance_notes: analysis.semantics.performance_notes,
            relationships: analysis.semantics.relationships,
            expected_behavior: analysis.intent.expected_behavior,
            edge_cases: analysis.intent.edge_cases,
        })
    }

    /// Analyze code intent and purpose with full context
    /// 
    /// Master Orchestrator understands not just what code does,
    /// but why it exists and how it fits into the larger system.
    pub async fn analyze_intent(&self, file_path: &Path) -> Result<CodeIntentResult, PhoenixError> {
        let analysis = self.analyze_file(file_path).await?;
        
        Ok(CodeIntentResult {
            file_path: file_path.to_path_buf(),
            primary_intent: analysis.intent.primary_intent,
            secondary_intents: analysis.intent.secondary_intents,
            business_purpose: analysis.intent.business_purpose,
            technical_purpose: analysis.intent.technical_purpose,
            user_purpose: analysis.intent.user_purpose,
            expected_behavior: analysis.intent.expected_behavior,
            edge_cases: analysis.intent.edge_cases,
        })
    }

    /// Analyze dependencies across the codebase
    /// 
    /// Master Orchestrator can trace dependencies across any files.
    pub async fn analyze_dependencies(&self, file_path: &Path) -> Result<DependencyAnalysis, PhoenixError> {
        let analysis = self.analyze_file(file_path).await?;
        
        Ok(DependencyAnalysis {
            file_path: file_path.to_path_buf(),
            external_dependencies: analysis.dependencies.external,
            internal_dependencies: analysis.dependencies.internal,
            dependency_graph: analysis.dependencies.graph,
            circular_dependencies: analysis.dependencies.circular,
        })
    }

    /// Get code quality metrics
    /// 
    /// Master Orchestrator can assess code quality comprehensively.
    pub async fn quality_metrics(&self, file_path: &Path) -> Result<QualityMetricsResult, PhoenixError> {
        let analysis = self.analyze_file(file_path).await?;
        
        Ok(QualityMetricsResult {
            file_path: file_path.to_path_buf(),
            maintainability: analysis.quality.maintainability,
            readability: analysis.quality.readability,
            test_coverage: analysis.quality.test_coverage,
            code_smells: analysis.quality.code_smells,
            best_practices: analysis.quality.best_practices,
            violations: analysis.quality.violations,
        })
    }
}

/// Definition list (partial access - just names)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefinitionList {
    pub file_path: PathBuf,
    pub functions: Vec<String>,
    pub types: Vec<String>,
    pub modules: Vec<String>,
    pub constants: Vec<String>,
}

/// Deep semantic analysis result (full access)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticAnalysisResult {
    pub file_path: PathBuf,
    pub purpose: String,
    pub intent: String,
    pub concepts: Vec<String>,
    pub patterns: Vec<String>,
    pub algorithms: Vec<String>,
    pub data_structures: Vec<String>,
    pub error_handling: String,
    pub concurrency: Option<String>,
    pub security_notes: Vec<String>,
    pub performance_notes: Vec<String>,
    pub relationships: Vec<crate::CodeRelationship>,
    pub expected_behavior: String,
    pub edge_cases: Vec<String>,
}

/// Code intent analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeIntentResult {
    pub file_path: PathBuf,
    pub primary_intent: String,
    pub secondary_intents: Vec<String>,
    pub business_purpose: Option<String>,
    pub technical_purpose: String,
    pub user_purpose: Option<String>,
    pub expected_behavior: String,
    pub edge_cases: Vec<String>,
}

/// Dependency analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyAnalysis {
    pub file_path: PathBuf,
    pub external_dependencies: Vec<crate::Dependency>,
    pub internal_dependencies: Vec<crate::Dependency>,
    pub dependency_graph: crate::DependencyGraph,
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Quality metrics result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityMetricsResult {
    pub file_path: PathBuf,
    pub maintainability: f64,
    pub readability: f64,
    pub test_coverage: Option<f64>,
    pub code_smells: Vec<crate::CodeSmell>,
    pub best_practices: Vec<String>,
    pub violations: Vec<String>,
}
