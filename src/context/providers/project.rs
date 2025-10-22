use crate::context::{ContextData, ContextProvider, ContextType, ProjectContext};
use crate::cursor_agent::CursorAgent;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde_json;
use std::path::PathBuf;

/// AI analysis prompt for processing documentation into structured project context
static DOCUMENTATION_ANALYSIS_PROMPT: &str = r#"You are an AI code analysis assistant. I need you to analyze documentation and respond with ONLY JSON - no other text, no explanations, no file creation.

CRITICAL:
- Do NOT create any files
- Do NOT write anything except the JSON response
- Do NOT add any explanatory text before or after the JSON
- Return ONLY the JSON object as specified

Analyze the provided repository documentation and respond with ONLY valid JSON following this exact schema:

{
"project": {
  "name": "<detected project name>",
  "description": "<one-sentence summary of what the project does>",
  "primary_language": "<main programming language or framework>",
  "keywords": ["<short tags about the project>"],
  "usage": {
    "commands": ["<common CLI commands or APIs users call>"],
    "examples": ["<short example snippets>"],
    "configuration": ["<key env vars, config files, or flags>"]
  },
  "development": {
    "setup_steps": ["<build or install instructions>"],
    "dependencies": ["<main packages, frameworks, or libraries>"],
    "testing": ["<test commands or frameworks>"]
  },
  "breaking_changes": {
    "rules": ["<any stated versioning or backwards compatibility policies>"],
    "indicators": ["<phrases or patterns that indicate breaking changes>"]
  },
  "conventions": {
    "commit_style": "<rules for commit messages or conventional commits if any>",
    "branching": "<branch naming conventions>",
    "release_process": "<release workflow, changelog rules, or tagging>",
    "docs_reference": "<key documentation files relevant to dev process>"
  },
  "misc": {
    "notable_designs": ["<high-level architecture or design patterns>"],
    "important_files": ["<critical paths or modules mentioned in docs>"],
    "security_notes": ["<any cautions, credentials, or privacy info>"]
  }
}
}

Rules:
- Output only the JSON object — no prose or markdown.
- If a field is unknown, use an empty array or null.
- Keep string values concise (max ~200 characters each).
- Preserve technical accuracy — this context will be used for automated code reasoning.
- Do not hallucinate features not found in the documentation.

Here is the full repository documentation:
---
{}
---"#;

/// Project context provider that processes project docs with AI
pub struct ProjectContextProvider {
    agent: CursorAgent,
}

impl ProjectContextProvider {
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
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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
            let walker = WalkBuilder::new("docs").hidden(false).build();

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
        DOCUMENTATION_ANALYSIS_PROMPT.replace("{}", docs_content)
    }

    /// Parse AI response into ProjectContext
    fn parse_ai_response(&self, response: &str, doc_files: &[PathBuf]) -> Result<ProjectContext> {
        // Parse the JSON response from the AI
        let parsed: serde_json::Value =
            serde_json::from_str(response).context("Failed to parse AI response as JSON")?;

        // Extract the top-level project object
        let project = parsed.get("project").unwrap_or(&serde_json::Value::Null);

        // Extract project info from the nested structure
        let project_info = crate::context::ProjectInfo {
            name: project
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from),
            description: project
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            primary_language: project
                .get("primary_language")
                .and_then(|v| v.as_str())
                .map(String::from),
            keywords: project
                .get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };

        // Extract usage info from nested structure
        let usage = project.get("usage").unwrap_or(&serde_json::Value::Null);
        let usage_info = crate::context::UsageInfo {
            commands: usage
                .get("commands")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            examples: usage
                .get("examples")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            configuration: usage
                .get("configuration")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };

        // Extract development info from nested structure
        let development = project
            .get("development")
            .unwrap_or(&serde_json::Value::Null);
        let development_info = crate::context::DevelopmentInfo {
            setup_steps: development
                .get("setup_steps")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: development
                .get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            testing: development
                .get("testing")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };

        // Extract breaking changes info from nested structure
        let breaking_changes = project
            .get("breaking_changes")
            .unwrap_or(&serde_json::Value::Null);
        let breaking_changes_info = crate::context::BreakingChangesInfo {
            rules: breaking_changes
                .get("rules")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            indicators: breaking_changes
                .get("indicators")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };

        // Extract conventions info from nested structure
        let conventions = project
            .get("conventions")
            .unwrap_or(&serde_json::Value::Null);
        let conventions_info = crate::context::ConventionsInfo {
            commit_style: conventions
                .get("commit_style")
                .and_then(|v| v.as_str())
                .map(String::from),
            branching: conventions
                .get("branching")
                .and_then(|v| v.as_str())
                .map(String::from),
            release_process: conventions
                .get("release_process")
                .and_then(|v| v.as_str())
                .map(String::from),
            docs_reference: conventions
                .get("docs_reference")
                .and_then(|v| v.as_str())
                .map(String::from),
        };

        // Extract misc info from nested structure
        let misc = project.get("misc").unwrap_or(&serde_json::Value::Null);
        let misc_info = crate::context::MiscInfo {
            notable_designs: misc
                .get("notable_designs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            important_files: misc
                .get("important_files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            security_notes: misc
                .get("security_notes")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };

        let doc_file_names: Vec<String> =
            doc_files.iter().map(|p| p.display().to_string()).collect();

        Ok(ProjectContext {
            project: project_info,
            usage: usage_info,
            development: development_info,
            breaking_changes: breaking_changes_info,
            conventions: conventions_info,
            misc: misc_info,
            doc_files: doc_file_names,
        })
    }

    /// Create a fallback context when AI processing is unavailable
    fn create_fallback_context(&self, doc_files: &[PathBuf]) -> Result<ContextData> {
        let doc_file_names: Vec<String> =
            doc_files.iter().map(|p| p.display().to_string()).collect();

        let fallback_context = ProjectContext {
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
            doc_files: doc_file_names,
        };

        Ok(ContextData::Project(fallback_context))
    }
}

#[async_trait::async_trait]
impl ContextProvider for ProjectContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        // Get all documentation files
        let doc_files = self.get_documentation_files();

        if doc_files.is_empty() {
            println!("📄 No documentation files found - using empty project context");
            // Return empty context if no documentation found
            return Ok(ContextData::Project(ProjectContext {
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

        // Inform user about AI processing
        println!(
            "Processing {} documentation files to extract project metadata...",
            doc_files.len()
        );

        // Create AI prompt
        let prompt = self.create_analysis_prompt(&combined_content);

        // Use the CursorAgent to process the documentation
        let ai_response = match self.agent.prompt(&prompt).await {
            Ok(response) if !response.trim().is_empty() => response,
            Ok(_) => {
                println!(
                    "⚠️  AI processing returned empty response - using fallback project context"
                );
                return self.create_fallback_context(&doc_files);
            }
            Err(e) => {
                println!(
                    "⚠️  AI processing failed ({}), using fallback project context",
                    e
                );
                return self.create_fallback_context(&doc_files);
            }
        };

        // Parse the response into structured context
        let project_context = match self.parse_ai_response(&ai_response, &doc_files) {
            Ok(context) => context,
            Err(e) => {
                println!(
                    "⚠️  AI response parsing failed ({}), using fallback project context",
                    e
                );
                return self.create_fallback_context(&doc_files);
            }
        };

        println!("✅ Project context updated successfully");

        Ok(ContextData::Project(project_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Project
    }

    async fn should_refresh(&self, cached_data: &ContextData) -> Result<bool> {
        // Project context should refresh if file hashes changed
        if let ContextData::Project(_) = cached_data {
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
