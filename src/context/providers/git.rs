use crate::context::{
    BranchInfo, CommitInfo, ContextData, ContextProvider, ContextType, FileStatus, GitContext,
    GitDiffs, RepositoryMetadata, RepositoryStatus, UserContext,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::str;

/// Git context provider
pub struct GitContextProvider;

impl GitContextProvider {
    pub fn new() -> Self {
        Self
    }

    /// Get repository status (staged, unstaged, untracked files)
    async fn get_repository_status(&self) -> Result<RepositoryStatus> {
        // Get porcelain status
        let output = Command::new("git")
            .args(["status", "--porcelain=v1", "-z"])
            .output()
            .context("Failed to execute git status")?;

        if !output.status.success() {
            anyhow::bail!(
                "git status failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let status_output = str::from_utf8(&output.stdout)?;
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut has_conflicts = false;

        // Parse status lines (null-terminated)
        for line in status_output.split('\0').filter(|s| !s.is_empty()) {
            if line.len() < 3 {
                continue;
            }

            let staged_status = line.chars().nth(0).unwrap_or(' ');
            let unstaged_status = line.chars().nth(1).unwrap_or(' ');
            let file_path = &line[3..];

            // Check for conflicts
            if staged_status == 'U'
                || unstaged_status == 'U'
                || (staged_status == 'A' && unstaged_status == 'A')
                || (staged_status == 'D' && unstaged_status == 'D')
            {
                has_conflicts = true;
            }

            // Handle staged changes
            if staged_status != ' ' && staged_status != '?' {
                let (insertions, deletions) = self
                    .get_file_stats(file_path, true)
                    .await
                    .unwrap_or((None, None));
                staged_files.push(FileStatus {
                    path: file_path.to_string(),
                    status: staged_status.to_string(),
                    insertions,
                    deletions,
                });
            }

            // Handle unstaged changes
            if unstaged_status != ' ' {
                if unstaged_status == '?' {
                    untracked_files.push(file_path.to_string());
                } else {
                    let (insertions, deletions) = self
                        .get_file_stats(file_path, false)
                        .await
                        .unwrap_or((None, None));
                    unstaged_files.push(FileStatus {
                        path: file_path.to_string(),
                        status: unstaged_status.to_string(),
                        insertions,
                        deletions,
                    });
                }
            }
        }

        let is_clean =
            staged_files.is_empty() && unstaged_files.is_empty() && untracked_files.is_empty();

        Ok(RepositoryStatus {
            staged_files,
            unstaged_files,
            untracked_files,
            is_clean,
            has_conflicts,
        })
    }

    /// Get file statistics (insertions/deletions) for a specific file
    async fn get_file_stats(
        &self,
        file_path: &str,
        staged: bool,
    ) -> Result<(Option<u32>, Option<u32>)> {
        let mut cmd_args = vec!["diff", "--numstat"];
        if staged {
            cmd_args.push("--cached");
        }
        cmd_args.push("--");
        cmd_args.push(file_path);

        let output = Command::new("git")
            .args(&cmd_args)
            .output()
            .context("Failed to execute git diff --numstat")?;

        if !output.status.success() {
            return Ok((None, None));
        }

        let numstat_output = String::from_utf8(output.stdout)?;
        if let Some(line) = numstat_output.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let insertions = parts[0].parse::<u32>().ok();
                let deletions = parts[1].parse::<u32>().ok();
                return Ok((insertions, deletions));
            }
        }

        Ok((None, None))
    }

    /// Get git diffs for staged and unstaged changes
    async fn get_diffs(&self) -> Result<GitDiffs> {
        // Get staged diff
        let staged_diff = self.get_diff(&["--cached"]).await.ok();

        // Get unstaged diff
        let unstaged_diff = self.get_diff(&[]).await.ok();

        // Get branch diff (against upstream or main/master)
        let branch_diff = self.get_branch_diff().await.ok();

        Ok(GitDiffs {
            staged: staged_diff,
            unstaged: unstaged_diff,
            branch_diff,
        })
    }

    /// Get diff with specified arguments
    async fn get_diff(&self, args: &[&str]) -> Result<String> {
        let mut cmd_args = vec!["diff"];
        cmd_args.extend(args);

        let output = Command::new("git")
            .args(&cmd_args)
            .output()
            .context("Failed to execute git diff")?;

        if !output.status.success() {
            anyhow::bail!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Get diff against upstream branch or main/master
    async fn get_branch_diff(&self) -> Result<String> {
        // Try to get upstream branch first
        if let Ok(upstream) = self.get_upstream_branch().await {
            return self.get_diff(&[&format!("{}..HEAD", upstream)]).await;
        }

        // Fallback to main or master
        for branch in &["main", "master"] {
            if self.branch_exists(branch).await {
                if let Ok(diff) = self.get_diff(&[&format!("{}..HEAD", branch)]).await {
                    return Ok(diff);
                }
            }
        }

        anyhow::bail!("Could not determine base branch for diff")
    }

    /// Check if a branch exists
    async fn branch_exists(&self, branch: &str) -> bool {
        Command::new("git")
            .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get recent commits
    async fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", count),
                "--pretty=format:%H|%h|%s|%an|%ai",
                "--name-only",
            ])
            .output()
            .context("Failed to execute git log")?;

        if !output.status.success() {
            anyhow::bail!(
                "git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let log_output = String::from_utf8(output.stdout)?;
        let mut commits = Vec::new();
        let mut current_commit: Option<CommitInfo> = None;
        let mut files_for_commit = Vec::new();

        for line in log_output.lines() {
            if line.contains('|') && line.split('|').count() == 5 {
                // Save previous commit if exists
                if let Some(mut commit) = current_commit.take() {
                    commit.files_changed = files_for_commit.clone();
                    commits.push(commit);
                    files_for_commit.clear();
                }

                // Parse new commit
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 5 {
                    current_commit = Some(CommitInfo {
                        hash: parts[0].to_string(),
                        short_hash: parts[1].to_string(),
                        message: parts[2].to_string(),
                        author: parts[3].to_string(),
                        date: chrono::DateTime::parse_from_rfc3339(parts[4])
                            .unwrap_or_else(|_| chrono::Utc::now().into())
                            .with_timezone(&chrono::Utc),
                        files_changed: Vec::new(),
                    });
                }
            } else if !line.is_empty() {
                // This is a file path
                files_for_commit.push(line.to_string());
            }
        }

        // Don't forget the last commit
        if let Some(mut commit) = current_commit {
            commit.files_changed = files_for_commit;
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Get branch information
    async fn get_branch_info(&self) -> Result<BranchInfo> {
        // Get current branch
        let current_branch = self.get_current_branch().await?;

        // Get upstream branch
        let upstream_branch = self.get_upstream_branch().await.ok();

        // Get ahead/behind counts
        let (ahead, behind) = if let Some(ref upstream) = upstream_branch {
            self.get_ahead_behind_counts(&current_branch, upstream)
                .await
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        let tracking_status = match (&upstream_branch, ahead, behind) {
            (None, _, _) => "No upstream".to_string(),
            (Some(_), 0, 0) => "Up to date".to_string(),
            (Some(_), a, 0) => format!("Ahead by {}", a),
            (Some(_), 0, b) => format!("Behind by {}", b),
            (Some(_), a, b) => format!("Ahead by {}, behind by {}", a, b),
        };

        Ok(BranchInfo {
            current_branch,
            upstream_branch,
            ahead,
            behind,
            tracking_status,
        })
    }

    /// Get current branch name
    async fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get current branch: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get upstream branch
    async fn get_upstream_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "@{upstream}"])
            .output()
            .context("Failed to get upstream branch")?;

        if !output.status.success() {
            anyhow::bail!("No upstream branch configured");
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get ahead/behind counts
    async fn get_ahead_behind_counts(&self, local: &str, upstream: &str) -> Result<(u32, u32)> {
        let output = Command::new("git")
            .args([
                "rev-list",
                "--left-right",
                "--count",
                &format!("{}...{}", upstream, local),
            ])
            .output()
            .context("Failed to get ahead/behind counts")?;

        if !output.status.success() {
            return Ok((0, 0));
        }

        let output_str = String::from_utf8(output.stdout)?;
        let parts: Vec<&str> = output_str.split_whitespace().collect();

        if parts.len() == 2 {
            let behind = parts[0].parse().unwrap_or(0);
            let ahead = parts[1].parse().unwrap_or(0);
            Ok((ahead, behind))
        } else {
            Ok((0, 0))
        }
    }

    /// Get user context
    async fn get_user_context(&self) -> Result<UserContext> {
        let name = Command::new("git")
            .args(["config", "user.name"])
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
            });

        let email = Command::new("git")
            .args(["config", "user.email"])
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
            });

        Ok(UserContext { name, email })
    }

    /// Get repository metadata
    async fn get_repository_metadata(&self) -> Result<RepositoryMetadata> {
        // Get repository root
        let root_output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get repository root")?;

        let root_path = if root_output.status.success() {
            String::from_utf8(root_output.stdout)?.trim().to_string()
        } else {
            std::env::current_dir()?.to_string_lossy().to_string()
        };

        // Get git directory
        let git_dir_output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .context("Failed to get git directory")?;

        let git_dir = if git_dir_output.status.success() {
            String::from_utf8(git_dir_output.stdout)?.trim().to_string()
        } else {
            ".git".to_string()
        };

        // Check if bare repository
        let is_bare = Command::new("git")
            .args(["rev-parse", "--is-bare-repository"])
            .output()
            .map(|output| {
                output.status.success()
                    && String::from_utf8(output.stdout).unwrap_or_default().trim() == "true"
            })
            .unwrap_or(false);

        // Get remote URLs
        let remotes_output = Command::new("git").args(["remote", "-v"]).output();

        let mut remote_urls = Vec::new();
        if let Ok(output) = remotes_output {
            if output.status.success() {
                let remotes_str = String::from_utf8(output.stdout).unwrap_or_default();
                for line in remotes_str.lines() {
                    if let Some(url_part) = line.split_whitespace().nth(1) {
                        if !remote_urls.contains(&url_part.to_string()) {
                            remote_urls.push(url_part.to_string());
                        }
                    }
                }
            }
        }

        Ok(RepositoryMetadata {
            root_path,
            git_dir,
            is_bare,
            remote_urls,
        })
    }
}

#[async_trait::async_trait]
impl ContextProvider for GitContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let repository_status = self.get_repository_status().await?;
        let diffs = self.get_diffs().await?;
        let recent_commits = self.get_recent_commits(10).await?;
        let branch_info = self.get_branch_info().await?;
        let user_context = self.get_user_context().await?;
        let repository_metadata = self.get_repository_metadata().await?;

        let git_context = GitContext {
            repository_status,
            diffs,
            recent_commits,
            branch_info,
            user_context,
            repository_metadata,
        };

        Ok(ContextData::Git(git_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Git
    }

    async fn should_refresh(&self, _cached_data: &ContextData) -> Result<bool> {
        // Git context should always be refreshed for accuracy
        // The caching system handles git hash-based invalidation
        Ok(true)
    }

    fn get_file_dependencies(&self) -> Vec<PathBuf> {
        // Git context doesn't depend on specific files, it uses git commands
        // So we return empty - it will use git hash-based invalidation instead
        vec![]
    }
}
