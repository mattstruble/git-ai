const MERGE_BASE_PROMPT_TEMPLATE: &str =
    "Analyze the merge of '{}' into the current branch and provide merge assistance.

If there are merge conflicts:
- Explain what caused the conflicts between the current branch and '{}'
- Identify the conflicting areas and their purposes
- Suggest a resolution strategy specific to merging '{}'
- Provide an appropriate merge commit message

If this is for generating a merge commit message:
- Summarize what changes from '{}' are being integrated
- Highlight key features or fixes being merged
- Follow standard merge commit format: 'Merge branch {}' or similar
- Include brief description of what '{}' brings to the codebase

Focus on clarity and helping developers understand the specific merge context for '{}'.";

const MERGE_INSTRUCTION_TEMPLATE: &str =
    "Please review the git status and provide appropriate merge guidance for '{}'.";

pub struct MergeCommand;

impl MergeCommand {
    pub fn default_prompt_with_branch(
        &self,
        target_branch: &str,
        custom_message: Option<String>,
    ) -> String {
        let base_prompt = MERGE_BASE_PROMPT_TEMPLATE.replace("{}", target_branch);

        let instruction = MERGE_INSTRUCTION_TEMPLATE.replace("{}", target_branch);

        match custom_message {
            Some(message) => format!(
                "{}\n\nSpecific context: {}\n\n{}",
                base_prompt, message, instruction
            ),
            None => format!("{}\n\n{}", base_prompt, instruction),
        }
    }
}
