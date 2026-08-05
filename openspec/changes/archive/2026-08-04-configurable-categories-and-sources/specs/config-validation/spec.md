# Delta Spec: config-validation

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines the validation rules that `load_config` applies to user-supplied
`[[categories]]` entries before a run starts. These rules generalize the
existing `validate_config` at `src/lib.rs:92-109` so category `name`
fields are checked the same way `[extensions]` values already are.

## ADDED Requirements

### Requirement: Category Name Safety

The system MUST reject any `[[categories]]` entry whose `name` field is an
absolute path or contains a `..` segment. Validation failure MUST occur
before the first move, with an error message naming the offending value.

#### Scenario: Absolute or parent-traversal name is rejected (test_10)

- GIVEN a `[[categories]]` rule with `name = "/etc/passwd"` OR
  `name = "../escape"` OR `name = "Sub/../Other"`
- WHEN `load_config` parses the config
- THEN `load_config` returns `Err` with a message naming the bad value
- AND `run` is never invoked
- Ref: extends `validate_config` at `src/lib.rs:92-109`; test at
  `src/lib.rs::tests::test_10`.

### Requirement: Category With Empty Extensions

The system MUST reject any `[[categories]]` entry whose `extensions`
array is empty. Validation failure MUST occur before the first move.

#### Scenario: Empty extensions array is rejected (test_11)

- GIVEN a `[[categories]]` rule with `name = "EmptyCat"` and
  `extensions = []`
- WHEN `load_config` parses the config
- THEN `load_config` returns `Err` describing the empty extension list
- Ref: extends `validate_config` at `src/lib.rs:92-109`; test at
  `src/lib.rs::tests::test_11`.
