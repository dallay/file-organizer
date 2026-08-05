# Delta Spec: source-precedence-and-first-run

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines the precedence among configured sources, environment-driven
auto-detection, and CLI positional directories, plus the first-run
behavior when no config file exists. The locked precedence order
(`intent.md`) is: explicit CLI positional dirs > configured
`source_directories` > `FILE_ORGANIZER_DOWNLOADS` > XDG / platform
default.

## ADDED Requirements

### Requirement: First-Run With Missing Default Config

When the default config path does not exist AND no `--config` flag is
supplied, the system MUST behave as if `Config::default()` were loaded
and MUST run auto-detection to populate `source_directories`. An explicit
`--config` to a missing path MUST still error (preserving current
behavior at `src/main.rs:54-61`).

#### Scenario: Missing default config uses Config::default() (test_23)

- GIVEN no config file at the default path AND no `--config` flag
- WHEN the user invokes `file-organizer run`
- THEN the run proceeds with `Config::default()` plus
  `default_downloads_path` populating `source_directories`
- Ref: new branch in `src/main.rs:50` and `src/lib.rs::load_config`;
  test at `src/lib.rs::tests::test_23`.

#### Scenario: Explicit missing --config still errors (test_24)

- GIVEN `--config /nonexistent/path.toml` AND that path does not exist
- WHEN the user invokes `file-organizer run` or `validate-config`
- THEN the command exits with an error naming the missing path
- Ref: preserves current behavior at `src/main.rs:54-61`; test at
  `src/lib.rs::tests::test_24`.

### Requirement: Source Precedence

The system MUST resolve sources in this strict order, highest priority
first: (a) CLI positional directories; (b) configured
`source_directories`; (c) `FILE_ORGANIZER_DOWNLOADS`; (d) platform
auto-detect. Steps (c) and (d) MUST run only when (a) and (b) are both
empty.

#### Scenario: Configured sources win over env and detection (test_25)

- GIVEN a config with `source_directories = ["/srv/inbox"]` AND
  `FILE_ORGANIZER_DOWNLOADS=/tmp/downloads`
- AND no CLI positional directories
- WHEN `run` resolves sources
- THEN only `/srv/inbox` is processed
- Ref: `src/main.rs:62-67` (extended); test at
  `src/lib.rs::tests::test_25`.

#### Scenario: CLI positional directories override config and detection (test_26)

- GIVEN a config with `source_directories = ["/srv/inbox"]` AND
  `FILE_ORGANIZER_DOWNLOADS=/tmp/downloads`
- AND CLI positional directory `/cli/arg`
- WHEN `run` resolves sources
- THEN only `/cli/arg` is processed
- Ref: preserves current behavior at `src/main.rs:62-64`; test at
  `src/lib.rs::tests::test_26`.
