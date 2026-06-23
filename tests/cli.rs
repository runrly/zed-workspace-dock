use std::{
    error::Error,
    fs,
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;
use tempfile::tempdir;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_zed-workspace-dock")
}

fn run(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(binary()).args(args).output()?)
}

fn run_with_home_and_cwd(args: &[&str], home: &Path, cwd: &Path) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(binary())
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .current_dir(cwd)
        .output()?)
}

fn stdout_line(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn create_writes_code_workspace_with_dock_config() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let workspace = temp.path().join("demo.code-workspace");
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "create",
        "--output",
        &workspace_arg,
        "--mode",
        "symlink",
        "--folder",
        "api=../api",
        "--folder",
        "../web",
    ])?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: Value = serde_json::from_str(&fs::read_to_string(&workspace)?)?;

    assert_eq!(document["zed-dock"]["mode"], "symlink");
    assert_eq!(document["folders"][0]["name"], "api");
    assert_eq!(document["folders"][0]["path"], "../api");
    assert_eq!(document["folders"][1]["path"], "../web");
    assert_eq!(
        stdout_line(&output),
        workspace.canonicalize()?.to_string_lossy().into_owned()
    );

    Ok(())
}

#[test]
fn create_without_name_registers_generated_workspace_and_prints_path() -> Result<(), Box<dyn Error>>
{
    let temp = tempdir()?;
    let home = temp.path().join("home");
    let project_root = temp.path().join("project");
    let api = project_root.join("api");
    fs::create_dir_all(&api)?;

    let output = run_with_home_and_cwd(
        &["create", "--mode", "symlink", "--folder", "api=api"],
        &home,
        &project_root,
    )?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let workspace = stdout_line(&output);
    let workspace_path = Path::new(&workspace);
    let file_name = workspace_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("workspace path must have a UTF-8 file name");
    let document: Value = serde_json::from_str(&fs::read_to_string(workspace_path)?)?;

    assert!(file_name.starts_with("ws-"));
    assert!(file_name.ends_with(".code-workspace"));
    assert_eq!(
        document["folders"][0]["path"],
        api.canonicalize()?.to_string_lossy().into_owned()
    );

    Ok(())
}

#[test]
fn create_list_and_open_registered_workspace_by_name() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let home = temp.path().join("home");
    let project_root = temp.path().join("project");
    let api = project_root.join("api");
    fs::create_dir_all(&api)?;

    let create = run_with_home_and_cwd(
        &[
            "create", "custom", "--mode", "folders", "--folder", "api=api",
        ],
        &home,
        &project_root,
    )?;

    assert!(
        create.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let workspace = stdout_line(&create);
    fs::create_dir(project_root.join("custom"))?;
    let list = run_with_home_and_cwd(&["list"], &home, &project_root)?;
    let open = run_with_home_and_cwd(
        &["open", "custom", "--reuse", "--zed-bin", "/bin/echo"],
        &home,
        &project_root,
    )?;
    let open_with_extension = run_with_home_and_cwd(
        &[
            "open",
            "custom.code-workspace",
            "--reuse",
            "--zed-bin",
            "/bin/echo",
        ],
        &home,
        &project_root,
    )?;

    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        format!("custom\t{workspace}")
    );
    assert!(
        open.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&open.stderr)
    );
    assert_eq!(stdout_line(&open), api.canonicalize()?.to_string_lossy());
    assert!(
        open_with_extension.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&open_with_extension.stderr)
    );
    assert_eq!(
        stdout_line(&open_with_extension),
        api.canonicalize()?.to_string_lossy()
    );

    Ok(())
}

#[test]
fn open_folders_mode_invokes_zed_binary_with_resolved_folder() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let project = temp.path().join("api");
    fs::create_dir(&project)?;
    let workspace = temp.path().join("demo.code-workspace");
    fs::write(
        &workspace,
        r#"{
          "folders": [{ "name": "api", "path": "api" }],
          "zed-dock": { "mode": "folders" }
        }"#,
    )?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "open",
        &workspace_arg,
        "--mode",
        "folders",
        "--reuse",
        "--zed-bin",
        "/bin/echo",
    ])?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        project.canonicalize()?.to_string_lossy()
    );

    Ok(())
}

#[test]
fn open_rejects_non_code_workspace_input() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let workspace = temp.path().join("demo.zed-workspace");
    fs::write(&workspace, "{}")?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&["open", &workspace_arg, "--reuse", "--zed-bin", "/bin/echo"])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(".code-workspace"));

    Ok(())
}

#[test]
fn create_rejects_parent_dir_link_name() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let workspace = temp.path().join("demo.code-workspace");
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "create",
        "--output",
        &workspace_arg,
        "--mode",
        "symlink",
        "--folder",
        "../x=/tmp",
    ])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid dock link name"));
    assert!(!workspace.exists());

    Ok(())
}

#[test]
fn open_rejects_parent_dir_link_name_before_launching_zed() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let project = temp.path().join("api");
    fs::create_dir(&project)?;
    let workspace = temp.path().join("demo.code-workspace");
    fs::write(
        &workspace,
        r#"{
          "folders": [{ "name": "../x", "path": "api" }],
          "zed-dock": { "mode": "symlink" }
        }"#,
    )?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "open",
        &workspace_arg,
        "--mode",
        "symlink",
        "--reuse",
        "--zed-bin",
        "/bin/false",
    ])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid dock link name"));

    Ok(())
}

#[test]
fn open_folders_mode_allows_invalid_link_name() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let project = temp.path().join("api");
    fs::create_dir(&project)?;
    let workspace = temp.path().join("demo.code-workspace");
    fs::write(
        &workspace,
        r#"{
          "folders": [{ "name": "../x", "path": "api" }],
          "zed-dock": { "mode": "folders" }
        }"#,
    )?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&["open", &workspace_arg, "--reuse", "--zed-bin", "/bin/echo"])?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        project.canonicalize()?.to_string_lossy()
    );

    Ok(())
}

#[test]
fn install_writes_root_array_from_resource() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let tasks_path = temp.path().join("tasks.json");
    let tasks_path_arg = tasks_path.to_string_lossy().into_owned();
    let command_path = temp.path().join("zed-workspace-dock");
    fs::write(&command_path, "")?;
    let command_arg = command_path.to_string_lossy().into_owned();

    let output = run(&[
        "install",
        "--tasks-path",
        &tasks_path_arg,
        "--command",
        &command_arg,
    ])?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: Value = serde_json::from_str(&fs::read_to_string(tasks_path)?)?;
    let tasks = document.as_array().expect("tasks json must be an array");
    let expected_command = command_path.canonicalize()?.to_string_lossy().into_owned();

    assert_eq!(tasks.len(), 3);
    assert!(tasks.iter().all(|task| task["command"] == expected_command));
    assert!(tasks.iter().all(|task| task["args"][1] == "$ZED_FILE"));

    Ok(())
}

#[test]
fn open_symlink_mode_rejects_marker_link_name_before_launching_zed() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let project = temp.path().join("api");
    fs::create_dir(&project)?;
    let workspace = temp.path().join("demo.code-workspace");
    fs::write(
        &workspace,
        r#"{
          "folders": [{ "name": ".zed-dock.json", "path": "api" }],
          "zed-dock": { "mode": "symlink" }
        }"#,
    )?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "open",
        &workspace_arg,
        "--mode",
        "symlink",
        "--reuse",
        "--zed-bin",
        "/bin/false",
    ])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reserved dock metadata name"));

    Ok(())
}

#[test]
fn open_symlink_rejects_case_insensitive_duplicate_links_before_zed() -> Result<(), Box<dyn Error>>
{
    let temp = tempdir()?;
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    fs::create_dir(&api)?;
    fs::create_dir(&web)?;
    let workspace = temp.path().join("demo.code-workspace");
    fs::write(
        &workspace,
        r#"{
          "folders": [
            { "name": "api", "path": "api" },
            { "name": "API", "path": "web" }
          ],
          "zed-dock": { "mode": "symlink" }
        }"#,
    )?;
    let workspace_arg = workspace.to_string_lossy().into_owned();

    let output = run(&[
        "open",
        &workspace_arg,
        "--mode",
        "symlink",
        "--reuse",
        "--zed-bin",
        "/bin/false",
    ])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate folder name"));

    Ok(())
}
