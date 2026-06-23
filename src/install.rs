use std::{
    env, fs,
    path::{Path, PathBuf},
};

use std::collections::HashSet;

use serde_json::Value;

use crate::error::{AppError, Result};

const TASK_TEMPLATES: &str = include_str!("../resources/zed-tasks.json");
const COMMAND_PLACEHOLDER: &str = "__ZED_WORKSPACE_DOCK_COMMAND__";

pub(crate) fn install_default_tasks(
    command: Option<&Path>,
    tasks_path: Option<&Path>,
) -> Result<()> {
    let tasks_path = match tasks_path {
        Some(tasks_path) => tasks_path.to_path_buf(),
        None => default_tasks_path()?,
    };
    let command = install_command(command)?;

    if is_target_binary(&command) {
        eprintln!(
            "warning: installing command from build output: {}",
            command.display()
        );
    }

    install_tasks_at(&tasks_path, &command)
}

fn default_tasks_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;

    Ok(zed_tasks_path(&home_dir))
}

fn zed_tasks_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".config").join("zed").join("tasks.json")
}

pub(crate) fn install_tasks_at(tasks_path: &Path, command: &Path) -> Result<()> {
    let mut tasks = if tasks_path.exists() {
        read_tasks_document(tasks_path)?
    } else {
        Vec::new()
    };

    merge_tasks(&mut tasks, command)?;

    if let Some(parent) = tasks_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(
        tasks_path,
        format!("{}\n", serde_json::to_string_pretty(&tasks)?),
    )?;

    Ok(())
}

fn read_tasks_document(tasks_path: &Path) -> Result<Vec<Value>> {
    let document: Value = serde_json::from_str(&fs::read_to_string(tasks_path)?)?;

    match document {
        Value::Array(tasks) => Ok(tasks),
        _ => Err(AppError::InvalidTasksJsonRoot),
    }
}

fn merge_tasks(tasks: &mut Vec<Value>, command: &Path) -> Result<()> {
    let templates = task_templates(command)?;
    let labels = managed_labels(&templates)?;

    tasks.retain(|task| {
        task.get("label")
            .and_then(Value::as_str)
            .map(|label| !labels.contains(label))
            .unwrap_or(true)
    });
    tasks.extend(templates);

    Ok(())
}

fn task_templates(command: &Path) -> Result<Vec<Value>> {
    let mut templates: Vec<Value> = serde_json::from_str(TASK_TEMPLATES)?;
    let command = command.to_string_lossy().into_owned();

    for template in &mut templates {
        let object = template
            .as_object_mut()
            .ok_or(AppError::InvalidTaskTemplateRoot)?;
        let Some(Value::String(template_command)) = object.get("command") else {
            return Err(AppError::InvalidTaskTemplateCommand {
                placeholder: COMMAND_PLACEHOLDER,
            });
        };

        if template_command != COMMAND_PLACEHOLDER {
            return Err(AppError::InvalidTaskTemplateCommand {
                placeholder: COMMAND_PLACEHOLDER,
            });
        }

        object.insert("command".to_string(), Value::String(command.clone()));
    }

    Ok(templates)
}

fn managed_labels(tasks: &[Value]) -> Result<HashSet<&str>> {
    tasks
        .iter()
        .map(|task| {
            task.get("label")
                .and_then(Value::as_str)
                .ok_or(AppError::InvalidTaskTemplateLabel)
        })
        .collect()
}

fn install_command(command: Option<&Path>) -> Result<PathBuf> {
    let command = match command {
        Some(command) => command.to_path_buf(),
        None => env::current_exe()?,
    };

    if !command.exists() {
        return Err(AppError::InstallCommandNotFound { path: command });
    }

    fs::canonicalize(command).map_err(AppError::from)
}

fn is_target_binary(command: &Path) -> bool {
    let mut components = command
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        });

    components.any(|component| component == "target")
        && components.any(|component| matches!(component.as_ref(), "debug" | "release"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn installs_tasks_without_duplicate_labels() {
        let temp = tempdir().unwrap();
        let tasks_path = temp.path().join("tasks.json");
        fs::write(
            &tasks_path,
            r#"[
              {"label":"Existing","command":"echo","args":["ok"]},
              {"label":"Zed Workspace Dock: Open","command":"old","args":[]}
            ]"#,
        )
        .unwrap();

        install_tasks_at(&tasks_path, Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();
        install_tasks_at(&tasks_path, Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();

        let tasks: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(tasks_path).unwrap()).unwrap();
        let dock_task_count = tasks
            .iter()
            .filter(|task| {
                task["label"]
                    .as_str()
                    .map(|label| label.starts_with("Zed Workspace Dock:"))
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(dock_task_count, 3);
        assert!(tasks.iter().any(|task| task["label"] == "Existing"));
        assert!(tasks.iter().all(|task| task["command"] != "old"));
    }

    #[test]
    fn task_templates_use_absolute_command_path() {
        let tasks = task_templates(Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();

        assert!(tasks.iter().all(|task| {
            task["command"]
                .as_str()
                .is_some_and(|command| command == "/usr/local/bin/zed-workspace-dock")
        }));
    }

    #[test]
    fn install_tasks_at_creates_root_array_when_missing() {
        let temp = tempdir().unwrap();
        let tasks_path = temp.path().join("tasks.json");

        install_tasks_at(&tasks_path, Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();

        let tasks: Value = serde_json::from_str(&fs::read_to_string(tasks_path).unwrap()).unwrap();
        assert!(tasks.is_array());
    }

    #[test]
    fn install_tasks_at_rejects_non_array_tasks_json() {
        let temp = tempdir().unwrap();
        let tasks_path = temp.path().join("tasks.json");
        fs::write(&tasks_path, r#"{"tasks":[]}"#).unwrap();

        let error = install_tasks_at(&tasks_path, Path::new("/usr/local/bin/zed-workspace-dock"))
            .unwrap_err()
            .to_string();

        assert!(error.contains("JSON array"));
    }

    #[test]
    fn task_templates_use_zed_file_argument() {
        let tasks = task_templates(Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();

        assert!(tasks.iter().all(|task| task["args"][1] == "$ZED_FILE"));
    }

    #[test]
    fn task_templates_replace_command_placeholder() {
        let tasks = task_templates(Path::new("/usr/local/bin/zed-workspace-dock")).unwrap();
        let content = serde_json::to_string(&tasks).unwrap();

        assert!(!content.contains(COMMAND_PLACEHOLDER));
    }

    #[test]
    fn is_target_binary_detects_debug_binary_paths() {
        assert!(is_target_binary(Path::new(
            "/tmp/project/target/debug/zed-workspace-dock"
        )));
    }

    #[test]
    fn zed_tasks_path_uses_zed_global_tasks_file_under_home() {
        assert_eq!(
            zed_tasks_path(Path::new("/Users/alice")),
            Path::new("/Users/alice")
                .join(".config")
                .join("zed")
                .join("tasks.json")
        );
    }
}
