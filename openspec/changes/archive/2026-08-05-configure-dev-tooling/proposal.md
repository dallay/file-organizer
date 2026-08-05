# Proposal: Configure Development Tooling

## Intent

Establish a reproducible contributor workflow for the single-package Rust 2021 project by adding Lefthook hooks, AgentSync-managed instructions, and GitHub CI. The change should reinforce the documented `cargo fmt`, Clippy, and test gate without changing organizer behavior, platform launchers, or Cargo dependencies.

## Scope

### In Scope
- Add tracked AgentSync sources/config, migrate and review the current root `AGENTS.md`, and document generated destinations and `.gitignore` behavior.
- Add Lefthook configuration for formatting, pre-push quality checks, and AgentSync refresh after checkout/merge/rewrite.
- Add GitHub Actions for Rust quality checks and the approved macOS/Linux/Windows test matrix; document setup and version policies.

### Out of Scope
- Rust production-code, scheduler, or runtime behavior changes.
- New Cargo dependencies, coverage tooling, release packaging, or automatic installation through an npm `prepare` script.

## Capabilities

### New Capabilities
- `agent-instruction-sync`: Canonical `.agents/AGENTS.md` plus explicit AgentSync targets, symlink, and managed-ignore policy.
- `local-development-hooks`: Lefthook-installed local quality and AgentSync lifecycle hooks.
- `ci-quality-validation`: GitHub Actions Rust checks, cross-platform tests, and optional AgentSync drift validation.

### Modified Capabilities
- None (no existing main specs are present).

## Approach

Use the exploration’s integrated approach: keep canonical instructions under `.agents/`, preserve ordinary ignores such as `target/` outside AgentSync’s managed block, and use explicit `lefthook install`. Recommend `agentsync apply || true` for lifecycle hooks so missing local tooling does not block Git operations; CI remains authoritative. Recommend a separate blocking AgentSync `status --json` job, a stable Rust quality job on Ubuntu, and tests on Ubuntu, macOS, and Windows. Pin GitHub Actions to full commit SHAs and document exact tool versions. Maintainer approval is required for: AgentSync target list and generated-vs-committed destinations; whether AgentSync status blocks CI; Lefthook installation/distribution and version policy; and the final CI platform matrix.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `.agents/`, `AGENTS.md`, `.gitignore` | New/Modified | Canonical instructions, generated destinations, and ignore policy. |
| `lefthook.yml` | New | Local Rust and AgentSync hooks. |
| `.github/workflows/ci.yml` | New | Formatting, Clippy, tests, and approved matrix. |
| `README.md` | Modified | Contributor installation, hooks, AgentSync, and CI guidance. |
| `src/`, `platform/`, `Cargo.toml` | Unchanged | Explicitly protected from implementation/dependency changes. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| AgentSync migration loses instructions or creates a symlink cycle | Med | Review merged source and every target before apply; test in a clean checkout. |
| Symlinks fail on native Windows | Med | Avoid requiring Windows AgentSync application in the test matrix; document Developer Mode/permissions. |
| Tool/action drift or supply-chain exposure | Med | Pin versions/SHA revisions and assign update ownership. |

## Rollback Plan

Remove the new workflow, Lefthook config, AgentSync config/sources, generated destinations, and managed ignore block; restore the reviewed root `AGENTS.md` from version control. No application or Cargo rollback is required.

## Dependencies

GitHub Actions hosting, Rust stable toolchain, Lefthook, Node.js >=18, and the approved AgentSync npm release.

## Success Criteria

- [ ] A clean checkout can install the documented tools, run `lefthook install`, and synchronize approved instruction targets without cycles.
- [ ] CI passes `cargo fmt -- --check`, Clippy with warnings denied, and `cargo test` on the approved matrix.
- [ ] AgentSync drift policy and all maintainer decisions are explicit in configuration and README documentation.
