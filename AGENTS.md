# Zed Workspace Dock Agent Guide

Zed Workspace Dock is a Rust CLI for creating and opening Zed `.code-workspace` files through either direct folder mode or a marker-protected symlink dock.

Use this file as the local operating guide for AI agents working in this repository. Keep changes aligned with the current CLI contract in `README.md`, the project language in `CONTEXT.md`, and the release decisions in `docs/adr/`.

> [!IMPORTANT]
> The supported way to add folders to an existing registered workspace today is to recreate it with `create ... --name <name> --force` and pass the full folder list again. Do not document or implement an incremental `add` workflow unless the CLI actually gains that command.

## Current Product Shape

- Canonical product name: Zed Workspace Dock.
- Canonical short name, repository slug, Cargo package, and installed binary: `zwd`.
- Public command and only installed binary: `zwd`, backed by `src/bin/zwd.rs`.
- Commands: `create`, `open`, `install`, and `list`.
- Default create mode: `symlink`.
- Alternate open/create mode: `folders`.
- Workspace format: strict JSON `.code-workspace` files with optional `folders` and optional `zed-dock`.
- Managed state:
  - Registered workspaces live under the user config directory at `zwd/workspaces/`.
  - Dock roots live under the platform cache directory at `zwd/docks/`.
  - Zed task templates come from `resources/zed-tasks.json`.
  - JSON Schemas live under `resources/schemas/` and are documentation/editor/test resources, not runtime validators.

Windows support is partial in the MVP: folder mode is supported, while symlink dock mode is currently macOS/Linux-oriented.

## Repository Map

- `src/bin/zwd.rs`: thin public binary entrypoint.
- `src/lib.rs`: command dispatch and public crate entrypoint.
- `src/cli.rs`: Clap command, flag, and mode definitions.
- `src/workspace.rs`: workspace creation, parsing, validation, listing, and reference resolution.
- `src/dock.rs`: symlink dock construction, marker handling, and safety checks.
- `src/install.rs`: global Zed task installation and merge behavior.
- `src/zed.rs`: Zed process invocation.
- `src/error.rs`: typed errors used across the CLI.
- `tests/cli.rs`: integration coverage for user-visible CLI behavior.
- `resources/`: task templates and JSON Schemas.
- `docs/adr/`: accepted project decisions.
- `release-plz.toml`, `dist-workspace.toml`, `.github/workflows/`: release automation.

## Development Commands

Use the pinned toolchain from `rust-toolchain.toml`. `devbox shell` or `devbox run -- <command>` is available when the host environment needs the repo toolchain.

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Useful local commands:

```bash
cargo build
cargo run -- create ../api ../web --name work
cargo run -- open work --zed-bin /bin/echo
cargo run -- list
```

`Makefile.toml` exposes equivalent `cargo make` tasks, but direct Cargo commands are the preferred validation path in this repo. Prior runs showed `cargo make` can add avoidable friction through extra metadata/download behavior.

## Validation Expectations

For Rust code changes, run at least:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

For CLI behavior changes, add or update integration tests in `tests/cli.rs`. Prefer temporary directories and harmless executables such as `/bin/echo` when asserting Zed invocation arguments.

For release or installer changes, also inspect:

```bash
cargo metadata --format-version 1
```

If a change affects the Zed UX directly, keep the manual Zed smoke-test gap visible until it has actually been exercised in Zed: terminal root, `ls`, search, Git status, LSP behavior, tasks, editing, delete behavior, and file watching.

## Coding Guidelines

- Follow Rust 2024, `rustfmt`, and the lint policy in `Cargo.toml`.
- Keep `src/bin/zwd.rs` thin. Put behavior in library modules.
- Preserve the current modular boundaries instead of moving logic into one file.
- Model user-visible failures through `src/error.rs`; avoid ad hoc string errors in new code.
- Treat warnings as blockers. The crate denies several Rust and Clippy lint groups and forbids unsafe code.
- Keep CLI-facing names kebab-case and Rust identifiers snake_case.
- Use structured JSON parsing via `serde`/`serde_json`; do not hand-roll workspace parsing.

## Safety Rules

- Never modify a dock directory unless it has a valid `.zed-dock.json` marker owned by this tool.
- Abort on existing dock directories without a valid marker.
- Abort when unmanaged files are present inside a marker-owned dock.
- Symlink project folders; do not copy source folders into dock roots.
- Do not delete or mutate symlink targets.
- Keep workspace names and dock link names to single filesystem entry names.
- Preserve the registered-workspace overwrite rule: creation fails unless `--force` is passed.
- Keep registered workspace resolution precedence: a simple name prefers a registered workspace over a same-name local path.

## Documentation Rules

- Update `README.md` when public install, usage, workspace, safety, or release behavior changes.
- Update `CONTEXT.md` when terminology or domain language changes.
- Add an ADR under `docs/adr/` for decisions that alter release flow, CLI contract, persistence layout, or safety rules.
- Do not add license, changelog, or contributing sections to general docs; those belong in dedicated files.
- Keep examples current with the simplified create command:

```bash
zwd create ../api ../web --name work
zwd create ../api ../web ../docs --name work --force
```

Avoid the removed `--folder name=path` create syntax.

## Release Notes

This project uses Release PRs:

1. Feature and fix commits merge into `main`.
2. `release-plz` opens or updates a Release PR.
3. Merging the Release PR creates the version tag.
4. `cargo-dist` builds release archives, installers, checksums, attestations, and the GitHub Release.

The package is currently `publish = false`; release artifacts are GitHub Release assets, not crates.io publishing.

## Commit And PR Expectations

Use conventional commit messages with lowercase imperative subjects, for example:

```text
feat: add registered workspace listing
fix: reject unmanaged dock contents
docs: document release automation
```

PRs should state the behavior change, validation commands run, and any release, schema, installer, persistence, or platform impact.
