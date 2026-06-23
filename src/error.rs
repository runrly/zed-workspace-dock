use std::{path::PathBuf, process::ExitStatus};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("MVP only supports .code-workspace files: {path}")]
    UnsupportedWorkspaceExtension { path: PathBuf },
    #[error(r#""zed-dock.mode" is required when "zed-dock" exists"#)]
    MissingDockMode,
    #[error("workspace path must have a parent directory")]
    MissingWorkspaceParent,
    #[error("folder path does not exist or cannot be resolved: {path} ({source})")]
    FolderResolve {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("folder target is not a directory: {path}")]
    FolderTargetNotDirectory { path: PathBuf },
    #[error("folder target has no usable basename: {path}")]
    FolderTargetMissingBasename { path: PathBuf },
    #[error("duplicate folder name: {name}")]
    DuplicateFolderName { name: String },
    #[error("invalid dock link name {name:?}: {reason}")]
    InvalidLinkName { name: String, reason: &'static str },
    #[error("folder entries must use name=path or path")]
    InvalidFolderArgument,
    #[error("folder path cannot be empty")]
    EmptyFolderPath,
    #[error("cache directory not found")]
    CacheDirNotFound,
    #[error("home directory not found")]
    HomeDirNotFound,
    #[error("config directory not found")]
    ConfigDirNotFound,
    #[error("workspace name and --output cannot be used together")]
    ConflictingWorkspaceDestination,
    #[error("invalid workspace name {name:?}: {reason}")]
    InvalidWorkspaceName { name: String, reason: &'static str },
    #[error("registered workspace already exists; use --force to overwrite: {path}")]
    WorkspaceAlreadyExists { path: PathBuf },
    #[error("registered workspace not found: {name}")]
    RegisteredWorkspaceNotFound { name: String },
    #[error("could not generate an unused workspace name")]
    WorkspaceNameGenerationExhausted,
    #[error(transparent)]
    Random(#[from] getrandom::Error),
    #[error("dock path exists and is not a directory: {path}")]
    DockPathNotDirectory { path: PathBuf },
    #[error("dock exists without marker; refusing to modify: {path}")]
    DockMissingMarker { path: PathBuf },
    #[error("unsupported dock marker version: {version}")]
    UnsupportedDockMarkerVersion { version: u8 },
    #[error("dock contains unmanaged content: {path}")]
    UnmanagedDockContent { path: PathBuf },
    #[error("managed dock entry is not a symlink: {path}")]
    ManagedDockEntryNotSymlink { path: PathBuf },
    #[error("dock marker belongs to {marker_workspace_path}, not {workspace_path}")]
    DockWorkspacePathMismatch {
        marker_workspace_path: PathBuf,
        workspace_path: PathBuf,
    },
    #[error("dock link path already exists: {path}")]
    DockLinkPathExists { path: PathBuf },
    #[error("symlink dock mode is only supported on Linux and macOS in the MVP")]
    UnsupportedSymlinkPlatform,
    #[error("Zed exited with status: {status}")]
    ZedExited { status: ExitStatus },
    #[error("failed to launch Zed binary {path}: {source}")]
    LaunchZed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Zed tasks.json must be a JSON array")]
    InvalidTasksJsonRoot,
    #[error("Zed task template must be a JSON object")]
    InvalidTaskTemplateRoot,
    #[error(r#"Zed task template field "command" must be {placeholder:?}"#)]
    InvalidTaskTemplateCommand { placeholder: &'static str },
    #[error(r#"Zed task template field "label" must be a string"#)]
    InvalidTaskTemplateLabel,
    #[error("current executable has no file name")]
    CurrentExecutableMissingName,
    #[error("install command does not exist: {path}")]
    InstallCommandNotFound { path: PathBuf },
}
