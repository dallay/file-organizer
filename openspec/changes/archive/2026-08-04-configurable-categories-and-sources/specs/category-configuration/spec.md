# Delta Spec: category-configuration

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines how user-supplied `[[categories]]` rules combine with the built-in
defaults at resolution time. The supplement-by-default + `replace = true`
opt-in semantics, plus the last-write position of `[extensions]`, are
locked decisions from `intent.md` and MUST NOT be re-opened here.

## ADDED Requirements

### Requirement: Category Rule Supplement and Replace Semantics

The system MUST apply `[[categories]]` rules in the order they appear in
TOML. A rule without `replace` MUST add its `extensions` to the built-in
map without removing any built-in mapping. A rule with `replace = true`
MUST discard the built-in mapping for that category name and substitute
only the rule's `extensions`.

#### Scenario: Supplemental rule adds without removing built-ins (test_6)

- GIVEN a `[[categories]]` rule with `name = "Text"` and
  `extensions = ["foo", "bar"]` and no `replace`
- WHEN the resolver combines rules with built-ins
- THEN `txt`, `pdf`, `md`, and all other built-in `Text` extensions still
  map to `Text`
- AND `foo` and `bar` additionally map to `Text`
- Ref: new `src/categories.rs::resolve_supplement`; test at
  `src/lib.rs::tests::test_6`.

#### Scenario: Replace = true substitutes the built-in list (test_7)

- GIVEN a `[[categories]]` rule with `name = "Text"`, `replace = true`,
  and `extensions = ["onlytxt"]`
- WHEN the resolver combines rules with built-ins
- THEN `txt` is the only built-in extension that maps to `Text`
- AND `pdf`, `md`, `xls`, `ppt`, and other built-ins do NOT map to `Text`
- Ref: new `src/categories.rs::resolve_replace`; test at
  `src/lib.rs::tests::test_7`.

#### Scenario: Non-colliding category is added untouched (test_8)

- GIVEN a `[[categories]]` rule with `name = "Design"` (no built-in
  collision) and `extensions = ["psd", "ai"]`
- WHEN the resolver combines rules with built-ins
- THEN `Design` exists in the resolved map with `psd` and `ai`
- AND `Image`, `Text`, `Other`, and other built-in mappings are unchanged
- Ref: new `src/categories.rs::resolve_non_colliding`; test at
  `src/lib.rs::tests::test_8`.

### Requirement: Extension Override Last-Write

The system MUST apply `[extensions]` AFTER the `[[categories]]` rules
resolve. A key in `[extensions]` MUST win over both built-in and
`[[categories]]` mappings for the same extension.

#### Scenario: [extensions] wins over [[categories]] (test_9)

- GIVEN a `[[categories]]` rule with `name = "Text"` and
  `extensions = ["md"]`, plus `[extensions] md = "Docs"`
- WHEN the resolver combines rules
- THEN `md` maps to `Docs`, not `Text`
- Ref: new `src/categories.rs::apply_extension_override`; test at
  `src/lib.rs::tests::test_9`.
