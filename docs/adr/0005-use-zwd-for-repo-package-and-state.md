# Use zwd for repository, package, and state paths

Zed Workspace Dock remains the full product name. The repository slug, Cargo package, release assets, schema namespace, and managed state directories use the shorter `zwd` name.

The short name is easier to remember and matches the installed command users type. Keeping `zed-workspace-dock` for those surfaces would make release URLs and installer names longer than the command.

Existing preview releases and tags predate public use and can be removed before the next release. The CLI does not migrate old `zed-workspace-dock` config or cache directories; the first supported distribution starts from the `zwd` paths.
