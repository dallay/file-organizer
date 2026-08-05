# Delta Spec: directory-movement

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines how depth-1 entries that are directories (not files) are treated
during `run()`. Eligible directories move to `Other/<dirname>/`;
generated, empty, symlinked, and (when `ignore_hidden = true`) hidden
directories are skipped. Dry-run mirrors file dry-run at
`src/lib.rs:255-263`: log the planned move, do not create the destination,
do not call `fs::rename`.

## ADDED Requirements

### Requirement: Generated Category Excluded From Recursive Scan

The system MUST treat the seven new flat category names (`Text`, `Other`,
`Executable`, `Compressed`, `Audio`, `Video`, `Image`) as generated when
present at the source root. A generated directory MUST NOT be re-entered
during recursive scans, and its contents MUST be left untouched.

#### Scenario: Pre-existing Image/ is excluded from scan (test_2)

- GIVEN a temp root containing `photo.JPG` and a pre-existing `Image/`
  directory holding `already.JPG`
- WHEN `run` processes the root
- THEN `photo.JPG` moves into `Image/`
- AND `Image/already.JPG` is left unchanged
- Ref: `is_generated_category` at `src/lib.rs:195-206` (rewritten) and
  `should_visit` at `src/lib.rs:178-189`; new
  `src/categories.rs::is_generated_category`; test at
  `src/lib.rs::tests::test_2`.

### Requirement: Top-Level Non-Generated Directory Movement

For every non-generated, non-empty, non-symlinked, non-hidden (when
`ignore_hidden = true`) directory at depth 1, the system MUST move that
directory into `Other/<dirname>/`. The destination directory MUST use the
same `on_conflict` policy as file moves at `src/lib.rs:238-253`.

#### Scenario: Non-generated non-empty directory moves to Other (test_12)

- GIVEN a temp root with a directory `Projects/` containing at least one
  file
- WHEN `run` processes the root
- THEN `Projects/` becomes `Other/Projects/` and the same files remain
  inside it
- Ref: new `src/lib.rs::collect_top_level_directories` and `move_dir`
  helper; test at `src/lib.rs::tests::test_12`.

#### Scenario: Generated top-level directory is not moved (test_13)

- GIVEN a temp root with a directory `Audio/` at depth 1
- WHEN `run` processes the root
- THEN `Audio/` is not moved into `Other/Audio/` and is not reclassified
- Ref: new `src/categories.rs::is_generated_category`; test at
  `src/lib.rs::tests::test_13`.

#### Scenario: Generated Other/ is not re-entered recursively (test_14)

- GIVEN a temp root with a pre-existing `Other/` directory containing
  files AND a top-level `Loose/` directory
- WHEN `run` processes the root recursively
- THEN `Loose/` moves into `Other/Loose/`
- AND no entry into the existing `Other/` is attempted (no scan, no move
  of its contents)
- Ref: `should_visit` at `src/lib.rs:178-189` (extended); test at
  `src/lib.rs::tests::test_14`.

#### Scenario: Empty top-level directory is skipped (test_15)

- GIVEN a temp root with an empty directory `Empty/`
- WHEN `run` processes the root
- THEN `Empty/` is not moved and no `Other/Empty/` is created
- Ref: new `src/lib.rs::is_empty_dir`; test at
  `src/lib.rs::tests::test_15`.

#### Scenario: Symlinked top-level directory is skipped (test_16)

- GIVEN a temp root with a symlink `Linked/` pointing at another
  directory
- WHEN `run` processes the root
- THEN `Linked/` is not moved (the symlink itself remains)
- Ref: `walkdir::DirEntry::file_type().is_symlink()` at
  `src/lib.rs:166-175`; test at `src/lib.rs::tests::test_16`.

#### Scenario: Hidden top-level directory is skipped with ignore_hidden (test_17)

- GIVEN a temp root with a `.cache/` directory and a config with
  `ignore_hidden = true`
- WHEN `run` processes the root
- THEN `.cache/` is not moved
- Ref: `is_hidden` at `src/lib.rs:208-212` and `ignore_hidden` config;
  test at `src/lib.rs::tests::test_17`.
