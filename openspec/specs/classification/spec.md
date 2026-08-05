# Classification

**Capability**: classification
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/classification/spec.md`.

## Purpose

Defines how the file organizer assigns every classified file to exactly one
category directory using the built-in extension map and any
user-supplied `[[categories]]` rules. The resolution order — built-in
defaults, then `[[categories]]` rules in declaration order, then a
last-write pass over `[extensions]` — keeps classification deterministic
and observable. Unknown extensions and extensionless filenames both map
to the catch-all category.

## Requirements

### Requirement: Default Category Set

The system MUST assign every classified file to exactly one of the seven
flat category directories: `Text`, `Other`, `Executable`, `Compressed`,
`Audio`, `Video`, `Image`. Files with unknown extensions and files
without an extension MUST both map to `Other`. The built-in `Text`
extension list MUST include `xls` and `ppt`.

#### Scenario: Case-insensitive extension classification (test_1)

- GIVEN files named `PHOTO.JPG`, `notes.TXT`, `archive.xyz`, and
  `README` (no extension)
- WHEN `category_for` resolves each filename against the built-in
  defaults
- THEN `PHOTO.JPG` maps to `Image`, `notes.TXT` maps to `Text`,
  `archive.xyz` maps to `Other`, and `README` maps to `Other`
- AND the match is case-insensitive on the extension token.
- Ref: `src/categories.rs::category_for` (`src/categories.rs:100`) and
  `src/categories.rs::default_categories` (`src/categories.rs:32`).
- Test runner: `cargo test tests::classifies_extensions_case_insensitively -- --exact`.

#### Scenario: Custom extension override wins (test_3)

- GIVEN a config with `[extensions] pdf = "Review"`
- WHEN `category_for` resolves `document.pdf`
- THEN the result is `Review`, not the built-in `Text` mapping.
- Ref: `src/categories.rs::apply_categories` extensions pass
  (`src/categories.rs:135-137`).
- Test runner: `cargo test tests::custom_extension_overrides_defaults -- --exact`.

#### Scenario: Unknown extension falls back to Other (test_4)

- GIVEN a file `data.unknownext`
- WHEN `category_for` resolves the filename
- THEN the result is `Other`.
- Ref: `src/categories.rs::category_for` fallback to `DEFAULT_CATEGORY`
  (`src/categories.rs:104-114`).
- Test runner: `cargo test tests::category_for_unknown_extension_returns_other -- --exact`.

#### Scenario: Every built-in extension maps to a category (test_5)

- GIVEN every extension listed in `default_categories()`
- WHEN `category_for` resolves each one
- THEN every extension maps to a non-empty, non-`Other` category
  string
- AND no extension maps to the empty string or to a missing category.
- Ref: `src/categories.rs::default_categories` (`src/categories.rs:32`).
  Coverage MUST include `xls` and `ppt` under `Text`.
- Test runner: `cargo test tests::every_builtin_extension_maps_to_nonempty_category -- --exact`.
