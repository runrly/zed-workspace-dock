# Changelog

All notable changes to Zed Workspace Dock are tracked here.

Release PRs update this file. Manual edits should only capture release history that predates automation.

## [0.2.0](https://github.com/runrly/zed-workspace-dock/compare/v0.1.0...v0.2.0) - 2026-06-26

### Added

- add zwd binary alias
- [**breaking**] simplify workspace creation command

### Other

- improve readme
- update agent guide
- add repository agent guidelines
- rename release app secrets
- add release automation

## [0.1.0] - 2026-06-23

### Added

- Initial Rust CLI with `create`, `open`, `install`, and `list`.
- Workspace files with `folders` and optional `zed-dock.mode`.
- Registered workspaces stored under the user config directory.
- Managed symlink docks with marker-based rebuild safety.
- Global Zed task installation from bundled task templates.
- JSON Schema resources for workspace files and dock markers.

### Validation

- Manual macOS binary smoke test passed on 2026-06-24.
- `cargo fmt`, `cargo check`, `cargo test`, and `cargo clippy` passed before the baseline tag.

[0.1.0]: https://github.com/runrly/zed-workspace-dock/releases/tag/v0.1.0
