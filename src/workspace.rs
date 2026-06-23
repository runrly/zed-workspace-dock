use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    cli::Mode,
    error::{AppError, Result},
};

const APP_DIR: &str = "zed-workspace-dock";
const RESERVED_DOCK_METADATA_NAMES: [&str; 1] = [".zed-dock.json"];
const WORKSPACE_EXTENSION: &str = "code-workspace";
const WORKSPACES_DIR: &str = "workspaces";
const GENERATED_NAME_ATTEMPTS: usize = 16;

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceFile {
    #[serde(default)]
    folders: Vec<WorkspaceFolder>,
    #[serde(rename = "zed-dock", skip_serializing_if = "Option::is_none")]
    zed_dock: Option<DockConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DockConfig {
    mode: Option<Mode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceFolder {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedFolder {
    pub name: LinkName,
    pub target: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RegisteredWorkspace {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct LinkName(String);

impl LinkName {
    pub(crate) fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_link_name(&name)?;
        Ok(Self(name))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn read_workspace_file(path: &Path) -> Result<WorkspaceFile> {
    ensure_code_workspace_path(path)?;

    let content = fs::read_to_string(path)?;
    let workspace = serde_json::from_str(&content)?;

    Ok(workspace)
}

pub(crate) fn create_workspace_file(
    name: Option<&str>,
    output: Option<&Path>,
    mode: Mode,
    folders: &[String],
    force: bool,
) -> Result<PathBuf> {
    if name.is_some() && output.is_some() {
        return Err(AppError::ConflictingWorkspaceDestination);
    }

    match output {
        Some(output) => {
            write_workspace_file(output, mode, folders)?;
            fs::canonicalize(output).map_err(AppError::from)
        }
        None => create_registered_workspace(name, mode, folders, force),
    }
}

pub(crate) fn resolve_workspace_reference(workspace: &Path) -> Result<PathBuf> {
    if !is_simple_path(workspace) {
        return Ok(workspace.to_path_buf());
    }

    let name = workspace
        .to_str()
        .ok_or_else(|| AppError::InvalidWorkspaceName {
            name: workspace.to_string_lossy().into_owned(),
            reason: "must be valid UTF-8",
        })?;
    let name = name.strip_suffix(".code-workspace").unwrap_or(name);
    let path = registered_workspace_path(name)?;

    if path.exists() {
        return Ok(path);
    }

    if workspace.exists() {
        return Ok(workspace.to_path_buf());
    }

    Err(AppError::RegisteredWorkspaceNotFound {
        name: name.to_string(),
    })
}

pub(crate) fn list_registered_workspaces() -> Result<Vec<RegisteredWorkspace>> {
    list_registered_workspaces_in(&registered_workspaces_dir()?)
}

pub(crate) fn write_workspace_file(output: &Path, mode: Mode, folders: &[String]) -> Result<()> {
    ensure_code_workspace_path(output)?;

    let workspace = WorkspaceFile {
        folders: folders
            .iter()
            .map(|folder| parse_folder_arg(folder))
            .collect::<Result<Vec<_>>>()?,
        zed_dock: Some(DockConfig { mode: Some(mode) }),
    };

    let content = serde_json::to_string_pretty(&workspace)?;
    fs::write(output, format!("{content}\n"))?;

    Ok(())
}

fn create_registered_workspace(
    name: Option<&str>,
    mode: Mode,
    folders: &[String],
    force: bool,
) -> Result<PathBuf> {
    let workspaces_dir = registered_workspaces_dir()?;
    let current_dir = std::env::current_dir()?;

    create_registered_workspace_in(&workspaces_dir, name, mode, folders, force, &current_dir)
}

fn create_registered_workspace_in(
    workspaces_dir: &Path,
    name: Option<&str>,
    mode: Mode,
    folders: &[String],
    force: bool,
    base_dir: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(workspaces_dir)?;
    let workspace = WorkspaceFile {
        folders: folders
            .iter()
            .map(|folder| parse_registered_folder_arg(folder, base_dir))
            .collect::<Result<Vec<_>>>()?,
        zed_dock: Some(DockConfig { mode: Some(mode) }),
    };
    let content = serde_json::to_string_pretty(&workspace)?;

    let output = match name {
        Some(name) => {
            let output = registered_workspace_path_in(workspaces_dir, name)?;
            ensure_can_write_registered_workspace(&output, force)?;
            output
        }
        None => generated_registered_workspace_path(workspaces_dir)?,
    };

    fs::write(&output, format!("{content}\n"))?;

    Ok(output)
}

fn registered_workspaces_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or(AppError::ConfigDirNotFound)?;

    Ok(registered_workspaces_dir_in(&config_dir))
}

fn registered_workspaces_dir_in(config_dir: &Path) -> PathBuf {
    config_dir.join(APP_DIR).join(WORKSPACES_DIR)
}

fn registered_workspace_path(name: &str) -> Result<PathBuf> {
    registered_workspace_path_in(&registered_workspaces_dir()?, name)
}

fn registered_workspace_path_in(workspaces_dir: &Path, name: &str) -> Result<PathBuf> {
    validate_workspace_name(name)?;

    Ok(workspaces_dir.join(format!("{name}.{WORKSPACE_EXTENSION}")))
}

fn generated_registered_workspace_path(workspaces_dir: &Path) -> Result<PathBuf> {
    for _ in 0..GENERATED_NAME_ATTEMPTS {
        let name = generated_workspace_name()?;
        let output = registered_workspace_path_in(workspaces_dir, &name)?;

        if !output.exists() {
            return Ok(output);
        }
    }

    Err(AppError::WorkspaceNameGenerationExhausted)
}

fn generated_workspace_name() -> Result<String> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes)?;

    Ok(generated_workspace_name_from_bytes(bytes))
}

fn generated_workspace_name_from_bytes(bytes: [u8; 8]) -> String {
    format!("ws-{}", hex::encode(bytes))
}

fn ensure_can_write_registered_workspace(output: &Path, force: bool) -> Result<()> {
    if output.exists() && !force {
        return Err(AppError::WorkspaceAlreadyExists {
            path: output.to_path_buf(),
        });
    }

    Ok(())
}

fn list_registered_workspaces_in(workspaces_dir: &Path) -> Result<Vec<RegisteredWorkspace>> {
    if !workspaces_dir.exists() {
        return Ok(Vec::new());
    }

    let mut workspaces = Vec::new();
    for entry in fs::read_dir(workspaces_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some(WORKSPACE_EXTENSION) {
            continue;
        }

        let Some(name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };

        workspaces.push(RegisteredWorkspace {
            name: name.to_string(),
            path,
        });
    }

    workspaces.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(workspaces)
}

impl WorkspaceFile {
    pub fn open_mode(&self, cli_mode: Option<Mode>) -> Result<Mode> {
        if let Some(mode) = cli_mode {
            return Ok(mode);
        }

        match &self.zed_dock {
            Some(config) => config.mode.ok_or(AppError::MissingDockMode),
            None => Ok(Mode::Folders),
        }
    }

    pub fn folder_targets(&self, workspace_path: &Path) -> Result<Vec<PathBuf>> {
        let workspace_dir = workspace_dir(workspace_path)?;

        self.folders
            .iter()
            .map(|folder| resolve_folder_target(workspace_dir, folder))
            .collect()
    }

    pub fn resolved_dock_folders(&self, workspace_path: &Path) -> Result<Vec<ResolvedFolder>> {
        let workspace_dir = workspace_dir(workspace_path)?;
        let mut names = HashSet::new();
        let mut folders = Vec::with_capacity(self.folders.len());

        for folder in &self.folders {
            let target = resolve_folder_target(workspace_dir, folder)?;
            let link_name = folder_link_name(folder, &target)?;

            if !names.insert(case_insensitive_link_key(&link_name)) {
                return Err(AppError::DuplicateFolderName {
                    name: link_name.as_str().to_string(),
                });
            }

            folders.push(ResolvedFolder {
                name: link_name,
                target,
            });
        }

        Ok(folders)
    }
}

fn workspace_dir(workspace_path: &Path) -> Result<&Path> {
    workspace_path
        .parent()
        .ok_or(AppError::MissingWorkspaceParent)
}

fn resolve_folder_target(workspace_dir: &Path, folder: &WorkspaceFolder) -> Result<PathBuf> {
    let joined = if folder.path.is_absolute() {
        folder.path.clone()
    } else {
        workspace_dir.join(&folder.path)
    };
    let target = fs::canonicalize(&joined).map_err(|source| AppError::FolderResolve {
        path: joined.clone(),
        source,
    })?;

    if !target.is_dir() {
        return Err(AppError::FolderTargetNotDirectory { path: target });
    }

    Ok(target)
}

fn case_insensitive_link_key(name: &LinkName) -> String {
    name.as_str().to_lowercase()
}

fn folder_link_name(folder: &WorkspaceFolder, target: &Path) -> Result<LinkName> {
    let name = folder
        .name
        .clone()
        .or_else(|| {
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .ok_or_else(|| AppError::FolderTargetMissingBasename {
            path: target.to_path_buf(),
        })?;

    LinkName::new(name)
}

fn ensure_code_workspace_path(path: &Path) -> Result<()> {
    if path.extension().and_then(|extension| extension.to_str()) != Some(WORKSPACE_EXTENSION) {
        return Err(AppError::UnsupportedWorkspaceExtension {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

fn validate_workspace_name(name: &str) -> Result<()> {
    let reason = if name.is_empty() {
        Some("empty name")
    } else if matches!(name, "." | "..") {
        Some("reserved relative path segment")
    } else if name.contains('\0') {
        Some("contains NUL byte")
    } else if name.contains('/') || name.contains('\\') {
        Some("contains path separator")
    } else if name.contains('.') {
        Some("must not include an extension")
    } else if !is_simple_path(Path::new(name)) {
        Some("must be a single path segment")
    } else {
        None
    };

    if let Some(reason) = reason {
        return Err(AppError::InvalidWorkspaceName {
            name: name.to_string(),
            reason,
        });
    }

    Ok(())
}

fn is_simple_path(path: &Path) -> bool {
    !path.is_absolute() && path.components().count() == 1
}

fn validate_link_name(name: &str) -> Result<()> {
    let reason = if name.is_empty() {
        Some("empty name")
    } else if matches!(name, "." | "..") {
        Some("reserved relative path segment")
    } else if name.contains('\0') {
        Some("contains NUL byte")
    } else if name.contains('/') || name.contains('\\') {
        Some("contains path separator")
    } else if RESERVED_DOCK_METADATA_NAMES
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
    {
        Some("reserved dock metadata name")
    } else if Path::new(name).is_absolute() {
        Some("absolute path")
    } else {
        let mut components = Path::new(name).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(component)), None) if component == OsStr::new(name) => None,
            (
                Some(
                    Component::ParentDir
                    | Component::CurDir
                    | Component::RootDir
                    | Component::Prefix(_),
                ),
                _,
            ) => Some("contains path component"),
            _ => Some("must be a single path segment"),
        }
    };

    if let Some(reason) = reason {
        return Err(AppError::InvalidLinkName {
            name: name.to_string(),
            reason,
        });
    }

    Ok(())
}

fn parse_registered_folder_arg(input: &str, base_dir: &Path) -> Result<WorkspaceFolder> {
    let mut folder = parse_folder_arg(input)?;
    folder.path = canonicalize_folder_arg(base_dir, &folder.path)?;

    Ok(folder)
}

fn canonicalize_folder_arg(base_dir: &Path, path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };

    fs::canonicalize(&joined).map_err(|source| AppError::FolderResolve {
        path: joined,
        source,
    })
}

fn parse_folder_arg(input: &str) -> Result<WorkspaceFolder> {
    if let Some((name, path)) = input.split_once('=') {
        if name.is_empty() || path.is_empty() {
            return Err(AppError::InvalidFolderArgument);
        }
        LinkName::new(name)?;

        return Ok(WorkspaceFolder {
            name: Some(name.to_string()),
            path: PathBuf::from(path),
        });
    }

    if input.is_empty() {
        return Err(AppError::EmptyFolderPath);
    }

    Ok(WorkspaceFolder {
        name: None,
        path: PathBuf::from(input),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::tempdir;

    use super::*;

    const CODE_WORKSPACE_SCHEMA: &str =
        include_str!("../resources/schemas/code-workspace.schema.json");

    #[test]
    fn parses_workspace_with_dock_mode() {
        let workspace: WorkspaceFile = serde_json::from_str(
            r#"{"folders":[{"name":"api","path":"../api"}],"zed-dock":{"mode":"symlink"}}"#,
        )
        .unwrap();

        assert_eq!(workspace.open_mode(None).unwrap(), Mode::Symlink);
    }

    #[test]
    fn code_workspace_schema_tracks_supported_modes() {
        let schema: Value = serde_json::from_str(CODE_WORKSPACE_SCHEMA).unwrap();

        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(
            schema.get("required").is_none(),
            "schema must match runtime parser, which defaults missing folders to []"
        );
        assert_eq!(
            schema["properties"]["zed-dock"]["properties"]["mode"]["enum"],
            serde_json::json!(["folders", "symlink"])
        );
    }

    #[test]
    fn rejects_jsonc_comments() {
        let error = serde_json::from_str::<WorkspaceFile>(
            r#"{"folders":[{"path":"../api"}] // unsupported comment
            }"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("expected"));
    }

    #[test]
    fn rejects_missing_mode_when_dock_config_exists() {
        let workspace: WorkspaceFile =
            serde_json::from_str(r#"{"folders":[],"zed-dock":{}}"#).unwrap();

        let error = workspace.open_mode(None).unwrap_err().to_string();

        assert!(error.contains("zed-dock.mode"));
    }

    #[test]
    fn resolves_relative_folder_paths_from_workspace_location() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace_path = temp.path().join("workspace/demo.code-workspace");
        fs::create_dir(workspace_path.parent().unwrap()).unwrap();
        let workspace: WorkspaceFile =
            serde_json::from_str(r#"{"folders":[{"name":"api","path":"../api"}]}"#).unwrap();

        let folders = workspace.resolved_dock_folders(&workspace_path).unwrap();

        assert_eq!(folders[0].name.as_str(), "api");
        assert_eq!(folders[0].target, project.canonicalize().unwrap());
    }

    #[test]
    fn folder_targets_ignore_invalid_link_names() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace_path = temp.path().join("demo.code-workspace");
        let workspace: WorkspaceFile =
            serde_json::from_str(r#"{"folders":[{"name":"../x","path":"api"}]}"#).unwrap();

        let folders = workspace.folder_targets(&workspace_path).unwrap();

        assert_eq!(folders, vec![project.canonicalize().unwrap()]);
    }

    #[test]
    fn registered_workspaces_dir_uses_app_config_subdirectory() {
        assert_eq!(
            registered_workspaces_dir_in(Path::new("/tmp/config")),
            Path::new("/tmp/config").join(APP_DIR).join(WORKSPACES_DIR)
        );
    }

    #[test]
    fn generated_workspace_name_uses_short_hex_hash() {
        let name = generated_workspace_name_from_bytes([0xab, 0xcd, 0, 1, 2, 3, 4, 0xff]);

        assert_eq!(name, "ws-abcd0001020304ff");
    }

    #[test]
    fn rejects_workspace_name_with_extension() {
        let error =
            registered_workspace_path_in(Path::new("/tmp/workspaces"), "demo.code-workspace")
                .unwrap_err()
                .to_string();

        assert!(error.contains("must not include an extension"));
    }

    #[test]
    fn create_registered_workspace_canonicalizes_folder_paths() {
        let temp = tempdir().unwrap();
        let workspaces_dir = temp.path().join("registry");
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let folders = vec!["api=api".to_string()];

        let output = create_registered_workspace_in(
            &workspaces_dir,
            Some("demo"),
            Mode::Symlink,
            &folders,
            false,
            temp.path(),
        )
        .unwrap();

        let workspace: WorkspaceFile =
            serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();

        assert_eq!(output, workspaces_dir.join("demo.code-workspace"));
        assert_eq!(workspace.folders[0].path, project.canonicalize().unwrap());
    }

    #[test]
    fn create_registered_workspace_requires_force_to_overwrite() {
        let temp = tempdir().unwrap();
        let workspaces_dir = temp.path().join("registry");
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let folders = vec!["api=api".to_string()];

        create_registered_workspace_in(
            &workspaces_dir,
            Some("demo"),
            Mode::Symlink,
            &folders,
            false,
            temp.path(),
        )
        .unwrap();

        let error = create_registered_workspace_in(
            &workspaces_dir,
            Some("demo"),
            Mode::Symlink,
            &folders,
            false,
            temp.path(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("--force"));
    }

    #[test]
    fn create_registered_workspace_overwrites_with_force() {
        let temp = tempdir().unwrap();
        let workspaces_dir = temp.path().join("registry");
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let folders = vec!["api=api".to_string()];

        create_registered_workspace_in(
            &workspaces_dir,
            Some("demo"),
            Mode::Symlink,
            &folders,
            false,
            temp.path(),
        )
        .unwrap();
        create_registered_workspace_in(
            &workspaces_dir,
            Some("demo"),
            Mode::Folders,
            &folders,
            true,
            temp.path(),
        )
        .unwrap();
        let workspace: WorkspaceFile = serde_json::from_str(
            &fs::read_to_string(workspaces_dir.join("demo.code-workspace")).unwrap(),
        )
        .unwrap();

        assert_eq!(workspace.open_mode(None).unwrap(), Mode::Folders);
    }

    #[test]
    fn list_registered_workspaces_returns_sorted_code_workspace_files() {
        let temp = tempdir().unwrap();
        let workspaces_dir = temp.path().join("registry");
        fs::create_dir(&workspaces_dir).unwrap();
        fs::write(workspaces_dir.join("b.code-workspace"), "{}").unwrap();
        fs::write(workspaces_dir.join("ignored.txt"), "{}").unwrap();
        fs::write(workspaces_dir.join("a.code-workspace"), "{}").unwrap();

        let workspaces = list_registered_workspaces_in(&workspaces_dir).unwrap();

        assert_eq!(
            workspaces,
            vec![
                RegisteredWorkspace {
                    name: "a".to_string(),
                    path: workspaces_dir.join("a.code-workspace")
                },
                RegisteredWorkspace {
                    name: "b".to_string(),
                    path: workspaces_dir.join("b.code-workspace")
                }
            ]
        );
    }

    #[test]
    fn rejects_duplicate_link_names() {
        let temp = tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let workspace_path = temp.path().join("demo.code-workspace");
        let workspace: WorkspaceFile = serde_json::from_str(
            r#"{"folders":[{"name":"api","path":"first"},{"name":"api","path":"second"}]}"#,
        )
        .unwrap();

        let error = workspace
            .resolved_dock_folders(&workspace_path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("duplicate folder name"));
    }

    #[test]
    fn rejects_case_insensitive_duplicate_link_names() {
        let temp = tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let workspace_path = temp.path().join("demo.code-workspace");
        let workspace: WorkspaceFile = serde_json::from_str(
            r#"{"folders":[{"name":"api","path":"first"},{"name":"API","path":"second"}]}"#,
        )
        .unwrap();

        let error = workspace
            .resolved_dock_folders(&workspace_path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("duplicate folder name"));
    }

    #[test]
    fn rejects_non_code_workspace_paths() {
        let temp = tempdir().unwrap();
        let workspace_path = temp.path().join("demo.zed-workspace");
        fs::write(&workspace_path, "{}").unwrap();

        let error = read_workspace_file(&workspace_path)
            .unwrap_err()
            .to_string();

        assert!(error.contains(".code-workspace"));
    }

    #[test]
    fn rejects_parent_dir_link_name() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace_path = temp.path().join("demo.code-workspace");
        let workspace: WorkspaceFile =
            serde_json::from_str(r#"{"folders":[{"name":"../x","path":"api"}]}"#).unwrap();

        let error = workspace
            .resolved_dock_folders(&workspace_path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("invalid dock link name"));
    }

    #[test]
    fn rejects_absolute_link_name() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("api");
        fs::create_dir(&project).unwrap();
        let workspace_path = temp.path().join("demo.code-workspace");
        let workspace: WorkspaceFile =
            serde_json::from_str(r#"{"folders":[{"name":"/tmp/x","path":"api"}]}"#).unwrap();

        let error = workspace
            .resolved_dock_folders(&workspace_path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("invalid dock link name"));
    }

    #[test]
    fn rejects_reserved_link_name() {
        let error = LinkName::new("..").unwrap_err().to_string();

        assert!(error.contains("reserved relative path segment"));
    }

    #[test]
    fn rejects_backslash_link_name() {
        let error = LinkName::new("api\\web").unwrap_err().to_string();

        assert!(error.contains("path separator"));
    }

    #[test]
    fn rejects_marker_link_name() {
        let error = LinkName::new(".zed-dock.json").unwrap_err().to_string();

        assert!(error.contains("reserved dock metadata name"));
    }

    #[test]
    fn rejects_marker_link_name_case_insensitively() {
        let error = LinkName::new(".ZED-DOCK.JSON").unwrap_err().to_string();

        assert!(error.contains("reserved dock metadata name"));
    }
}
