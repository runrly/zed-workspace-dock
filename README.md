# Zed Workspace Dock

Rust CLI for opening Zed workspace files through direct folder mode or a managed symlink dock.

Dock mode creates one marker-protected cache directory with symlinks to the workspace folders, then opens that dock in Zed. This gives the Zed terminal one real root where `ls` shows the linked projects.

## Install

Install the latest release on macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/runrly/zed-workspace-dock/releases/latest/download/zed-workspace-dock-installer.sh | sh
```

Install the latest release on Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/runrly/zed-workspace-dock/releases/latest/download/zed-workspace-dock-installer.ps1 | iex"
```

Installers place the binaries under Cargo's bin directory by default. Make sure that directory is on your `PATH`.

You can also download platform archives from GitHub Releases. Release artifacts include SHA256 checksums.

Windows support is partial in the MVP: `folders` mode is supported, while `symlink` mode is currently supported on macOS and Linux only.

## Usage

The canonical command is `zed-workspace-dock`. The short alias `zwd` runs the same CLI, so every example below can use either name.

Create a registered workspace with a generated name:

```bash
zed-workspace-dock create ../api ../web
```

The command prints the real `.code-workspace` path it created under the user config directory, for example `~/Library/Application Support/zed-workspace-dock/workspaces/ws-abcd0001020304ff.code-workspace` on macOS.

Create a registered workspace with an explicit name:

```bash
zed-workspace-dock create ../api ../web --name work
```

Recreate an existing registered workspace:

```bash
zed-workspace-dock create ../api ../web --name work --force
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

Create a workspace file in an output directory:

```bash
zed-workspace-dock create ../api ../web --name work --output ../workspaces
```

Create a workspace in folder mode:

```bash
zed-workspace-dock create ../api ../web --mode folders
```

Open a workspace by path:

```bash
zed-workspace-dock open ../workspaces/work.code-workspace
```

Force folder mode for one run:

```bash
zed-workspace-dock open ../workspaces/work.code-workspace --mode folders
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

You can also point installed tasks at the short alias:

```bash
zwd install --command /usr/local/bin/zwd
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

Registered workspaces are stored under the user config directory at `zed-workspace-dock/workspaces/`. Workspaces created with `--output <dir>` are standalone files in that directory and are opened by path.

The `create` command accepts one or more folder paths as positional arguments. It writes `symlink` mode unless `--mode folders` is passed. It stores canonical absolute folder paths for both registered workspaces and output-directory workspaces. The runtime parser defaults a missing `folders` field to an empty list. The `create` command always writes `folders`, but hand-written workspace files may omit it.

## Schemas

JSON Schemas are published under `resources/schemas/` using JSON Schema Draft 2020-12:

- `resources/schemas/code-workspace.schema.json` describes the `.code-workspace` shape used by this tool.
- `resources/schemas/zed-dock-marker.schema.json` describes the internal `.zed-dock.json` marker stored inside managed docks.

The schemas are currently documentation/editor/test resources. Runtime parsing still uses the Rust data model and explicit validation errors. The workspace schema mirrors runtime parsing: `folders` is optional at the root, and `zed-dock.mode` is required only when `zed-dock` exists.

## Releases

This project uses Release PRs:

1. Merge feature and fix PRs into `main`.
2. `release-plz` opens or updates a Release PR with the next version and changelog.
3. Merge the Release PR.
4. `release-plz` creates the `vX.Y.Z` tag.
5. `dist` builds release artifacts, installers, checksums, attestations, and the GitHub Release.

Versioning follows CLI SemVer. `fix` commits bump patch versions. `feat` commits bump minor versions. Breaking changes bump minor versions while the project is below `1.0.0`, then major versions after `1.0.0`.

The first automated release after the manually tested `v0.1.0` baseline is expected to be `v0.1.1`.

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
