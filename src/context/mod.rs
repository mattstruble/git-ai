pub mod cache;
pub mod providers;
pub mod types;

pub use cache::ContextCache;
pub use providers::*;
pub use types::*;

use anyhow::{Context, Result};
use std::collections::HashMap;

/// Central manager for context gathering and caching
pub struct ContextManager {
    cache: ContextCache,
    providers: HashMap<ContextType, Box<dyn ContextProvider>>,
}

impl ContextManager {
    /// Create a new context manager with default providers
    pub fn new() -> Result<Self> {
        let cache = ContextCache::new()?;
        let mut providers: HashMap<ContextType, Box<dyn ContextProvider>> = HashMap::new();

        // Register default providers
        providers.insert(ContextType::Git, Box::new(GitContextProvider::new()));
        providers.insert(ContextType::Project, Box::new(ProjectContextProvider::new()));
        providers.insert(ContextType::Agent, Box::new(AgentContextProvider::new()));
        providers.insert(ContextType::Interaction, Box::new(InteractionContextProvider::new()));

        Ok(Self { cache, providers })
    }

    /// Gather specified context types, using cache when possible
    pub async fn gather_context(&self, required_types: &[ContextType]) -> Result<ContextBundle> {
        let mut contexts = HashMap::new();

        for context_type in required_types {
            let context_data = self.get_or_fetch_context(*context_type).await?;
            contexts.insert(*context_type, context_data);
        }

        Ok(ContextBundle::new(contexts))
    }

    /// Get context from cache or fetch fresh if needed
    async fn get_or_fetch_context(&self, context_type: ContextType) -> Result<ContextData> {
        // Try to get from cache first
        if let Some(cached_data) = self.cache.get(context_type).await? {
            return Ok(cached_data);
        }

        // Not in cache or expired, fetch fresh
        if let Some(provider) = self.providers.get(&context_type) {
            let fresh_data = provider.gather().await?;

            // Cache the fresh data
            self.cache.store(context_type, &fresh_data).await?;

            Ok(fresh_data)
        } else {
            anyhow::bail!("No provider registered for context type: {:?}", context_type);
        }
    }

    /// Force refresh context (bypass cache)
    #[allow(dead_code)]
    pub async fn refresh_context(&self, context_type: ContextType) -> Result<ContextData> {
        if let Some(provider) = self.providers.get(&context_type) {
            let fresh_data = provider.gather().await?;
            self.cache.store(context_type, &fresh_data).await?;
            Ok(fresh_data)
        } else {
            anyhow::bail!("No provider registered for context type: {:?}", context_type);
        }
    }

    /// Clear cache for specific context type or all
    #[allow(dead_code)]
    pub async fn clear_cache(&self, context_type: Option<ContextType>) -> Result<()> {
        match context_type {
            Some(ct) => self.cache.clear_type(ct).await,
            None => self.cache.clear_all().await,
        }
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ContextManager")
    }
}

/// Apply context to a prompt by embedding the context as structured JSON
pub fn apply_context(prompt: &str, context: &ContextBundle) -> Result<String> {
    let context_json = context.to_json().context("Failed to serialize context")?;

    let enhanced_prompt = format!(
        "{}\n\n--- CONTEXT ---\n{}\n--- END CONTEXT ---",
        prompt, context_json
    );

    Ok(enhanced_prompt)
}
