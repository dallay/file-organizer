# CI Quality Validation Specification

## Purpose

Validate Rust quality and supported platforms reproducibly while detecting instruction drift.

## Requirements

### Requirement: Trigger, permissions, concurrency, and pinning

The workflow MUST run on pushes and pull requests, request read-only repository contents permission, cancel superseded runs for the same change, and pin every third-party action to a full commit SHA. Rust and external tool versions MUST be explicit and documented.

#### Scenario: Pull request validation

- GIVEN a pull request or push is received
- WHEN GitHub Actions starts
- THEN the workflow runs with read-only permissions and a supersedable concurrency group

#### Scenario: Unpinned action

- GIVEN a third-party action reference is not a full SHA
- WHEN workflow configuration is reviewed
- THEN the configuration MUST be rejected until pinned

### Requirement: Rust quality and platform matrix

A stable Ubuntu quality job MUST run `cargo fmt -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`. A test job MUST run `cargo test` on Linux (Ubuntu), macOS, and Windows without changing scheduler or runtime assumptions. The test matrix MUST cover the three supported OS families; Windows MUST NOT be required to create AgentSync symlinks.

#### Scenario: Cross-platform pass

- GIVEN the repository builds on all approved runners
- WHEN CI executes
- THEN formatting, Clippy, and tests pass in their required jobs

#### Scenario: Platform regression

- GIVEN `cargo test` fails on one matrix runner
- WHEN CI completes
- THEN the test job fails and identifies that platform

### Requirement: Blocking AgentSync drift status

A separate Ubuntu AgentSync job MUST invoke the pinned approved AgentSync package, run status validation in JSON mode, and fail on configuration or generated-target drift. The job MUST keep any apply step isolated from the checkout's committed state and MUST NOT persist changes to the checkout's `.gitignore`; Windows CI MUST NOT be required to create AgentSync symlinks.

#### Scenario: Clean instruction state

- GIVEN canonical instructions, configuration, and generated targets are synchronized
- WHEN the AgentSync job runs
- THEN `status --json` reports clean and the job passes

#### Scenario: Instruction drift

- GIVEN a target or configuration differs from the canonical source
- WHEN the AgentSync job runs
- THEN the job fails independently of Rust test results

### Requirement: Documentation and rollback

README documentation MUST describe CI jobs, required local setup, tool/version ownership, AgentSync drift policy, Windows symlink limitations, and the release pipeline jobs (release-please, binaries, assets, npm, crates.io, Docker) with the secrets they require. Removing the workflow and tooling files MUST be a documented rollback that requires no application or Cargo rollback; publishing MUST have a documented rollback (npm unpublish within 72 hours, crates.io yank, image retag, release deletion).

#### Scenario: Documented recovery

- GIVEN maintainers remove the tooling change
- WHEN they follow the rollback instructions
- THEN CI no longer runs these jobs and application behavior remains unchanged

#### Scenario: Documented release rollback

- GIVEN a release is published with an error
- WHEN maintainers follow the release rollback instructions
- THEN the npm package is unpublished within 72 hours, the crates.io version is yanked, and the GitHub Release and assets are deleted

### Requirement: Release workflow pinning

The release workflow MUST pin every third-party action to a full commit SHA annotated with a version comment, MUST pin exact versions for external tools (cross, Node/npm, Docker buildx/QEMU), and MUST NOT publish to any registry or attach release assets until every preceding job in the pipeline passes.

#### Scenario: Unpinned release action

- GIVEN a third-party action in the release workflow is not a full SHA
- WHEN workflow configuration is reviewed
- THEN the configuration MUST be rejected until pinned

#### Scenario: Gated publish

- GIVEN a build, test, or asset job fails
- WHEN the pipeline reaches a publish step
- THEN no npm package, crate, image, or release asset is published
