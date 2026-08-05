# Delta Spec: classification

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines how the file organizer assigns every classified file to exactly one
category directory using the built-in extension map. The resolution order —
user `[extensions]`, built-in `default_categories()`, then a fixed fallback —
keeps the shape of `category_for` at `src/lib.rs:302-323`; only the values
change to the seven flat English names locked in `intent.md`.

## ADDED Requirements

### Requirement: Default Category Set

The system MUST assign every classified file to exactly one of the seven
flat category directories: `Text`, `Other`, `Executable`, `Compressed`,
`Audio`, `Video`, `Image`. Files with unknown extensions and files without
an extension MUST both map to `Other`. The built-in `Text` extension list
MUST include `xls` and `ppt` per the `intent.md` lock-in.

#### Scenario: Case-insensitive extension classification (test_1)

- GIVEN files named `PHOTO.JPG`, `notes.TXT`, `archive.xyz`, and `README`
  (no extension)
- WHEN `category_for` resolves each filename against the built-in defaults
- THEN `PHOTO.JPG` maps to `Image`, `notes.TXT` maps to `Text`,
  `archive.xyz` maps to `Other`, and `README` maps to `Other`
- AND the match is case-insensitive on the extension token
- Ref: current `category_for` at `src/lib.rs:302-323`; new module
  `src/categories.rs::category_for`; test at
  `src/lib.rs::tests::test_1`.

#### Scenario: Custom extension override wins (test_3)

- GIVEN a config with `[extensions] pdf = "Review"`
- WHEN `category_for` resolves `document.pdf`
- THEN the result is `Review`, not the built-in `Text` mapping
- Ref: current `src/lib.rs:312-316`; new `src/categories.rs::resolve`;
  test at `src/lib.rs::tests::test_3`.

#### Scenario: Unknown extension falls back to Other (test_4)

- GIVEN a file `data.unknownext`
- WHEN `category_for` resolves the filename
- THEN the result is `Other`
- Ref: current fallback at `src/lib.rs:322`; new fallback in
  `src/categories.rs::category_for`; test at
  `src/lib.rs::tests::test_4`.

#### Scenario: Every built-in extension maps to a category (test_5)

- GIVEN every extension listed in `default_categories()`
- WHEN `category_for` resolves each one
- THEN every extension maps to a non-empty, non-Other category string
- AND no extension maps to the empty string or to a missing category
- Ref: `src/categories.rs::default_categories`; test at
  `src/lib.rs::tests::test_5`. Coverage MUST include `xls` and `ppt` under
  `Text`.
