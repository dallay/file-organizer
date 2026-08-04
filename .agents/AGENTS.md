# AGENTS.md

<!-- agentsync:agent-config-layout:start -->
## Agent config layout

`.agents/` is the canonical source for shared instructions in this project.

- Instructions: `.agents/AGENTS.md` is the canonical instructions file, and these `symlink` targets reflect it directly in `CLAUDE.md`, `.github/copilot-instructions.md`, and `AGENTS.md` (the repository root, consumed by OpenCode).

No skills, commands, MCP, or per-agent config directories are managed by AgentSync in this repository.

<!-- agentsync:agent-config-layout:end -->

## Repository map

- This is one Rust 2021 Cargo package, not a workspace.
- Keep Clap parsing and CLI-only overrides in `src/main.rs`; keep config validation, classification, traversal, locking, logging, and file operations reusable in `src/lib.rs`.
- `platform/` contains install-time launchers: launchd for macOS and a systemd user service/timer for Linux. Windows Task Scheduler setup exists only in `README.md`.
- Durable change rules live in `openspec/config.yaml`; its apply phase requires strict TDD.

## Development and verification

- Rust stable is the only documented prerequisite.
- Run one exact unit test with `cargo test tests::<test_name> -- --exact` (example: `cargo test tests::classifies_extensions_case_insensitively -- --exact`). Tests currently live inline in `src/lib.rs` and use `tempfile`.
- Before finishing a Rust change, run the local gate from cheap to broad: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo test`.
- Add or change a failing regression/unit test before production code. There is no repository CI, coverage setup, or custom rustfmt/Clippy configuration to replace the local gate.
- Use `cargo build --release` only when validating packaging or platform launchers; the artifact is `target/release/file-organizer` (`.exe` on Windows).

## Safe CLI checks

- Validate an explicit config from source with `cargo run -- --config ./config.toml validate-config`.
- For manual organization checks, use a temporary source directory and `cargo run -- --config ./config.toml run --dry-run --log /dev/null /tmp/input` (`NUL` on Windows). Positional directories replace `source_directories`.
- `--dry-run` skips moves and locking but still opens/writes the configured log unless `--log /dev/null` or `--log NUL` is supplied. Never test against a real Downloads/Desktop directory unless explicitly requested.

## Behavior that is easy to break

- Config lookup is `FILE_ORGANIZER_CONFIG` first, then `%APPDATA%/file-organizer/config.toml` on Windows or `$XDG_CONFIG_HOME/file-organizer/config.toml` (falling back to `~/.config`) elsewhere.
- Extension overrides are case-insensitive and replace built-in mappings. Generated category directories are excluded from recursive scans so organized files are not reprocessed.
- Non-dry runs acquire a directory lock (`~/.cache/file-organizer.lock` on Unix-like systems; `LOCALAPPDATA/file-organizer.lock` on Windows). A concurrent run must fail rather than proceed.
- Moves intentionally use `rename`: they are atomic on one volume, and cross-volume moves return an error instead of falling back to copy/delete.
- Scheduler changes must preserve the installed-binary/default-config assumptions in `platform/` and stay consistent with the platform instructions in `README.md`.
