# Category Configuration

**Capability**: category-configuration
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/category-configuration/spec.md`.

## Purpose

Defines how user-supplied `[[categories]]` rules combine with the
built-in defaults at resolution time. The supplement-by-default plus
`replace = true` opt-in semantics, together with the last-write position
of `[extensions]`, are observable guarantees: callers can extend the
built-in table by adding rules, override an entire built-in category
with `replace = true`, or carve out a fine-grained override via
`[extensions]` that always wins.

## Requirements

### Requirement: Category Rule Supplement and Replace Semantics

The system MUST apply `[[categories]]` rules in the order they appear in
the TOML source. A rule without `replace` MUST add its `extensions` to
the built-in map without removing any built-in mapping. A rule with
`replace = true` MUST discard the existing entries for that category
name (both built-in and any previously applied non-replacing rule) and
substitute only the rule's `extensions` for that name.

#### Scenario: Supplemental rule adds without removing built-ins (test_6)

- GIVEN a `[[categories]]` rule with `name = "Text"` and
  `extensions = ["foo", "bar"]` and no `replace`
- WHEN the resolver combines rules with built-ins
- THEN `txt`, `pdf`, `md`, and every other built-in `Text` extension
  still map to `Text`
- AND `foo` and `bar` additionally map to `Text`.
- Ref: `src/categories.rs::apply_categories` supplement branch
  (`src/categories.rs:126-133`).
- Test runner: `cargo test tests::supplemental_category_rule_adds_extensions -- --exact`.

#### Scenario: Replace = true substitutes the built-in list (test_7)

- GIVEN a `[[categories]]` rule with `name = "Text"`,
  `replace = true`, and `extensions = ["onlytxt"]`
- WHEN the resolver combines rules with built-ins
- THEN `onlytxt` is the only extension that maps to `Text`
- AND `pdf`, `md`, `xls`, `ppt`, and the other built-in `Text`
  extensions do NOT map to `Text` anymore.
- Ref: `src/categories.rs::apply_categories` replace branch
  (`src/categories.rs:127-129`).
- Test runner: `cargo test tests::replace_true_substitutes_builtin_list -- --exact`.

#### Scenario: Non-colliding category is added untouched (test_8)

- GIVEN a `[[categories]]` rule with `name = "Design"` (no built-in
  collision) and `extensions = ["psd", "ai"]`
- WHEN the resolver combines rules with built-ins
- THEN `Design` exists in the resolved map with `psd` and `ai`
- AND `Image`, `Text`, `Other`, and every other built-in mapping are
  unchanged.
- Ref: `src/categories.rs::apply_categories` additive branch
  (`src/categories.rs:126-133`).
- Test runner: `cargo test tests::non_colliding_category_adds_untouched -- --exact`.

### Requirement: Extension Override Last-Write

The system MUST apply `[extensions]` AFTER the `[[categories]]` rules
resolve. A key in `[extensions]` MUST win over both built-in and
`[[categories]]` mappings for the same extension.

#### Scenario: [extensions] wins over [[categories]] (test_9)

- GIVEN a `[[categories]]` rule with `name = "Text"` and
  `extensions = ["md"]`, plus `[extensions] md = "Docs"`
- WHEN the resolver combines rules
- THEN `md` maps to `Docs`, not `Text`.
- Ref: `src/categories.rs::apply_categories` final extension pass
  (`src/categories.rs:135-137`).
- Test runner: `cargo test tests::extension_override_wins_after_rules -- --exact`.
