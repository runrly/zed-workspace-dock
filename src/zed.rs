use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::{AppError, Result};

pub(crate) enum OpenTarget {
    Folders(Vec<PathBuf>),
    Dock(PathBuf),
}

pub(crate) fn open_zed(zed_bin: &Path, target: OpenTarget, reuse: bool) -> Result<()> {
    let args = zed_args(target, reuse);
    let status = Command::new(zed_bin)
        .args(&args)
        .status()
        .map_err(|source| AppError::LaunchZed {
            path: zed_bin.to_path_buf(),
            source,
        })?;

    if !status.success() {
        return Err(AppError::ZedExited { status });
    }

    Ok(())
}

fn zed_args(target: OpenTarget, reuse: bool) -> Vec<PathBuf> {
    let mut args = Vec::new();

    if !reuse {
        args.push(PathBuf::from("-n"));
    }

    match target {
        OpenTarget::Folders(folders) => {
            args.extend(folders);
        }
        OpenTarget::Dock(dock_root) => {
            args.push(dock_root);
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn folders_mode_uses_new_window_by_default() {
        let args = zed_args(OpenTarget::Folders(vec![PathBuf::from("/tmp/api")]), false);

        assert_eq!(args, vec![PathBuf::from("-n"), PathBuf::from("/tmp/api")]);
    }

    #[test]
    fn reuse_mode_omits_new_window_flag() {
        let args = zed_args(OpenTarget::Dock(PathBuf::from("/tmp/dock")), true);

        assert_eq!(args, vec![PathBuf::from("/tmp/dock")]);
    }
}
