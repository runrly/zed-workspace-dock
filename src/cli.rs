use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Create {
        name: Option<String>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        mode: Mode,
        #[arg(long = "folder", required = true)]
        folder: Vec<String>,
        #[arg(long)]
        force: bool,
    },
    Open {
        workspace: PathBuf,
        #[arg(long)]
        mode: Option<Mode>,
        #[arg(long)]
        reuse: bool,
        #[arg(long, default_value = "zed")]
        zed_bin: PathBuf,
    },
    Install {
        #[arg(long)]
        command: Option<PathBuf>,
        #[arg(long)]
        tasks_path: Option<PathBuf>,
    },
    List,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Folders,
    Symlink,
}
