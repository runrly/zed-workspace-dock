# Zed Workspace Dock

<p align="center">
  <a href="https://github.com/runrly/zed-workspace-dock/actions/workflows/ci.yml"><img src="https://shieldcn.dev/github/ci/runrly/zed-workspace-dock.svg?workflow=CI&branch=main&variant=secondary" alt="CI" /></a>
  <a href="https://github.com/runrly/zed-workspace-dock/releases"><img src="https://shieldcn.dev/github/release/runrly/zed-workspace-dock.svg?variant=secondary" alt="Release" /></a>
  <a href="https://github.com/runrly/zed-workspace-dock"><img src="https://shieldcn.dev/github/license/runrly/zed-workspace-dock.svg?variant=secondary" alt="License" /></a>
  <a href="rust-toolchain.toml"><img src="https://shieldcn.dev/badge/Rust-1.96%2B-b7410e.svg?logo=rust&variant=secondary" alt="Rust 1.96+" /></a>
</p>

Rust CLI for opening multi-project Zed sessions from `.code-workspace` files.

Zed can open multiple folders directly, but its terminal still benefits from a single visible root in some workflows. Zed Workspace Dock can create a marker-protected cache directory where each project folder is linked, then open that dock root in Zed. Running `ls` in the terminal shows the linked projects without copying source code.

[Install](#install) - [Quick start](#quick-start) - [Usage](#usage) - [Workspace files](#workspace-files) - [Development](#development)

> [!NOTE]
> Windows support is partial in the MVP. `folders` mode is supported on Windows; `symlink` dock mode is currently intended for macOS and Linux.

## Features

- Create registered Zed workspace files from one or more project folders.
- Open a workspace by registered name or by `.code-workspace` path.
- Choose between direct `folders` mode and managed `symlink` dock mode.
- Install global Zed tasks backed by the packaged task templates.
- Rebuild dock roots safely using `.zed-dock.json` ownership markers.
- Use the canonical `zed-workspace-dock` command or the shorter `zwd` alias.

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

Installers place binaries under Cargo's bin directory by default. Make sure that directory is on your `PATH`.

You can also download platform archives from [GitHub Releases](https://github.com/runrly/zed-workspace-dock/releases). Release artifacts include SHA256 checksums.

## Quick Start

Create a registered workspace from two project folders:

```bash
zed-workspace-dock create ../api ../web --name work
```

Open it in Zed:

```bash
zed-workspace-dock open work
```

List registered workspaces:

```bash
zed-workspace-dock list
```

Add another folder later by recreating the workspace with the complete folder list:

```bash
zed-workspace-dock create ../api ../web ../docs --name work --force
```

> [!IMPORTANT]
> There is no incremental `add` command yet. Recreate the workspace with `--force` when the folder list changes.

## Usage

The canonical command is `zed-workspace-dock`. The short alias `zwd` runs the same CLI, so every example can use either name.

Create a registered workspace with a generated name:

```bash
zed-workspace-dock create ../api ../web
```

The command prints the real `.code-workspace` path created under the user config directory, for example:

```text
~/Library/Application Support/zed-workspace-dock/workspaces/ws-abcd0001020304ff.code-workspace
```

Create a workspace file in a specific output directory:

```bash
zed-workspace-dock create ../api ../web --name work --output ../workspaces
```

Create a workspace that opens folders directly instead of a dock root:

```bash
zed-workspace-dock create ../api ../web --name work --mode folders
```

Open a workspace file by path:

```bash
zed-workspace-dock open ../workspaces/work.code-workspace
```

Force folder mode for one run:

```bash
zed-workspace-dock open ../workspaces/work.code-workspace --mode folders
```

Reuse an existing Zed window:

```bash
zed-workspace-dock open work --reuse
```

Use a custom Zed executable, useful for tests or alternate installs:

```bash
zed-workspace-dock open work --zed-bin /Applications/Zed.app/Contents/MacOS/cli
```

For a simple argument such as `work`, registered workspaces take precedence over a same-name file or directory in the current directory. Use an explicit path such as `./work.code-workspace` when you want to open a local file.

## Zed Tasks

Install global Zed tasks:

```bash
zed-workspace-dock install
```

By default, this writes the global Zed tasks file at `~/.config/zed/tasks.json`. The installer reads templates from `resources/zed-tasks.json`, injects the absolute command path, and merges by task label without duplicating managed tasks.

Install tasks with an explicit binary path:

```bash
zed-workspace-dock install --command /usr/local/bin/zed-workspace-dock
```

Install tasks pointing at the short alias:

```bash
zwd install --command /usr/local/bin/zwd
```

Install into an explicit tasks file:

```bash
zed-workspace-dock install --tasks-path ~/.config/zed/tasks.json
```

Installed tasks use `$ZED_FILE`. Run them while a `.code-workspace` file is open or selected in Zed.

## Workspace Files

Zed Workspace Dock accepts strict JSON `.code-workspace` files:

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

Supported modes are:

- `symlink`: build and open a managed dock root.
- `folders`: pass resolved project folder paths directly to Zed.

If `zed-dock` exists, `mode` is required. If `folders` is missing, runtime parsing treats it as an empty list.

Registered workspaces are stored under the user config directory at `zed-workspace-dock/workspaces/`. Workspaces created with `--output <dir>` are standalone files in that directory and are opened by path.

The `create` command accepts one or more folder paths as positional arguments. It writes `symlink` mode unless `--mode folders` is passed. Created workspaces store canonical absolute folder paths resolved from the current working directory.

## Schemas

JSON Schemas are published under `resources/schemas/` using JSON Schema Draft 2020-12:

- `resources/schemas/code-workspace.schema.json` describes the `.code-workspace` shape used by this tool.
- `resources/schemas/zed-dock-marker.schema.json` describes the internal `.zed-dock.json` marker stored inside managed docks.

The schemas are documentation, editor, and test resources. Runtime parsing still uses the Rust data model and explicit validation errors.

## Safety

- Dock directories live under the platform cache directory at `zed-workspace-dock/docks/`.
- Each managed dock contains `.zed-dock.json`.
- Rebuilds modify only marker-owned docks.
- Existing dock directories without a valid marker abort.
- Unmanaged files inside a marker-owned dock abort.
- Project folders are symlinked, not copied.
- Symlink targets are not deleted or mutated.
- `folders[].name` must be one filesystem entry name, not a path.
- `folders[].name` cannot use reserved dock metadata names such as `.zed-dock.json`.
- Registered workspace names must be one filesystem entry name without `.code-workspace`; `open` accepts the name with or without that extension.
- Registered workspace creation does not overwrite an existing workspace unless `--force` is passed.

## Development

Use the pinned Rust toolchain from `rust-toolchain.toml`.

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Build locally:

```bash
cargo build
```

Build a production binary:

```bash
cargo build --release --locked
```

The release binary is written to `target/release/zed-workspace-dock`.

## Release Flow

This project uses Release PRs:

1. Merge feature and fix PRs into `main`.
2. `release-plz` opens or updates a Release PR with the next version and changelog.
3. Merge the Release PR.
4. `release-plz` creates the `vX.Y.Z` tag.
5. `cargo-dist` builds release artifacts, installers, checksums, attestations, and the GitHub Release.

Versioning follows CLI SemVer. The package is currently `publish = false`, so distribution is through GitHub Release artifacts rather than crates.io.

## References

This project is a from-scratch Rust study project informed by:

- [`fu5ha/zed-workspaces`](https://github.com/fu5ha/zed-workspaces)
- [`artumont/zed-workspaces`](https://github.com/artumont/zed-workspaces)

No source code from those projects is copied here. If that changes later, keep the required license notices with the copied code.
