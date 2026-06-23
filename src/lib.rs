#![doc = include_str!("../README.md")]

pub mod cli;
mod dock;
pub mod error;
mod install;
mod workspace;
mod zed;

use cli::{Cli, Commands};
use error::Result;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create {
            name,
            output,
            mode,
            folder,
            force,
        } => {
            let output = workspace::create_workspace_file(
                name.as_deref(),
                output.as_deref(),
                mode,
                &folder,
                force,
            )?;
            println!("{}", output.display());

            Ok(())
        }
        Commands::Open {
            workspace,
            mode,
            reuse,
            zed_bin,
        } => {
            let workspace = workspace::resolve_workspace_reference(&workspace)?;
            let workspace_file = workspace::read_workspace_file(&workspace)?;
            let mode = workspace_file.open_mode(mode)?;

            let target = match mode {
                cli::Mode::Folders => {
                    let folders = workspace_file.folder_targets(&workspace)?;
                    zed::OpenTarget::Folders(folders)
                }
                cli::Mode::Symlink => {
                    let folders = workspace_file.resolved_dock_folders(&workspace)?;
                    let dock_root = dock::build_dock(&workspace, &folders)?;
                    zed::OpenTarget::Dock(dock_root)
                }
            };

            zed::open_zed(&zed_bin, target, reuse)
        }
        Commands::Install {
            command,
            tasks_path,
        } => install::install_default_tasks(command.as_deref(), tasks_path.as_deref()),
        Commands::List => {
            for workspace in workspace::list_registered_workspaces()? {
                println!("{}\t{}", workspace.name, workspace.path.display());
            }

            Ok(())
        }
    }
}
