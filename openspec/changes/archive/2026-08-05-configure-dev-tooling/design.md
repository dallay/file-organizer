# Design: Configure Development Tooling

## Technical Approach

Add configuration around the single-package Rust project; do not change `src/`, `platform/`, `Cargo.toml`, or dependencies. `.agents/` is the instruction source, Lefthook provides local gates and refresh hooks, and GitHub Actions is authoritative for Rust and AgentSync validation.

## Architecture Decisions

### Canonical instructions and target mapping

**Choice**: Track `.agents/AGENTS.md` as the only source. Configure generated targets for root `AGENTS.md`, Claude (`CLAUDE.md`), and Copilot (`.github/copilot-instructions.md`); define no MCP target. Targets are symlinks, not hand-edited files.

**Approved correction (verified against AgentSync 1.45.2 docs)**: OpenCode consumes the root `AGENTS.md` — the same destination as the `root` target — so NO separate `opencode` target is declared. Its only additional AgentSync surface is `opencode.json` (native MCP), which the spec forbids generating. Declaring an opencode target would either collide with `root` on `AGENTS.md` or create an unwanted MCP config.

**Alternatives considered**: Keep root canonical; commit duplicated files; generate MCP speculatively.

**Rationale**: One source prevents drift while preserving tool discovery paths. Explicit paths make the boundary reviewable; omitting MCP matches the repository’s current setup.

### Safe migration and ownership

**Choice**: Copy and review the complete root `AGENTS.md` in `.agents/AGENTS.md` before reconciling targets. Treat root as a migrated destination, not a second source. AgentSync must reject cycles without deleting the canonical file.

**Alternatives considered**: Apply before migration; delete root without review.

**Rationale**: The untracked instructions contain project conventions. Staging canonical content first avoids self-linking and loss.

### Gitignore ownership

**Choice**: Keep `target/` and `.DS_Store` outside AgentSync markers. AgentSync owns only marker-delimited generated destinations; `.agents/` and its config stay tracked. Do not clean existing build output.

**Rationale**: Boundaries prevent rewriting unrelated ignores and make rollback surgical.

### Lefthook behavior

**Choice**: `pre-commit` runs `cargo fmt -- --check`; `pre-push` runs Clippy with warnings denied then `cargo test`, blocking failures. `post-checkout`, `post-merge`, and `post-rewrite` run `agentsync apply || true`.

**Rationale**: Formatting is fast; the broad gate belongs before pushes. Refreshes must not break Git operations. Missing AgentSync, bad config, and Windows symlink errors are warnings swallowed by `|| true`; README documents `agentsync status --json` as the health check.

### CI jobs and toolchain

**Choice**: Use read-only permissions and cancellation for superseded runs. Add Ubuntu `quality`, Ubuntu `agentsync`, and `test` jobs on Linux (Ubuntu), macOS, and Windows. A checked-in `rust-toolchain.toml` pins stable Rust plus `rustfmt`/`clippy` for every job.

**Rationale**: Drift is separate from quality; the matrix protects the three supported OS families without running schedulers. One toolchain file prevents version disagreement.

### Cross-platform support (maintainer requirement)

**Choice**: The tooling MUST support macOS, Windows, and Linux distros. CI validates on GitHub-hosted Ubuntu (Linux), macOS, and Windows runners. Local setup documentation MUST provide per-OS installation: Lefthook via Homebrew (macOS/Linux), winget/scoop/npm (Windows), and the official Debian/RPM/Alpine/Arch packages for Linux distros; AgentSync via npm `@dallay/agentsync@1.45.2` (node >=18) on every OS. Windows contributors MUST enable the AgentSync symlink prerequisites (Developer Mode or elevated privileges) per the official Windows Symlink Setup guide; until then `agentsync apply` may fail locally and MUST NOT block Git operations.

**Rationale**: The application already targets all three families (launchd, systemd, Windows Task Scheduler). Hooks and CI must not become the first OS-specific barrier. The non-blocking lifecycle pattern (`agentsync apply || true`) is taken verbatim from the official AgentSync Lefthook guide so behavior stays consistent across shells; `agentsync status --json` remains the authoritative health check on every platform. Windows CI intentionally does not create AgentSync symlinks, keeping the test job green without Developer Mode.

### Pinning and supply-chain constraints

**Choice**: Pin every third-party Action to a full commit SHA with a version comment. Invoke the exact approved AgentSync npm version via Node.js >=18, disabling install scripts where supported; add no package manifest or Cargo dependency.

**Rationale**: Exact versions reduce drift and substitution risk while preserving this Rust-only package. Updates require maintainer review.

## Data Flow

```text
Git operation -> Lefthook -> Rust gate (blocking) or AgentSync refresh (non-blocking)
                                      -> local generated symlinks

GitHub checkout -> pinned toolchains/actions -> quality + platform tests
                                          -> isolated AgentSync apply/status
                                          -> failure reported independently
```

The CI AgentSync job must apply in an isolated/disposable checkout or use the documented no-persist option so `.gitignore` changes do not remain; `status --json` is the blocking assertion.

## File Changes

| File | Action | Description |
|---|---|---|
| `.agents/AGENTS.md` | Create | Reviewed canonical migration of root instructions. |
| `.agents/agentsync.toml` | Create | Explicit three-target mapping (root/Claude/Copilot), symlink mode, managed-ignore policy, no MCP. |
| `AGENTS.md` | Replace locally | Generated root destination; no longer a source. |
| `.gitignore` | Create | Ordinary Rust/system ignores plus AgentSync marker boundary. |
| `lefthook.yml` | Create | Version declaration and five hook behaviors. |
| `rust-toolchain.toml` | Create | Approved Rust channel/version and required components. |
| `.github/workflows/ci.yml` | Create | Pinned actions, quality job, test matrix, AgentSync job. |
| `README.md` | Modify | Installation, versions, hook setup, symlink limits, CI and rollback. |

## Testing Strategy

| Layer | What to Test | Approach |
|---|---|---|
| Configuration | No cycles; only approved targets; ignore ownership | Apply/status in a clean temporary checkout and inspect symlink targets. |
| Local integration | Hook blocking/non-blocking semantics | Run Lefthook with passing/failing Rust commands and without AgentSync. |
| CI integration | Formatting, Clippy, tests, drift | Validate workflow YAML, run Cargo gate, and exercise each matrix runner. |
| E2E | None | No application behavior changes; scheduler E2E is out of scope. |

## Migration / Rollout

Land canonical/config files and README together, validate in a clean checkout, then run `lefthook install`. Rollback removes workflow, Lefthook, AgentSync files, generated destinations, and only the managed ignore block; restore root `AGENTS.md` from version control. No Cargo or application rollback is required.

## Approved Versions (maintainer decision, 2026-08-04)

| Tool | Approved |
|---|---|
| `@dallay/agentsync` (npm) | 1.45.2 (node >=18) |
| Node.js | 22 LTS in CI (local 24 OK) |
| Lefthook | 2.1.10 (local Homebrew) |
| Rust | `rust-toolchain.toml` channel `stable` + components `rustfmt`, `clippy` |
| actions/checkout | v7.0.1 — pinned to full SHA |
| actions/setup-node | v7.0.0 — pinned to full SHA |
| dtolnay/rust-toolchain | v1 — pinned to full SHA |

Target aliases confirmed against official docs: `[agents.root]`, `[agents.claude]`, `[agents.copilot]`. Delivery strategy: 3 logical commits on `main` (no remote yet); revisit PR chaining when a remote exists.
