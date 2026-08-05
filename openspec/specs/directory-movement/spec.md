# Directory Movement

**Capability**: directory-movement
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/directory-movement/spec.md`.

## Purpose

Defines how depth-1 entries that are directories (not files) are treated
during `run()`. Eligible directories move to `Other/<dirname>/`;
generated, empty, symlinked, and (when `ignore_hidden = true`) hidden
directories are skipped. Dry-run mirrors file dry-run semantics: log the
planned move, do not create the destination, do not call `fs::rename`.
Recursive scans MUST NOT re-enter generated directories.

## Requirements

### Requirement: Generated Category Excluded From Recursive Scan

The system MUST treat the seven flat category names (`Text`, `Other`,
`Executable`, `Compressed`, `Audio`, `Video`, `Image`) as generated
when they appear at the source root. A generated directory MUST NOT be
re-entered during recursive scans, and its contents MUST be left
untouched. User-supplied `[[categories]]` names SHALL also be treated as
generated at the source root.

#### Scenario: Pre-existing Image/ is excluded from scan (test_2)

- GIVEN a temp root containing `photo.JPG` and a pre-existing `Image/`
  directory holding `already.JPG`
- WHEN `run` processes the root
- THEN `photo.JPG` moves into `Image/`
- AND `Image/already.JPG` is left unchanged.
- Ref: `src/categories.rs::is_generated_category`
  (`src/categories.rs:160-166`) consulted by
  `src/lib.rs::should_visit` (`src/lib.rs:293-304`).
- Test runner: `cargo test tests::run_moves_files_and_leaves_generated_categories_out_of_scan -- --exact`.

### Requirement: Top-Level Non-Generated Directory Movement

For every non-generated, non-empty, non-symlinked, non-hidden (when
`ignore_hidden = true`) directory at depth 1, the system MUST move that
directory into `Other/<dirname>/`. The destination directory MUST use
the same `on_conflict` policy as file moves.

#### Scenario: Non-generated non-empty directory moves to Other (test_12)

- GIVEN a temp root with a directory `Projects/` containing at least
  one file
- WHEN `run` processes the root
- THEN `Projects/` becomes `Other/Projects/` and the same files
  remain inside it.
- Ref: `src/lib.rs::collect_top_level_directories`
  (`src/lib.rs:252-291`) and `src/lib.rs::move_dir`
  (`src/lib.rs:384-442`).
- Test runner: `cargo test tests::run_moves_top_level_directory_to_other -- --exact`.

#### Scenario: Generated top-level directory is not moved (test_13)

- GIVEN a temp root with a directory `Audio/` at depth 1
- WHEN `run` processes the root
- THEN `Audio/` is not moved into `Other/Audio/` and is not
  reclassified.
- Ref: `src/lib.rs::move_dir` pre-classify
  (`src/lib.rs:275-277`); see also
  `src/categories.rs::is_generated_category`
  (`src/categories.rs:160-166`).
- Test runner: `cargo test tests::generated_top_level_directory_not_moved -- --exact`.

#### Scenario: Generated Other/ is not re-entered recursively (test_14)

- GIVEN a temp root with a pre-existing `Other/` directory containing
  files AND a top-level `Loose/` directory
- WHEN `run` processes the root recursively
- THEN `Loose/` moves into `Other/Loose/`
- AND no entry into the existing `Other/` is attempted (no scan, no
  move of its contents).
- Ref: `src/lib.rs::should_visit` (`src/lib.rs:293-304`) and the
  pre-classify in `src/lib.rs::move_dir` (`src/lib.rs:275-277`),
  both reusing `src/categories.rs::is_generated_category_set`
  (`src/categories.rs:145-155`).
- Test runner: `cargo test tests::generated_other_not_reentered -- --exact`.

#### Scenario: Empty top-level directory is skipped (test_15)

- GIVEN a temp root with an empty directory `Empty/`
- WHEN `run` processes the root
- THEN `Empty/` is not moved and no `Other/Empty/` is created.
- Ref: `src/lib.rs::move_dir` empty-directory check
  (`src/lib.rs:278-281`, `fs::read_dir(entry.path())?.count() == 0`).
- Test runner: `cargo test tests::empty_top_level_directory_skipped -- --exact`.

#### Scenario: Symlinked top-level directory is skipped (test_16)

- GIVEN a temp root with a symlink `Linked/` pointing at another
  directory
- WHEN `run` processes the root
- THEN `Linked/` is not moved (the symlink itself remains).
- Ref: `src/lib.rs::collect_top_level_directories` symlink skip
  (`src/lib.rs:266-268`, `entry.file_type().is_symlink()`); test
  gated `#[cfg(unix)]`.
- Test runner: `cargo test tests::symlinked_top_level_directory_skipped -- --exact`.

#### Scenario: Hidden top-level directory is skipped with ignore_hidden (test_17)

- GIVEN a temp root with a `.cache/` directory and a config with
  `ignore_hidden = true`
- WHEN `run` processes the root
- THEN `.cache/` is not moved.
- Ref: `src/lib.rs::collect_top_level_directories` hidden skip
  (`src/lib.rs:272-274`,
  `config.ignore_hidden && is_hidden(entry.path())`).
- Test runner: `cargo test tests::hidden_top_level_directory_skipped_with_ignore_hidden -- --exact`.
