# Archive Report — configure-dev-tooling

- **Change**: configure-dev-tooling
- **Mode**: openspec
- **Archived at**: 2026-08-05
- **Archived to**: `openspec/changes/archive/2026-08-05-configure-dev-tooling/`
- **Verification verdict**: PASS (no CRITICAL, no WARNING) — see `verify-report.md`
- **Strict TDD**: active; satisfied vacuously (config-only change, zero production code)

## Specs Synced (delta -> source of truth)

| Domain | Action | Details |
|--------|--------|---------|
| `agent-instruction-sync` | Created | New domain — all Requirements ADDED (R1 Canonical source & migration safety, R2 Explicit target & MCP policy, R3 Managed destinations & ignore boundaries). No MODIFIED/REMOVED. |
| `ci-quality-validation` | Created | New domain — all Requirements ADDED (R1 Trigger/permissions/concurrency/pinning, R2 Rust quality & platform matrix, R3 Blocking AgentSync drift status, R4 Documentation & rollback). No MODIFIED/REMOVED. |
| `local-development-hooks` | Created | New domain — all Requirements ADDED (R1 External tool setup & docs, R2 Local Rust quality gates, R3 AgentSync lifecycle refresh, R4 Cross-platform setup, R5 Hook rollback). No MODIFIED/REMOVED. |

Merge kind: **additive_only** — every requirement in each delta was ADDED; no
main spec was rewritten or had content removed. Copies verified byte-identical
to delta specs (`diff -q`). No destructive merge performed, honoring
`openspec/config.yaml::rules.archive` ("Warn before merging destructive
deltas").

New canonical specs:
- `openspec/specs/agent-instruction-sync/spec.md`
- `openspec/specs/ci-quality-validation/spec.md`
- `openspec/specs/local-development-hooks/spec.md`

## Archive Bookkeeping Fixes Applied (verify-report SUGGESTIONS)

| # | Finding | Fix applied |
|---|---------|-------------|
| S1 | design.md line 75: "Explicit four-target mapping" (stale) | Corrected to **three-target** mapping (root/Claude/Copilot), matching the approved ADR + tasks P2 + `agentsync.toml` (exactly 3 `[agents.*]` sections; no `opencode` target; no MCP). |
| S2 | tasks.md Phase 5 items (5.1–5.6) remained `[ ]` | Ticketed `[x]` with per-task evidence from `verify-report.md` (clean-checkout apply/status, lefthook registration + non-blocking lifecycle, YAML/TOML parse, cargo gates exit 0, no src/platform/Cargo changes, rollback rehearsal). Also corrected stale "four symlink targets" in 5.1 to the verified three. |
| S3 | README Lefthook Linux package names deferred to official docs | Informational — NOT applied (out of archive scope; touches README.md, which is a committed tooling file and explicitly out of bounds for this phase). |
| S4 | `.gitignore` managed block contains AgentSync auxiliary paths | Informational — no action required (expected AgentSync 1.45.2 behavior; `status --json` clean; `target/`/`.DS_Store` outside block). |

## Archive Contents

- `exploration.md` ✅
- `proposal.md` ✅
- `specs/agent-instruction-sync/spec.md` ✅
- `specs/ci-quality-validation/spec.md` ✅
- `specs/local-development-hooks/spec.md` ✅
- `design.md` ✅
- `tasks.md` ✅ (all tasks complete, incl. Phase 5)
- `verify-report.md` ✅
- `state.yaml` ✅ (archived/closed)
- `archive-report.md` ✅ (this file)

## Traceability

- **Verified at**: HEAD `715a4cc` (branch `feat/configurable-categories-and-sources`)
- **Tooling commits reviewed**: `e31eb6d` (AgentSync sync), `9ea7e5c` (Lefthook + toolchain), `2bc2d15` (CI), `ad0a15d` (README per-OS), `715a4cc` (.gitignore reconcile) — config-only, no `src/`/`platform/`/`Cargo.toml`/`Cargo.lock` changes
- **Local gate**: `cargo fmt -- --check` exit 0; `cargo clippy --all-targets --all-features -- -D warnings` exit 0; `cargo test` 35 passed / 0 failed
- **Risks carried forward** (from verify-report, informational): GitHub-hosted matrix not yet exercised (no remote); lifecycle hooks require network at hook time (non-blocking by design); no new failures introduced.

## SDD Cycle Complete

plan -> spec -> design -> tasks -> apply -> verify (PASS) -> archive. The change
has been fully planned, implemented, verified, and archived. Ready for the next change.
