use crate::context::{
    ContextData, ContextProvider, ContextType, ExecutionMetadata, InteractionContext,
};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

/// Interaction context provider
pub struct InteractionContextProvider {
    command: Option<String>,
    user_message: Option<String>,
    flags: HashMap<String, String>,
}

impl InteractionContextProvider {
    pub fn new() -> Self {
        Self {
            command: None,
            user_message: None,
            flags: HashMap::new(),
        }
    }

    /// Create with command information
    #[allow(dead_code)]
    pub fn with_command(command: String) -> Self {
        Self {
            command: Some(command),
            user_message: None,
            flags: HashMap::new(),
        }
    }

    /// Set user message
    #[allow(dead_code)]
    pub fn with_message(mut self, message: Option<String>) -> Self {
        self.user_message = message;
        self
    }

    /// Add flag
    #[allow(dead_code)]
    pub fn with_flag(mut self, key: String, value: String) -> Self {
        self.flags.insert(key, value);
        self
    }

    /// Add flags from a collection
    #[allow(dead_code)]
    pub fn with_flags(mut self, flags: HashMap<String, String>) -> Self {
        self.flags.extend(flags);
        self
    }

    /// Get cursor-agent version
    async fn get_cursor_agent_version(&self) -> Option<String> {
        Command::new("cursor-agent")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Get git-ai version from Cargo.toml or compiled version
    fn get_git_ai_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get current working directory
    fn get_working_directory(&self) -> String {
        std::env::current_dir()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Create execution metadata
    async fn create_execution_metadata(&self) -> ExecutionMetadata {
        ExecutionMetadata {
            timestamp: chrono::Utc::now(),
            working_directory: self.get_working_directory(),
            git_ai_version: self.get_git_ai_version(),
            cursor_agent_version: self.get_cursor_agent_version().await,
        }
    }
}

#[async_trait::async_trait]
impl ContextProvider for InteractionContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let execution_metadata = self.create_execution_metadata().await;

        let interaction_context = InteractionContext {
            command: self
                .command
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            user_message: self.user_message.clone(),
            flags: self.flags.clone(),
            execution_metadata,
        };

        Ok(ContextData::Interaction(interaction_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Interaction
    }

    async fn should_refresh(&self, _cached_data: &ContextData) -> Result<bool> {
        // Interaction context should always be fresh since it represents
        // the current command execution
        Ok(true)
    }
}
