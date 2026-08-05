# Delta Spec: ci-quality-validation

**Change**: 2026-08-05-distribution-pipeline
**Status**: Proposed

## Purpose

Extends the existing CI spec so the new release pipeline follows the same conventions: every third-party action pinned to a full commit SHA, explicit tool versions, gated publishing, and README documentation of release jobs and rollback.

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Documentation and rollback

README documentation MUST describe CI jobs, required local setup, tool/version ownership, AgentSync drift policy, Windows symlink limitations, and the release pipeline jobs (release-please, binaries, assets, npm, crates.io, Docker) with the secrets they require. Removing the workflow and tooling files MUST be a documented rollback that requires no application or Cargo rollback; publishing MUST have a documented rollback (npm unpublish within 72 hours, crates.io yank, image retag, release deletion).
(Previously: README documented CI jobs, local setup, tool/version ownership, AgentSync drift policy, and Windows symlink limitations only.)

#### Scenario: Documented recovery

- GIVEN maintainers remove the tooling change
- WHEN they follow the rollback instructions
- THEN CI no longer runs these jobs and application behavior remains unchanged

#### Scenario: Documented release rollback

- GIVEN a release is published with an error
- WHEN maintainers follow the release rollback instructions
- THEN the npm package is unpublished within 72 hours, the crates.io version is yanked, and the GitHub Release and assets are deleted
