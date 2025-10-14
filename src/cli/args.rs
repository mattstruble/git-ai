/// Common arguments shared by all commands
#[derive(Debug, Clone)]
pub struct CommonArgs {
    pub dry_run: bool,
    #[allow(dead_code)] // Will be used in future phases
    pub verbose: bool,
    pub message: Option<String>,
}

/// Arguments specific to commit command
#[derive(Debug, Clone)]
pub struct CommitArgs {
    pub common: CommonArgs,
    pub no_confirm: bool,
}

/// Arguments specific to PR command
#[derive(Debug, Clone)]
pub struct PrArgs {
    pub common: CommonArgs,
    pub no_confirm: bool,
}

/// Arguments specific to merge command
#[derive(Debug, Clone)]
pub struct MergeArgs {
    pub common: CommonArgs,
    pub branch: String,
    pub no_confirm: bool,
}

/// Arguments specific to config command
#[derive(Debug, Clone)]
pub struct ConfigArgs {
    pub show: bool,
    pub init: bool,
}

/// Arguments specific to init command
#[derive(Debug, Clone)]
pub struct InitArgs {
    pub common: CommonArgs,
    pub language: Option<String>,
    pub name: Option<String>,
    pub no_confirm: bool,
}
