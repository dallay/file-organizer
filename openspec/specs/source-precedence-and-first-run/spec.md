# Source Precedence and First-Run Behavior

**Capability**: source-precedence-and-first-run
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/source-precedence-and-first-run/spec.md`.

## Purpose

Defines the precedence among configured sources, environment-driven
auto-detection, and CLI positional directories, plus the first-run
behavior when no config file exists. The precedence order is:
explicit CLI positional directories > configured `source_directories`
> `FILE_ORGANIZER_DOWNLOADS` > XDG / platform default. Steps (c) and
(d) MUST run only when (a) and (b) are both empty.

## Requirements

### Requirement: First-Run With Missing Default Config

When the default config path does not exist AND no `--config` flag is
supplied, the system MUST behave as if `Config::default()` were loaded
and MUST run auto-detection to populate `source_directories`. An
explicit `--config` to a missing path MUST still error.

#### Scenario: Missing default config uses Config::default() (test_23)

- GIVEN no config file at the default path AND no `--config` flag
- WHEN the user invokes `file-organizer run`
- THEN the run proceeds with `Config::default()` plus
  `default_downloads_path` populating `source_directories`.
- Ref: `src/lib.rs::resolve_config` `(false, false)` arm
  (`src/lib.rs:111-120`) and the auto-detect trigger
  (`src/lib.rs:124-128`).
- Test runner: `cargo test tests::missing_default_config_uses_config_default_and_autodetect -- --exact`.

#### Scenario: Explicit missing --config still errors (test_24)

- GIVEN `--config /nonexistent/path.toml` AND that path does not
  exist
- WHEN the user invokes `file-organizer run` or `validate-config`
- THEN the command exits with an error naming the missing path.
- Ref: `src/lib.rs::resolve_config` `(true, false)` arm
  (`src/lib.rs:116-120`) surfacing the missing-path error from
  `src/lib.rs::load_config` (`src/lib.rs:86`).
- Test runner: `cargo test tests::explicit_missing_config_path_still_errors -- --exact`.

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
- THEN only `/srv/inbox` is processed.
- Ref: `src/lib.rs::resolve_config` source-selection logic
  (`src/lib.rs:124-128`); config sources block the auto-detect
  because `config.source_directories.is_empty()` is false.
- Test runner: `cargo test tests::configured_sources_win_over_env_and_detection -- --exact`.

#### Scenario: CLI positional directories override config and detection (test_26)

- GIVEN a config with `source_directories = ["/srv/inbox"]` AND
  `FILE_ORGANIZER_DOWNLOADS=/tmp/downloads`
- AND CLI positional directory `/cli/arg`
- WHEN `run` resolves sources
- THEN only `/cli/arg` is processed.
- Ref: `src/lib.rs::resolve_config` positional-override branch
  (`src/lib.rs:122-123`).
- Test runner: `cargo test tests::positional_directories_override_config_and_detection -- --exact`.

### Requirement: Log File Excluded From Classification

The system MUST NOT classify or move the configured log file even if
its path lies under one of the resolved source roots. The check MUST
compare canonicalized paths so symlinked Downloads roots resolve
correctly.

#### Scenario: Log file inside Downloads is excluded from classification (test_28)

- GIVEN `log_file = "<root>/log.txt"` and
  `source_directories = ["<root>"]`
- WHEN `run` processes the root
- THEN the log file is created by `Logger` and is NOT classified or
  moved into any category directory.
- Ref: `src/lib.rs::run` canonical-log computation
  (`src/lib.rs:162-170`); exclusion applied in
  `src/lib.rs::collect_files` (`src/lib.rs:240`) and
  `src/lib.rs::collect_top_level_directories` (`src/lib.rs:282-286`)
  via a `skip_paths: HashSet<PathBuf>`.
- Test runner: `cargo test tests::log_file_in_downloads_is_excluded_from_classification -- --exact`.
