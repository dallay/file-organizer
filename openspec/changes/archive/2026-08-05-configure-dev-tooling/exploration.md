## Exploration: Configure Lefthook, AgentSync, and CI

### Current State
The repository is a single Rust 2021 Cargo package (`file-organizer`) with reusable organization logic in `src/lib.rs` and Clap-only CLI parsing in `src/main.rs`. It has no existing `.gitignore`, `.github/` workflow, Lefthook configuration, package manifest, Rust toolchain file, or AgentSync configuration. The documented local quality gate is `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`; tests are inline unit tests in `src/lib.rs` and there is no coverage setup.

The repository claims macOS, Linux, and Windows support. Platform behavior is represented by the macOS launchd example, Linux systemd user service/timer, and Windows Task Scheduler instructions in `README.md`; the requested tooling should not alter those runtime assumptions. The current Git repository has only the application files tracked, no configured remote, and the working tree contains untracked project-support files (`AGENTS.md`, `.atl/`, `openspec/`) plus generated `target/` output. There are no main OpenSpec domain specs yet.

Current AgentSync documentation identifies `.agents/AGENTS.md` as the canonical instruction source, `.agents/agentsync.toml` as the configuration, and `agentsync apply` as the symlink reconciler. The recommended installation path is the npm package with Node.js >=18; source installation has additional Rust 1.89+ and Node.js 22.22.0+ requirements. AgentSync can manage a marker-delimited `.gitignore` block, offers `status --json` for CI, and recommends `post-checkout`, `post-merge`, and `post-rewrite` hooks for refreshing ignored local symlinks. Native Windows symlink prerequisites remain a contributor concern.

Current Lefthook documentation describes a dependency-free binary, a checked-in `lefthook.yml`/equivalent configuration, and a one-time `lefthook install` command. Its current examples use hook jobs; the AgentSync guide specifically shows Lefthook lifecycle hooks for `post-checkout`, `post-merge`, and `post-rewrite`.

### Affected Areas
- `openspec/changes/configure-dev-tooling/exploration.md` — this exploration artifact; recommended change name is `configure-dev-tooling`.
- `.gitignore` — currently absent; needs a Rust baseline such as `target/` and a deliberate AgentSync managed-destinations policy.
- `.agents/AGENTS.md` — proposed canonical source. The existing root `AGENTS.md` must be reviewed and migrated/merged rather than discarded.
- `.agents/agentsync.toml` — proposed AgentSync configuration, including the selected agent targets, symlink modes, and gitignore policy.
- `lefthook.yml` — proposed shared Git hook configuration for Rust checks and AgentSync refresh hooks.
- `.github/workflows/ci.yml` (or similarly named workflow) — proposed GitHub Actions checks for formatting, Clippy, tests, and optionally AgentSync status.
- `README.md` — needs contributor setup instructions for installing Lefthook/AgentSync, running `lefthook install`, and understanding symlink/gitignore behavior.
- `AGENTS.md` — likely becomes a generated AgentSync destination or remains a deliberately tracked canonical file; this is an explicit policy decision.
- `Cargo.toml` and `Cargo.lock` — no application dependency change is needed if Lefthook and AgentSync remain external developer tools. A dependency addition would be unnecessary coupling.
- `src/`, `platform/` — no implementation changes are expected; CI should validate existing cross-platform behavior without changing scheduler assumptions.

### Approaches
1. **Integrated local tooling with CI enforcement** — migrate instructions to `.agents/`, configure AgentSync-managed symlink destinations, use Lefthook for `pre-commit` formatting, `pre-push` Clippy/tests, and AgentSync post-operation refreshes, then add GitHub Actions Rust checks plus an AgentSync validation job.
   - Pros: one documented developer workflow; catches formatting and quality issues early; keeps agent symlinks synchronized after branch/merge/rewrite operations; CI can verify AgentSync drift with `status --json`; preserves the repository's existing quality gate.
   - Cons: introduces both external CLIs and a Node-based CI step; local hooks depend on contributors installing the tools; AgentSync can change `.gitignore` and create symlinks in the worktree; cross-platform symlink setup is a contributor risk.
   - Effort: Medium

2. **Minimal local hooks, Rust-only CI** — use Lefthook only for Rust checks and AgentSync lifecycle refreshes locally, while GitHub Actions runs the Rust formatter, Clippy, and tests but does not install or validate AgentSync.
   - Pros: smaller and more reliable CI; no Node dependency in the workflow; AgentSync remains a local developer convenience; lower supply-chain and setup complexity.
   - Cons: malformed or drifting AgentSync configuration can pass CI; contributors may not receive synchronized agent files unless hooks are installed and AgentSync is available; less explicit enforcement of the requested AgentSync setup.
   - Effort: Low/Medium

3. **Pinned tool distribution** — add a repository-managed installation path for exact Lefthook and AgentSync versions (for example, a pinned Lefthook package/binary and a pinned AgentSync npm or release artifact), and use those versions in hooks and CI.
   - Pros: reproducible tool versions and clearer supply-chain review; fewer “works on my machine” differences.
   - Cons: this Rust-only repository has no package-manager manifest; introducing one or downloading platform-specific binaries expands maintenance and cross-platform complexity; exact AgentSync/Lefthook version and checksum policy must be selected and maintained.
   - Effort: High

### Recommendation
Proceed with change `configure-dev-tooling` using Approach 1, but keep the implementation bounded: no Rust production-code changes and no new Cargo dependencies. Adopt `.agents/AGENTS.md` and `.agents/agentsync.toml` as tracked sources, migrate the current root `AGENTS.md` content with an explicit diff review, and use AgentSync’s default managed-gitignore workflow for generated agent destinations unless the maintainer intentionally wants those destinations committed. Keep ordinary Rust ignores (`target/`, and an agreed policy for `.DS_Store`) outside AgentSync’s marker-managed block.

Use Lefthook lifecycle hooks for `post-checkout`, `post-merge`, and `post-rewrite` to run `agentsync apply || true`, matching AgentSync’s official guidance so local setup drift does not block Git operations. Use a lightweight `pre-commit` format check and a `pre-push` Clippy/test gate; CI remains authoritative. Document that `lefthook install` and AgentSync installation are explicit contributor setup steps because this repository has no npm `prepare` script.

Add GitHub Actions with stable Rust, rustfmt, Clippy, and the existing commands. Because the application is explicitly cross-platform, prefer a single Ubuntu quality job plus a test matrix for Ubuntu, macOS, and Windows. If AgentSync is enforced in CI, run it in a separate Ubuntu job through a pinned npm package invocation: apply without changing the CI checkout’s `.gitignore`, then run `status --json`. Pin third-party action revisions to full commit SHAs during design/apply rather than using floating tags. If avoiding the Node toolchain is more important than validating AgentSync in CI, use Approach 2 and state that limitation in the proposal.

### Risks
- Migrating the existing untracked `AGENTS.md` into `.agents/AGENTS.md` can lose project instructions or create a symlink/source cycle unless the content and generated destinations are reviewed before applying AgentSync.
- AgentSync’s default managed-gitignore behavior may ignore root-level generated files such as `AGENTS.md` or `CLAUDE.md`; committing those destinations instead requires the documented `[gitignore].enabled = false` opt-out and a deliberate team decision.
- AgentSync uses symbolic links. Native Windows contributors may need Developer Mode, appropriate permissions, or WSL; CI should avoid assuming Windows symlink creation works unless it is explicitly tested.
- `agentsync apply` in post-operation hooks can modify `.gitignore` and leave a dirty worktree; `|| true` also means missing/broken local tooling can be easy to overlook without a documented health check.
- An unpinned npm package, Lefthook binary, or GitHub Action creates supply-chain and reproducibility risk. Exact versions, action SHAs, and update ownership should be decided before implementation.
- The current working tree has large untracked `target/` output and no `.gitignore`; adding ignore rules will not delete or clean those files. The implementation must not accidentally stage build artifacts.
- There is no configured Git remote, so the workflow cannot be observed on GitHub until repository hosting is connected; local YAML validation and command checks will still be needed.

### Ready for Proposal
Yes. The orchestrator can start `sdd-propose` for `configure-dev-tooling`. The proposal should ask the maintainer to choose: the AgentSync target agents and whether generated destinations are ignored or committed; the Lefthook installation/version policy; whether AgentSync status is enforced in CI; the Rust CI platform matrix; and whether action/tool versions will be pinned by release version, full SHA, or another approved policy.
