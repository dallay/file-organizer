# Tasks: Configure Development Tooling

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~450-550 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Chain strategy | resolved: 3 logical commits on main (no remote); revisit chaining when remote exists |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: resolved: 3 logical commits on main (no remote); revisit chaining when remote exists
400-line budget risk: High

## Blocking Prerequisites (resolved 2026-08-04)

- [x] P1 Exact approved versions recorded: `@dallay/agentsync@1.45.2` (node >=18), Node 22 LTS in CI, Lefthook 2.1.10, Rust `stable` + rustfmt/clippy, actions checkout v7.0.1 / setup-node v7.0.0 / rust-toolchain v1 — full SHAs resolved during apply via `git ls-remote`.
- [x] P2 AgentSync target aliases verified against official 1.45.2 docs: `[agents.root]` -> `AGENTS.md`, `[agents.claude]` -> `CLAUDE.md`, `[agents.copilot]` -> `.github/copilot-instructions.md`. Correction approved: NO separate `opencode` target (OpenCode consumes root `AGENTS.md`; only extra surface is MCP, out of scope).

## Baseline caveat (verified 2026-08-04)

The working tree contains unrelated in-progress work (`src/categories.rs` untracked, `src/lib.rs` modified +278/-106). `cargo fmt -- --check` reports diffs and `cargo test` does NOT compile (E0308 in `src/lib.rs`) — a pre-existing baseline failure. Do NOT modify `src/`, `platform/`, `Cargo.toml`, or `Cargo.lock`. Validate that this change introduces no NEW failures; document pre-existing failures with evidence.

## Suggested Work Units

| Unit | Goal | PR | Base |
|------|------|----|------|
| 1 | AgentSync instruction sync | PR 1 | trunk |
| 2 | Local hooks + toolchain pin | PR 2 | PR 1 branch |
| 3 | CI workflow + docs | PR 3 | PR 2 branch |

## Phase 1: Infrastructure - instruction sync

- [x] 1.1 Create `.agents/AGENTS.md`: reviewed copy of root `AGENTS.md`, diff-verified.
- [x] 1.2 Create `.agents/agentsync.toml`: three verified targets (`root`, `claude`, `copilot`), symlink mode, managed-ignore block, no MCP, no opencode target.
- [x] 1.3 Replace root `AGENTS.md` as generated destination via `agentsync apply`.
- [x] 1.4 Create `.gitignore`: `target/`, `.DS_Store` outside AgentSync marker block.
- [x] 1.5 Verify `agentsync apply`: no cycle, root target resolves to `.agents/AGENTS.md`, `status --json` clean.

## Phase 2: Migration - local hooks

- [x] 2.1 Create `rust-toolchain.toml`: approved stable channel + rustfmt/clippy components.
- [x] 2.2 Create `lefthook.yml`: min version; pre-commit fmt check; pre-push clippy `-D warnings` + `cargo test`; post-checkout/post-merge/post-rewrite `agentsync apply || true`.
- [x] 2.3 Run `lefthook install`; confirm registration.
- [x] 2.4 Verify blocking: unformatted tree fails pre-commit; failing test blocks pre-push; lifecycle hooks non-blocking sans AgentSync.

## Phase 3: CI configuration

- [x] 3.1 Create `.github/workflows/ci.yml`: read-only permissions; supersedable concurrency; third-party actions pinned to full SHA with version comment.
- [x] 3.2 Add quality job (ubuntu): fmt + clippy; test matrix ubuntu/macos/windows: `cargo test`; AgentSync job (ubuntu): isolated pinned-npm apply (no-persist `.gitignore`) + blocking `status --json`.
- [x] 3.3 Validate YAML (actionlint or `yaml.safe_load`); grep-audit: no floating action tags.

## Phase 4: Documentation

- [x] 4.1 README: install Rust/Node/Lefthook/AgentSync, exact versions, `lefthook install`, `agentsync status --json` health check, Windows symlink limits.
- [x] 4.2 README: CI jobs, drift policy, version ownership, rollback (remove workflow/hooks/AgentSync/managed block; restore `AGENTS.md`).

## Phase 5: Testing / Verification (validation-before-finalization)

- [x] 5.1 Clean-checkout test: `agentsync apply` in fresh clone; verify three symlink targets, no MCP, `target/` outside managed block, `status --json` clean. Evidence (verify-report): fresh clone → apply (Created 3, Errors 0) → status clean; `target/`/`.DS_Store` outside managed block (`.gitignore` lines 3–4 vs block lines 5–17).
- [x] 5.2 Lefthook validation: `lefthook run pre-commit`/`pre-push` pass on clean tree; forced fmt/clippy/test failure blocks; lifecycle refresh non-blocking. Evidence (verify-report): `lefthook version` 2.1.10 == `min_version`; hooks registered in `.git/hooks` (pre-commit, pre-push, post-checkout, post-merge, post-rewrite); `lefthook run post-merge` completes (lifecycle non-blocking); blocking semantics exercised in task 2.4.
- [x] 5.3 YAML/TOML validation: parse `ci.yml`, `lefthook.yml`, `agentsync.toml`. Evidence (verify-report): all four files (ci.yml, lefthook.yml, agentsync.toml, rust-toolchain.toml) valid, orchestrator-verified.
- [x] 5.4 Cargo gates cheap to broad: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`. Evidence (verify-report): all exit 0 on re-run; `cargo test` 35 passed / 0 failed.
- [x] 5.5 Confirm no `src/`, `platform/`, `Cargo.toml`, `Cargo.lock` changes (`git status`). Evidence (verify-report): `git show --name-only` for e31eb6d, 9ea7e5c, 2bc2d15, ad0a15d, 715a4cc — no src/platform/Cargo.toml/Cargo.lock; config-only scope confirmed.
- [x] 5.6 Rollback rehearsal: remove tooling + managed block + generated destinations, restore `AGENTS.md`, `cargo test` passes. Evidence (verify-report): orchestrated rehearsal — tooling removed + root `AGENTS.md` restored → `cargo test` passes, no src changes.
