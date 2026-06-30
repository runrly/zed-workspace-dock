use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    error::{AppError, Result},
    workspace::ResolvedFolder,
};

const MARKER_FILE: &str = ".zed-dock.json";
const MARKER_VERSION: u8 = 1;

#[derive(Debug, Deserialize, Serialize)]
struct DockMarker {
    version: u8,
    workspace_path: PathBuf,
    links: Vec<DockLink>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DockLink {
    name: String,
    target: PathBuf,
}

pub(crate) fn build_dock(workspace_path: &Path, folders: &[ResolvedFolder]) -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir().ok_or(AppError::CacheDirNotFound)?;
    build_dock_in(&cache_dir, workspace_path, folders)
}

pub(crate) fn build_dock_in(
    cache_dir: &Path,
    workspace_path: &Path,
    folders: &[ResolvedFolder],
) -> Result<PathBuf> {
    let workspace_abs = absolute_workspace_path(workspace_path)?;
    let dock_root = cache_dir
        .join("zed-workspace-dock")
        .join("docks")
        .join(dock_name(workspace_path, &workspace_abs));

    prepare_dock_dir(&dock_root, &workspace_abs)?;

    for folder in folders {
        create_symlink(&folder.target, &dock_root.join(folder.name.as_str()))?;
    }

    write_marker(&dock_root, &workspace_abs, folders)?;

    Ok(dock_root)
}

fn prepare_dock_dir(dock_root: &Path, workspace_path: &Path) -> Result<()> {
    if !dock_root.exists() {
        fs::create_dir_all(dock_root)?;
        return Ok(());
    }

    if !dock_root.is_dir() {
        return Err(AppError::DockPathNotDirectory {
            path: dock_root.to_path_buf(),
        });
    }

    let marker = read_marker(dock_root)?;
    if marker.workspace_path != workspace_path {
        return Err(AppError::DockWorkspacePathMismatch {
            marker_workspace_path: marker.workspace_path,
            workspace_path: workspace_path.to_path_buf(),
        });
    }

    let expected_links = marker
        .links
        .iter()
        .map(|link| link.name.as_str())
        .collect::<HashSet<_>>();

    for entry in fs::read_dir(dock_root)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == MARKER_FILE {
            continue;
        }

        if !expected_links.contains(file_name.as_ref()) {
            return Err(AppError::UnmanagedDockContent { path: entry.path() });
        }

        if !entry.file_type()?.is_symlink() {
            return Err(AppError::ManagedDockEntryNotSymlink { path: entry.path() });
        }

        remove_managed_link(&entry.path())?;
    }

    Ok(())
}

fn read_marker(dock_root: &Path) -> Result<DockMarker> {
    let marker_path = dock_root.join(MARKER_FILE);

    if !marker_path.exists() {
        return Err(AppError::DockMissingMarker {
            path: dock_root.to_path_buf(),
        });
    }

    let content = fs::read_to_string(marker_path)?;
    let marker: DockMarker = serde_json::from_str(&content)?;

    if marker.version != MARKER_VERSION {
        return Err(AppError::UnsupportedDockMarkerVersion {
            version: marker.version,
        });
    }

    Ok(marker)
}

fn write_marker(dock_root: &Path, workspace_path: &Path, folders: &[ResolvedFolder]) -> Result<()> {
    let marker = DockMarker {
        version: MARKER_VERSION,
        workspace_path: workspace_path.to_path_buf(),
        links: folders
            .iter()
            .map(|folder| DockLink {
                name: folder.name.as_str().to_string(),
                target: folder.target.clone(),
            })
            .collect(),
    };
    let content = serde_json::to_string_pretty(&marker)?;

    fs::write(dock_root.join(MARKER_FILE), format!("{content}\n"))?;

    Ok(())
}

fn absolute_workspace_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }

    let current_dir = std::env::current_dir()?;
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    })
}

fn dock_name(workspace_path: &Path, workspace_abs: &Path) -> String {
    let stem = workspace_path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_else(|| "workspace".into());
    let slug = slugify(&stem);
    let mut hasher = Sha256::new();
    hasher.update(workspace_abs.to_string_lossy().as_bytes());
    let hash = hex::encode(hasher.finalize());

    format!("{slug}-{}", &hash[..12])
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "workspace".to_string()
    } else {
        slug
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if link.exists() || link.symlink_metadata().is_ok() {
        return Err(AppError::DockLinkPathExists {
            path: link.to_path_buf(),
        });
    }

    std::os::unix::fs::symlink(target, link)?;

    Ok(())
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if link.exists() || link.symlink_metadata().is_ok() {
        return Err(AppError::DockLinkPathExists {
            path: link.to_path_buf(),
        });
    }

    std::os::windows::fs::symlink_dir(target, link).map_err(|source| AppError::WindowsSymlink {
        path: link.to_path_buf(),
        source,
    })?;

    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_target: &Path, _link: &Path) -> Result<()> {
    Err(AppError::UnsupportedSymlinkPlatform)
}

#[cfg(windows)]
fn remove_managed_link(path: &Path) -> Result<()> {
    fs::remove_dir(path)?;

    Ok(())
}

#[cfg(not(windows))]
fn remove_managed_link(path: &Path) -> Result<()> {
    fs::remove_file(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use serde_json::Value;
    use tempfile::tempdir;

    use super::*;
    use crate::workspace::LinkName;

    const MARKER_SCHEMA: &str = include_str!("../resources/schemas/zed-dock-marker.schema.json");

    #[test]
    fn builds_dock_with_symlinks_and_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        fs::write(&workspace, "{}").unwrap();

        let dock_root = build_dock_in(
            temp.path(),
            &workspace,
            &[ResolvedFolder {
                name: LinkName::new("api").unwrap(),
                target: project.clone(),
            }],
        )
        .unwrap();

        assert!(dock_root.join(MARKER_FILE).exists());
        assert!(
            dock_root
                .join("api")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(read_link_target(&dock_root.join("api")), project);
    }

    #[test]
    fn marker_schema_tracks_current_marker_version() {
        let schema: Value = serde_json::from_str(MARKER_SCHEMA).unwrap();

        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["properties"]["version"]["const"], MARKER_VERSION);
    }

    #[test]
    fn generated_marker_matches_published_schema_shape() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        fs::write(&workspace, "{}").unwrap();

        let dock_root = build_dock_in(
            temp.path(),
            &workspace,
            &[ResolvedFolder {
                name: LinkName::new("api").unwrap(),
                target: project.clone(),
            }],
        )
        .unwrap();
        let marker: Value =
            serde_json::from_str(&fs::read_to_string(dock_root.join(MARKER_FILE)).unwrap())
                .unwrap();
        let marker = marker.as_object().expect("marker must be a JSON object");
        let links = marker["links"].as_array().expect("links must be an array");
        let first_link = links[0].as_object().expect("link must be a JSON object");

        assert_eq!(marker.len(), 3);
        assert_eq!(marker["version"], MARKER_VERSION);
        assert!(
            marker["workspace_path"]
                .as_str()
                .is_some_and(|path| !path.is_empty())
        );
        assert_eq!(first_link.len(), 2);
        assert_eq!(first_link["name"], "api");
        assert_eq!(first_link["target"], project.to_string_lossy().into_owned());
    }

    #[test]
    fn rebuilds_dock_idempotently() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        fs::write(&workspace, "{}").unwrap();
        let folders = [ResolvedFolder {
            name: LinkName::new("api").unwrap(),
            target: project,
        }];

        let first = build_dock_in(temp.path(), &workspace, &folders).unwrap();
        let second = build_dock_in(temp.path(), &workspace, &folders).unwrap();

        assert_eq!(first, second);
        assert!(
            second
                .join("api")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn aborts_when_marker_belongs_to_another_workspace() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        let other_workspace = temp.path().join("other.code-workspace");
        fs::write(&workspace, "{}").unwrap();
        fs::write(&other_workspace, "{}").unwrap();
        let folders = [ResolvedFolder {
            name: LinkName::new("api").unwrap(),
            target: project.clone(),
        }];
        let dock_root = build_dock_in(temp.path(), &workspace, &folders).unwrap();
        let marker = serde_json::json!({
            "version": MARKER_VERSION,
            "workspace_path": other_workspace.canonicalize().unwrap(),
            "links": [{ "name": "api", "target": project }]
        });
        fs::write(
            dock_root.join(MARKER_FILE),
            format!("{}\n", serde_json::to_string_pretty(&marker).unwrap()),
        )
        .unwrap();

        let error = build_dock_in(temp.path(), &workspace, &folders)
            .unwrap_err()
            .to_string();

        assert!(error.contains("dock marker belongs"));
    }

    #[test]
    fn aborts_when_dock_exists_without_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        fs::write(&workspace, "{}").unwrap();
        let dock_root = temp
            .path()
            .join("zed-workspace-dock")
            .join("docks")
            .join(dock_name(&workspace, &workspace.canonicalize().unwrap()));
        fs::create_dir_all(&dock_root).unwrap();

        let error = build_dock_in(
            temp.path(),
            &workspace,
            &[ResolvedFolder {
                name: LinkName::new("api").unwrap(),
                target: project,
            }],
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("without marker"));
    }

    #[test]
    fn aborts_when_marker_owned_dock_has_unmanaged_content() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace = temp.path().join("demo.code-workspace");
        fs::write(&workspace, "{}").unwrap();
        let folders = [ResolvedFolder {
            name: LinkName::new("api").unwrap(),
            target: project,
        }];
        let dock_root = build_dock_in(temp.path(), &workspace, &folders).unwrap();
        fs::write(dock_root.join("notes.txt"), "do not delete").unwrap();

        let error = build_dock_in(temp.path(), &workspace, &folders)
            .unwrap_err()
            .to_string();

        assert!(error.contains("unmanaged content"));
        assert!(dock_root.join("notes.txt").exists());
    }

    fn read_link_target(path: &Path) -> PathBuf {
        fs::read_link(path).unwrap()
    }
}
