# Config Validation

**Capability**: config-validation
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/config-validation/spec.md`.

## Purpose

Defines the validation rules that `load_config` and `resolve_config`
apply to user-supplied `[[categories]]` entries before a run starts.
These rules generalize the existing per-extension checks in
`validate_config` so that category `name` fields are checked the same
way `[extensions]` values already are. Validation failures MUST surface
before the first move, with the offending value named in the error.

## Requirements

### Requirement: Category Name Safety

The system MUST reject any `[[categories]]` entry whose `name` field is
an absolute path or contains a `..` segment. Validation failure MUST
occur before the first move, with an error message naming the offending
value.

#### Scenario: Absolute or parent-traversal name is rejected (test_10)

- GIVEN a `[[categories]]` rule with `name = "/etc/passwd"` OR
  `name = "../escape"` OR `name = "Sub/../Other"`
- WHEN `load_config` parses the config
- THEN `load_config` returns `Err` with a message naming the bad value
- AND `run` is never invoked.
- Ref: `src/categories.rs::validate_categories` absolute/`..` check
  (`src/categories.rs:176-181`), invoked from
  `src/lib.rs::validate_config` (`src/lib.rs:92-109`).
- Test runner: `cargo test tests::category_name_absolute_or_parent_traversal_rejected -- --exact`.

### Requirement: Category With Empty Extensions

The system MUST reject any `[[categories]]` entry whose `extensions`
array is empty. Validation failure MUST occur before the first move.

#### Scenario: Empty extensions array is rejected (test_11)

- GIVEN a `[[categories]]` rule with `name = "EmptyCat"` and
  `extensions = []`
- WHEN `load_config` parses the config
- THEN `load_config` returns `Err` describing the empty extension
  list.
- Ref: `src/categories.rs::validate_categories` empty-array check
  (`src/categories.rs:182-184`).
- Test runner: `cargo test tests::category_with_empty_extensions_rejected -- --exact`.

### Requirement: Duplicate Category Names Rejected

The system MUST reject any configuration that declares two
`[[categories]]` entries with the same `name`, whether or not
`replace` is set. Validation failure MUST occur before the first move.

#### Scenario: Duplicate name in [[categories]] is rejected (test_27)

- GIVEN two `[[categories]]` rules with the same `name = "Text"`
- WHEN `load_config` parses the config
- THEN `load_config` returns `Err` describing the duplicate name.
- Ref: `src/categories.rs::validate_categories` duplicate-name check
  (`src/categories.rs:190-194`).
- Test runner: `cargo test categories::tests::validate_categories_rejects_duplicate_names -- --exact`
  (covered alongside `tests::duplicate_category_name_rejected`).
