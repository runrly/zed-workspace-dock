# Use zwd as the public command

Zed Workspace Dock remains the full product name, but `zwd` is the only installed CLI binary and user-facing command. This keeps repeated terminal usage short.

The longer `zed-workspace-dock` binary target is removed before the first public release so there is no compatibility burden for a command name we do not want to support.

ADR 0005 supersedes the earlier decision to keep the repository, Cargo package, and managed state directories named `zed-workspace-dock`.
