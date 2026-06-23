# Zed Workspace Dock

Rust CLI for opening Zed workspace files through direct folder mode or a managed symlink dock.

Dock mode creates one marker-protected cache directory with symlinks to the workspace folders, then opens that dock in Zed. This gives the Zed terminal one real root where `ls` shows the linked projects.

## Usage

Create a registered workspace with a generated name:

```bash
zed-workspace-dock create \
  --mode symlink \
  --folder api=../api \
  --folder ../web
```

The command prints the real `.code-workspace` path it created under the user config directory, for example `~/Library/Application Support/zed-workspace-dock/workspaces/ws-abcd0001020304ff.code-workspace` on macOS.

Create a registered workspace with an explicit name:

```bash
zed-workspace-dock create work \
  --mode symlink \
  --folder api=../api \
  --folder ../web
```

Recreate an existing registered workspace:

```bash
zed-workspace-dock create work --force \
  --mode symlink \
  --folder api=../api \
  --folder ../web
```

Open a registered workspace by name:

```bash
zed-workspace-dock open work
```

For a simple argument such as `work`, registered workspaces take precedence over a same-name file or directory in the current directory. Use an explicit path such as `./work.code-workspace` when you want to open a local file.

List registered workspaces:

```bash
zed-workspace-dock list
```

The output is one registered workspace per line:

```text
work	/Users/alice/Library/Application Support/zed-workspace-dock/workspaces/work.code-workspace
```

Create a workspace file at an explicit path:

```bash
zed-workspace-dock create --output work.code-workspace \
  --mode symlink \
  --folder api=../api \
  --folder ../web
```

Open a workspace by path:

```bash
zed-workspace-dock open work.code-workspace
```

Force folder mode for one run:

```bash
zed-workspace-dock open work.code-workspace --mode folders
```

Install global Zed tasks:

```bash
zed-workspace-dock install
```

By default, this writes the global Zed tasks file at `~/.config/zed/tasks.json`.
Zed tasks are stored as a JSON array. The install command reads the managed task templates from `resources/zed-tasks.json`, injects the absolute command path, and merges by task label without duplicating managed tasks.

Install global Zed tasks with an explicit binary path:

```bash
zed-workspace-dock install --command /usr/local/bin/zed-workspace-dock
```

Install into an explicit tasks file:

```bash
zed-workspace-dock install --tasks-path ~/.config/zed/tasks.json
```

Installed tasks use `$ZED_FILE`. Run them while a `.code-workspace` file is open or selected in Zed.

## Workspace Contract

```json
{
  "folders": [
    { "name": "api", "path": "../api" },
    { "path": "../web" }
  ],
  "zed-dock": {
    "mode": "symlink"
  }
}
```

The MVP accepts strict JSON `.code-workspace` files only. If `zed-dock` exists, `mode` is required. Supported modes are `folders` and `symlink`.

Registered workspaces are stored under the user config directory at `zed-workspace-dock/workspaces/`. When `create` writes a registered workspace, folder paths are resolved from the current working directory and stored as absolute canonical paths so the workspace can be opened from any directory. When `create --output` writes an explicit file, folder paths are preserved exactly as passed.

The runtime parser defaults a missing `folders` field to an empty list. The `create` command always writes `folders`, but hand-written workspace files may omit it.

## Schemas

JSON Schemas are published under `resources/schemas/` using JSON Schema Draft 2020-12:

- `resources/schemas/code-workspace.schema.json` describes the `.code-workspace` shape used by this tool.
- `resources/schemas/zed-dock-marker.schema.json` describes the internal `.zed-dock.json` marker stored inside managed docks.

The schemas are currently documentation/editor/test resources. Runtime parsing still uses the Rust data model and explicit validation errors. The workspace schema mirrors runtime parsing: `folders` is optional at the root, and `zed-dock.mode` is required only when `zed-dock` exists.

## Safety

- Dock directories live under the platform cache directory at `zed-workspace-dock/docks/`.
- Each managed dock contains `.zed-dock.json`.
- Rebuilds modify only marker-owned docks.
- Existing dock directories without a valid marker abort.
- Unmanaged files inside a marker-owned dock abort.
- Project folders are symlinked, not copied.
- `folders[].name` must be one filesystem entry name, not a path.
- `folders[].name` cannot use reserved dock metadata names such as `.zed-dock.json`.
- Registered workspace names must be one filesystem entry name without `.code-workspace`; `open` accepts the name with or without that extension.
- Registered workspace creation does not overwrite an existing workspace unless `--force` is passed.

## References

This project is a from-scratch Rust study project informed by:

- `fu5ha/zed-workspaces`
- `artumont/zed-workspaces`

No source code from those projects is copied here. If that changes later, keep the required license notices with the copied code.
