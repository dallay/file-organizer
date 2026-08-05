# Release Distribution Pipeline Specification

**Change**: 2026-08-05-distribution-pipeline
**Status**: Proposed

## Purpose

Rebrand the CLI to `organiza` and distribute it through release-please-driven releases: GitHub Release assets, crates.io, and an npm wrapper (`@dallay/organiza`) plus six platform packages. One release-please version MUST stay synchronized across every artifact so `cargo install organiza` and `npm i -g @dallay/organiza` work from the same tag. No organizer logic changes.

## Requirements

### Requirement: Rebrand and publish metadata

The Cargo package MUST be named `organiza` and produce a binary named `organiza`. `Cargo.toml` MUST declare repository, readme, license (MIT), keywords, categories, authors, an exclude list, and `[profile.release]`. A `LICENSE` file MUST exist at the repository root.

#### Scenario: Rebranded crate

- GIVEN the distribution branch is built
- WHEN `cargo build --release` runs
- THEN it produces a binary named `organiza`
- AND `cargo metadata` reports package name `organiza`

#### Scenario: License present

- GIVEN the repository is checked before release
- THEN a `LICENSE` file exists and `Cargo.toml` declares `license = "MIT"`

### Requirement: release-please versioning

release-please (release-type rust) MUST bump the version in `Cargo.toml` and in every npm `package.json` via extra-files jsonpaths, keeping all artifacts on one version. `.release-please-manifest.json` MUST track the current version.

#### Scenario: Version sync on release PR

- GIVEN release-please opens a release PR
- WHEN the version is bumped to X.Y.Z
- THEN `Cargo.toml`, `@dallay/organiza`, and all six platform packages declare exactly X.Y.Z

### Requirement: npm wrapper behavior

`@dallay/organiza` MUST be a Node wrapper that spawns the platform binary for the current os/arch, passes argv unchanged, forwards stdout/stderr, and exits with the child's exit code. On an unsupported platform or architecture it MUST exit non-zero with a clear error naming the unsupported target.

#### Scenario: Wrapper spawns binary

- GIVEN a supported platform (e.g., darwin-arm64)
- WHEN the user runs `organiza --version`
- THEN the wrapper spawns the matching platform binary and prints its version

#### Scenario: Unsupported platform error

- GIVEN a platform with no published binary package
- WHEN the user runs the wrapper
- THEN the wrapper prints a clear error and exits non-zero

### Requirement: Platform packages and exact pins

Six platform packages MUST exist: darwin-arm64, darwin-x64, linux-arm64, linux-x64, windows-arm64, windows-x64. The base wrapper MUST declare them as optionalDependencies pinned to the exact release version, without ranges.

#### Scenario: Exact optional dependencies

- GIVEN a release at version X.Y.Z
- WHEN the wrapper's package.json is inspected
- THEN every optionalDependency is pinned to exactly X.Y.Z

### Requirement: Publish order and gating

Platform packages MUST be published before the base wrapper, and GitHub Release assets MUST be attached before or with npm publish. A failure in any publishing step MUST halt the pipeline.

#### Scenario: Dependency-order failure

- GIVEN the base wrapper is published before its platform packages
- WHEN the pipeline runs
- THEN publishing fails and the pipeline halts

### Requirement: Cross-compiled release assets

The pipeline MUST build 8 targets (linux x86_64/aarch64 gnu+musl, darwin x64/arm64, windows x86_64/aarch64-msvc), produce tar.gz/zip archives, and upload each archive with a sha256 checksum to the GitHub Release.

#### Scenario: Assets with checksums

- GIVEN a GitHub Release is created
- WHEN assets are uploaded
- THEN each archive has a matching `.sha256` file

### Requirement: crates.io publish

The pipeline MUST publish with `cargo publish --locked`. The crate MUST carry license and repository metadata before publishing.

#### Scenario: Registry publish

- GIVEN the crate passes all quality gates
- WHEN `cargo publish --locked` runs
- THEN version X.Y.Z is published to crates.io

### Requirement: npm provenance

npm publishing SHOULD use `npm publish --provenance` (npm >= 9) so releases carry signed attestations where the registry supports it.
