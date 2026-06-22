# Zed Workspaces Dock

Zed Workspaces Dock is a developer tool for opening multi-project Zed sessions from workspace files, with a managed symlink root when terminal-visible project grouping matters.

## Language

**Zed Workspaces Dock**:
The product and CLI that reads a workspace file and opens projects in Zed through either direct folders or a managed symlink dock.
_Avoid_: Zed Workspace Dock, zed-workspace-dock

**Workspace file**:
A `.code-workspace` file describing one or more project folders for a Zed session. In the MVP, each workspace file has zero or one `zed-dock` configuration object.
_Avoid_: Project file, session file

**Symlink dock**:
A marker-protected directory whose entries are symbolic links to the projects from a workspace file. One symlink dock belongs to one workspace file path.
_Avoid_: Temporary copy, workspace copy, virtual workspace

**Folder mode**:
Workspace opening mode where Zed receives the resolved project folder paths directly.
_Avoid_: Standard mode, normal mode

**Dock mode**:
Workspace opening mode where Zed receives the symlink dock root instead of the individual project folder paths.
_Avoid_: Temporary mode, symlink mode

**Flagged ambiguities**:
`zed-workspace-dock` appears in older planning text, but `zed-workspaces-dock` is the canonical project and binary name.

## Example Dialogue

Dev: Should this workspace open as folders or through a dock?

Domain expert: Use dock mode when the Zed terminal should start in one root where `ls` shows every linked project. Use folder mode when direct project paths are enough.
