use crate::context::{ContextData, ContextProvider, ContextType, DocumentationContext};
use crate::cursor_agent::CursorAgent;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde_json;
use std::path::PathBuf;

/// Documentation context provider that processes project docs with AI
pub struct DocumentationContextProvider {
    agent: CursorAgent,
}

impl DocumentationContextProvider {
    pub fn new() -> Self {
        Self {
            agent: CursorAgent::new(),
        }
    }

    /// Get all documentation files in the repository
    fn get_documentation_files(&self) -> Vec<PathBuf> {
        let mut doc_files = Vec::new();

        // Common documentation patterns
        let doc_patterns = [
            "README.*",
            "LICENSE*",
            "COPYING*",
            "CHANGELOG.*",
            "CONTRIBUTING.*",
            "CODE_OF_CONDUCT.*",
            "SECURITY.*",
            "SUPPORT.*",
            "AUTHORS*",
            "MAINTAINERS*",
            "GOVERNANCE.*",
        ];

        // Check for files in root directory
        for pattern in &doc_patterns {
            let pattern_without_asterisk = pattern.trim_end_matches('*');
            let walker = WalkBuilder::new(".")
                .max_depth(Some(1))
                .hidden(false)
                .build();

            for entry in walker {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        if pattern.ends_with('*') {
                            if file_name.starts_with(pattern_without_asterisk) {
                                doc_files.push(path.to_path_buf());
                            }
                        } else if pattern.contains('*') {
                            // Handle patterns like README.*
                            let parts: Vec<&str> = pattern.split('*').collect();
                            if parts.len() == 2 {
                                let prefix = parts[0];
                                if file_name.starts_with(prefix) {
                                    doc_files.push(path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check docs/ directory
        if PathBuf::from("docs").exists() {
            let walker = WalkBuilder::new("docs")
                .hidden(false)
                .build();

            for entry in walker {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "md" || ext == "txt" || ext == "rst" {
                                doc_files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        doc_files.sort();
        doc_files.dedup();
        doc_files
    }

    /// Read and combine all documentation content
    async fn combine_documentation_content(&self, doc_files: &[PathBuf]) -> Result<String> {
        let mut combined_content = String::new();

        for file_path in doc_files {
            if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                // Add file header
                combined_content.push_str(&format!(
                    "\n=== {} ===\n{}\n",
                    file_path.display(),
                    content
                ));
            }
        }

        Ok(combined_content)
    }

    /// Create the AI prompt for processing documentation
    fn create_analysis_prompt(&self, docs_content: &str) -> String {
        format!(r#"You are an AI code analysis assistant that reads repository documentation to generate a structured context for a developer CLI tool.

You will be given the combined contents of all documentation and markdown files in a repository.

From this, extract the key information that will help an automated Git assistant
generate commit messages, PR descriptions, and detect breaking changes.

Respond ONLY in valid JSON following this schema:

{{
"project": {{
  "name": "<detected project name>",
  "description": "<one-sentence summary of what the project does>",
  "primary_language": "<main programming language or framework>",
  "keywords": ["<short tags about the project>"],
  "usage": {{
    "commands": ["<common CLI commands or APIs users call>"],
    "examples": ["<short example snippets>"],
    "configuration": ["<key env vars, config files, or flags>"]
  }},
  "development": {{
    "setup_steps": ["<build or install instructions>"],
    "dependencies": ["<main packages, frameworks, or libraries>"],
    "testing": ["<test commands or frameworks>"]
  }},
  "breaking_changes": {{
    "rules": ["<any stated versioning or backwards compatibility policies>"],
    "indicators": ["<phrases or patterns that indicate breaking changes>"]
  }},
  "conventions": {{
    "commit_style": "<rules for commit messages or conventional commits if any>",
    "branching": "<branch naming conventions>",
    "release_process": "<release workflow, changelog rules, or tagging>",
    "docs_reference": "<key documentation files relevant to dev process>"
  }},
  "misc": {{
    "notable_designs": ["<high-level architecture or design patterns>"],
    "important_files": ["<critical paths or modules mentioned in docs>"],
    "security_notes": ["<any cautions, credentials, or privacy info>"]
  }}
}}
}}

Rules:
- Output only the JSON object — no prose or markdown.
- If a field is unknown, use an empty array or null.
- Keep string values concise (max ~200 characters each).
- Preserve technical accuracy — this context will be used for automated code reasoning.
- Do not hallucinate features not found in the documentation.

Here is the full repository documentation:
---
{}
---"#, docs_content)
    }

    /// Parse AI response into DocumentationContext
    fn parse_ai_response(&self, response: &str, doc_files: &[PathBuf]) -> Result<DocumentationContext> {
        // Parse the JSON response from the AI
        let parsed: serde_json::Value = serde_json::from_str(response)
            .context("Failed to parse AI response as JSON")?;

        // Extract the top-level project object
        let project = parsed.get("project").unwrap_or(&serde_json::Value::Null);

        // Extract project info from the nested structure
        let project_info = crate::context::ProjectInfo {
            name: project.get("name").and_then(|v| v.as_str()).map(String::from),
            description: project.get("description").and_then(|v| v.as_str()).map(String::from),
            primary_language: project.get("primary_language").and_then(|v| v.as_str()).map(String::from),
            keywords: project.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        // Extract usage info from nested structure
        let usage = project.get("usage").unwrap_or(&serde_json::Value::Null);
        let usage_info = crate::context::UsageInfo {
            commands: usage.get("commands")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            examples: usage.get("examples")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            configuration: usage.get("configuration")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        // Extract development info from nested structure
        let development = project.get("development").unwrap_or(&serde_json::Value::Null);
        let development_info = crate::context::DevelopmentInfo {
            setup_steps: development.get("setup_steps")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            dependencies: development.get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            testing: development.get("testing")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        // Extract breaking changes info from nested structure
        let breaking_changes = project.get("breaking_changes").unwrap_or(&serde_json::Value::Null);
        let breaking_changes_info = crate::context::BreakingChangesInfo {
            rules: breaking_changes.get("rules")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            indicators: breaking_changes.get("indicators")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        // Extract conventions info from nested structure
        let conventions = project.get("conventions").unwrap_or(&serde_json::Value::Null);
        let conventions_info = crate::context::ConventionsInfo {
            commit_style: conventions.get("commit_style").and_then(|v| v.as_str()).map(String::from),
            branching: conventions.get("branching").and_then(|v| v.as_str()).map(String::from),
            release_process: conventions.get("release_process").and_then(|v| v.as_str()).map(String::from),
            docs_reference: conventions.get("docs_reference").and_then(|v| v.as_str()).map(String::from),
        };

        // Extract misc info from nested structure
        let misc = project.get("misc").unwrap_or(&serde_json::Value::Null);
        let misc_info = crate::context::MiscInfo {
            notable_designs: misc.get("notable_designs")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            important_files: misc.get("important_files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            security_notes: misc.get("security_notes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        let doc_file_names: Vec<String> = doc_files
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        Ok(DocumentationContext {
            project: project_info,
            usage: usage_info,
            development: development_info,
            breaking_changes: breaking_changes_info,
            conventions: conventions_info,
            misc: misc_info,
            doc_files: doc_file_names,
        })
    }
}

#[async_trait::async_trait]
impl ContextProvider for DocumentationContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        // Get all documentation files
        let doc_files = self.get_documentation_files();

        if doc_files.is_empty() {
            // Return empty context if no documentation found
            return Ok(ContextData::Documentation(DocumentationContext {
                project: crate::context::ProjectInfo {
                    name: None,
                    description: None,
                    primary_language: None,
                    keywords: vec![],
                },
                usage: crate::context::UsageInfo {
                    commands: vec![],
                    examples: vec![],
                    configuration: vec![],
                },
                development: crate::context::DevelopmentInfo {
                    setup_steps: vec![],
                    dependencies: vec![],
                    testing: vec![],
                },
                breaking_changes: crate::context::BreakingChangesInfo {
                    rules: vec![],
                    indicators: vec![],
                },
                conventions: crate::context::ConventionsInfo {
                    commit_style: None,
                    branching: None,
                    release_process: None,
                    docs_reference: None,
                },
                misc: crate::context::MiscInfo {
                    notable_designs: vec![],
                    important_files: vec![],
                    security_notes: vec![],
                },
                doc_files: vec![],
            }));
        }

        // Combine all documentation content
        let combined_content = self.combine_documentation_content(&doc_files).await?;

        // Create AI prompt
        let prompt = self.create_analysis_prompt(&combined_content);

        // Use the CursorAgent to process the documentation
        let ai_response = self.agent.prompt(&prompt).await
            .context("Failed to process documentation with AI agent")?;

        // Parse the response into structured context
        let documentation_context = self.parse_ai_response(&ai_response, &doc_files)?;

        Ok(ContextData::Documentation(documentation_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Documentation
    }

    async fn should_refresh(&self, cached_data: &ContextData) -> Result<bool> {
        // Documentation context should refresh if file hashes changed
        if let ContextData::Documentation(_) = cached_data {
            // Let the cache system handle expiry based on file hashes
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn get_file_dependencies(&self) -> Vec<PathBuf> {
        // Return all documentation files for hash tracking
        self.get_documentation_files()
    }
}
