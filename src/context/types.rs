use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Types of context that can be gathered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextType {
    /// Git repository context (status, diffs, commits, branches)
    Git,
    /// Project structure context (directory tree, file analysis)
    Project,
    /// Agent configuration context (.cursoragent files)
    Agent,
    /// Interaction context (user intent, command flags)
    Interaction,
}

/// Base trait for context providers
#[async_trait::async_trait]
pub trait ContextProvider: Send + Sync {
    /// Gather fresh context data
    async fn gather(&self) -> Result<ContextData>;

    /// Get the context type this provider handles
    #[allow(dead_code)]
    fn context_type(&self) -> ContextType;

    /// Check if context should be refreshed based on current state
    #[allow(dead_code)]
    async fn should_refresh(&self, cached_data: &ContextData) -> Result<bool>;

    /// Get file dependencies for this context provider
    /// Returns paths to files that this context depends on for cache invalidation
    fn get_file_dependencies(&self) -> Vec<PathBuf>;
}

/// Container for all context data with serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundle {
    pub contexts: HashMap<ContextType, ContextData>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub git_hash: Option<String>,
    pub working_tree_hash: Option<String>,
}

impl ContextBundle {
    pub fn new(contexts: HashMap<ContextType, ContextData>) -> Self {
        Self {
            contexts,
            generated_at: chrono::Utc::now(),
            git_hash: None,
            working_tree_hash: None,
        }
    }

    /// Get context data for a specific type
    pub fn get(&self, context_type: ContextType) -> Option<&ContextData> {
        self.contexts.get(&context_type)
    }

    /// Check if bundle contains specific context type
    #[allow(dead_code)]
    pub fn has(&self, context_type: ContextType) -> bool {
        self.contexts.contains_key(&context_type)
    }

    /// Convert to JSON string for passing to cursor-agent
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }
}

/// Generic context data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextData {
    Git(GitContext),
    Project(ProjectContext),
    Agent(AgentContext),
    Interaction(InteractionContext),
}

/// Git repository context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub repository_status: RepositoryStatus,
    pub diffs: GitDiffs,
    pub recent_commits: Vec<CommitInfo>,
    pub branch_info: BranchInfo,
    pub user_context: UserContext,
    pub repository_metadata: RepositoryMetadata,
}

/// Repository status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub staged_files: Vec<FileStatus>,
    pub unstaged_files: Vec<FileStatus>,
    pub untracked_files: Vec<String>,
    pub is_clean: bool,
    pub has_conflicts: bool,
}

/// Git diffs for different scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffs {
    pub staged: Option<String>,
    pub unstaged: Option<String>,
    pub branch_diff: Option<String>,
}

/// File status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String, // M, A, D, R, C, U, etc.
    pub insertions: Option<u32>,
    pub deletions: Option<u32>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub files_changed: Vec<String>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub current_branch: String,
    pub upstream_branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub tracking_status: String,
}

/// Git user context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub root_path: String,
    pub git_dir: String,
    pub is_bare: bool,
    pub remote_urls: Vec<String>,
}

/// Project structure context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub directory_tree: String,
    pub dependency_files: HashMap<String, String>, // filename -> content
    pub file_counts: HashMap<String, u32>,         // extension -> count
    pub recently_changed_files: Vec<String>,
    pub total_files: u32,
    pub total_size: u64,
}

/// Agent configuration context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub config_files: Vec<AgentConfigFile>,
    pub rules: Vec<String>,
    pub custom_prompts: HashMap<String, String>,
}

/// Agent configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigFile {
    pub path: String,
    pub content: String,
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Text,
}

/// Interaction context (user intent and flags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub command: String,
    pub user_message: Option<String>,
    pub flags: HashMap<String, String>,
    pub execution_metadata: ExecutionMetadata,
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub working_directory: String,
    pub git_ai_version: String,
    pub cursor_agent_version: Option<String>,
}
