# Local Development Hooks Specification

## Purpose

Provide explicit contributor setup and local checks without coupling developer tools to Cargo.

## Requirements

### Requirement: External tool setup and documentation

The repository MUST document installation of Lefthook and the approved AgentSync npm release, the supported Node.js requirement, exact tool/version policy, and one-time `lefthook install`. Lefthook configuration MUST declare its supported minimum version. Lefthook MUST be installed externally or through a package manager and MUST NOT be added to Cargo or an automatic npm `prepare` script.

#### Scenario: Set up a clean checkout

- GIVEN Rust, the documented Node.js version, Lefthook, and AgentSync are available
- WHEN a contributor follows README setup and runs `lefthook install`
- THEN the configured hooks are installed and the approved synchronization workflow is usable

#### Scenario: Missing optional local tool

- GIVEN AgentSync is unavailable locally
- WHEN a lifecycle hook attempts synchronization
- THEN the lifecycle hook MAY report the failure but MUST NOT block checkout, merge, or rewrite operations

### Requirement: Local Rust quality gates

The `pre-commit` hook MUST run `cargo fmt -- --check`. The `pre-push` hook MUST run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test`; failures MUST block the corresponding commit or push.

#### Scenario: Passing quality checks

- GIVEN the working tree is formatted and Rust quality checks pass
- WHEN commit or push hooks run
- THEN the operation proceeds

#### Scenario: Quality failure

- GIVEN formatting, Clippy, or tests fail
- WHEN the corresponding hook runs
- THEN it returns a failure and blocks the operation

### Requirement: AgentSync lifecycle refresh semantics

`post-checkout`, `post-merge`, and `post-rewrite` hooks MUST attempt `agentsync apply || true`, keeping local generated destinations refreshed while treating CI as authoritative.

#### Scenario: Refresh after Git operation

- GIVEN AgentSync is installed and the configuration is valid
- WHEN checkout, merge, or rewrite completes
- THEN the hook attempts to reconcile generated destinations

#### Scenario: Apply cannot reconcile

- GIVEN permissions or native Windows symlink prerequisites prevent reconciliation
- WHEN a lifecycle hook runs
- THEN Git remains successful and the failure is documented as requiring a contributor health check

### Requirement: Cross-platform local setup

The repository MUST document per-OS installation for Lefthook and AgentSync covering macOS, Windows, and Linux distros (including Debian-, RPM-, Alpine-, and Arch-based package managers where applicable). Local hooks MUST behave the same on every OS: blocking hooks block, lifecycle refresh hooks MUST NOT block Git operations even when AgentSync is missing or cannot create symlinks (e.g., Windows without Developer Mode).

#### Scenario: Windows contributor without Developer Mode

- GIVEN AgentSync cannot create symlinks on a Windows machine
- WHEN a lifecycle hook attempts synchronization
- THEN the hook reports the failure without blocking checkout, merge, or rewrite
- AND the documented health check (`agentsync status --json`) identifies the drift

#### Scenario: Linux distro install

- GIVEN a contributor on a Debian-, RPM-, Alpine-, or Arch-based distribution
- WHEN they follow the README setup section
- THEN the documented package-manager installation path works and `lefthook install` registers the hooks

### Requirement: Hook rollback

Removing `lefthook.yml`, uninstalling hooks, and removing AgentSync configuration MUST restore a workflow that relies only on the documented Cargo commands.

#### Scenario: Disable local tooling

- GIVEN the tooling configuration is removed
- WHEN hooks are uninstalled
- THEN no local hook remains required for Rust development
