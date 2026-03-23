use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "sac", about = "Save all commands - terminal command manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        #[arg(long)]
        folder: Option<String>,
    },
    NewFolder {
        name: String,
        #[arg(long)]
        parent: Option<String>,
    },
    Edit {
        command_id: u32,
    },
    Delete {
        command_id: u32,
    },
    Sync {
        #[arg(long)]
        force: bool,
    },
    Config(ConfigArgs),
    Where {
        target: WhereTarget,
    },
    Install,
    Export {
        path: String,
    },
    Import {
        path: String,
    },
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub subcommand: Option<ConfigSubcommand>,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    Set { key: String, value: String },
}

#[derive(ValueEnum, Clone)]
pub enum WhereTarget {
    Config,
    Commands,
}
