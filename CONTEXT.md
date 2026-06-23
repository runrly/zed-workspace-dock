# Zed Workspace Dock

Zed Workspace Dock is a developer tool for opening multi-project Zed sessions from workspace files, with a managed symlink root when terminal-visible project grouping matters.

## Language

**Zed Workspace Dock**:
The product and CLI that reads a workspace file and opens projects in Zed through either direct folders or a managed symlink dock.
_Avoid_: the pluralized product and binary names

**Workspace file**:
A `.code-workspace` file describing one or more project folders for a Zed session. In the MVP, each workspace file has zero or one `zed-dock` configuration object.
_Avoid_: Project file, session file

**Registered workspace**:
A workspace file stored in the user's config directory under `zed-workspace-dock/workspaces/`. Registered workspaces can be opened by name and are listed by `zed-workspace-dock list`.
_Avoid_: Saved project, cached workspace

**Workspace name**:
The single filesystem entry name used to address a registered workspace without typing its full `.code-workspace` path. It must not include `.code-workspace`, but `open` accepts the name with or without that extension.
_Avoid_: Workspace path, dock name

**Explicit workspace output**:
A workspace file written to a user-provided path via `create --output`. This path-oriented mode preserves folder paths exactly as entered instead of canonicalising them for the registry.
_Avoid_: Local registry, exported registry entry

**Symlink dock**:
A marker-protected directory whose entries are symbolic links to the projects from a workspace file. One symlink dock belongs to one workspace file path.
_Avoid_: Temporary copy, workspace copy, virtual workspace

**Dock link name**:
The single filesystem entry name used for one project link inside a symlink dock. A dock link name is a name, not a path.
_Avoid_: Link path, relative link target

**Folder mode**:
Workspace opening mode where Zed receives the resolved project folder paths directly.
_Avoid_: Standard mode, normal mode

**Dock mode**:
Workspace opening mode where Zed receives the symlink dock root instead of the individual project folder paths.
_Avoid_: Temporary mode, symlink mode

**Task template resource**:
A versioned product resource that defines the Zed task templates Zed Workspace Dock can install.
_Avoid_: Generated task file, task config cache

**Schema resource**:
A JSON Schema Draft 2020-12 document under `resources/schemas/` that documents editor/test validation for either workspace files or dock markers. Schemas are not runtime validators.
_Avoid_: Runtime contract, migration validator

**Flagged ambiguities**:
Older planning text may use pluralized product or binary names, but `zed-workspace-dock` is the canonical project and binary name.

## Behavior Notes

- `create --mode ... --folder ...` creates a registered workspace with a generated `ws-<16-hex>` name and prints the real file path.
- `create <name> --mode ... --folder ...` creates a registered workspace with the given name and fails if it already exists unless `--force` is passed.
- `create --output <path.code-workspace> --mode ... --folder ...` writes an explicit workspace file and prints its canonical path.
- Registered workspaces store folder paths as canonical absolute paths resolved from the current working directory at creation time.
- Explicit workspace outputs preserve folder path strings exactly as entered.
- `open <simple-name>` prefers a registered workspace over a same-name file or directory in the current working directory. Use an explicit path such as `./work.code-workspace` when the local file is intended.
- `list` prints registered workspaces as `name<TAB>path`.
- Workspace schemas document the current parser behavior: `folders` defaults to an empty list when absent, and `zed-dock.mode` is required only when `zed-dock` exists.

## Example Dialogue

Dev: Should this workspace open as folders or through a dock?

Domain expert: Use dock mode when the Zed terminal should start in one root where `ls` shows every linked project. Use folder mode when direct project paths are enough.
