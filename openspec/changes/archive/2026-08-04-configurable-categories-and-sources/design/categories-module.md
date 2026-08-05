# Categories Module — Signatures and Import Graph

## New file: `src/categories.rs`

### Public surface

```rust
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use serde::Deserialize;

use crate::Config;

/// User-supplied category declaration. Order = TOML declaration order.
/// `replace = false` (default) supplements; `replace = true` substitutes the
/// built-in list for that category name. See category-configuration/spec.md.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub struct CategoryRule {
    pub name: String,
    pub extensions: Vec<String>,
    #[serde(default)]
    pub replace: bool,
}

/// Built-in flat 7-name table: extension → category. The `'static` lifetime
/// lets the map live in a `OnceLock` if apply chooses to optimize; trivially
/// it can also be a fresh `HashMap` per call (matches the current shape at
/// src/lib.rs:325-378).
pub fn default_categories() -> HashMap<&'static str, &'static str>;

/// Classify `path`. Returns `"Other"` for unknown extension AND no-extension
/// files (locked: classification/spec.md:31).
pub fn category_for(path: &Path, config: &Config) -> String;

/// One-shot composition of the resolution map. Called once at `load_config`
/// time (and once per `CategoryRule` edit if apply exposes a rebuilder).
/// Order: built-ins → [[categories]] (replace consumed) → [extensions] last.
pub fn apply_categories(config: &Config) -> HashMap<String, String>;

/// Membership for `is_generated_category`. Always contains the seven flat
/// English names; merged with `config.categories[*].name` (so users can
/// mark their own dirs as "do not re-enter"). Locked: directory-movement/
/// spec.md:21-23.
pub fn is_generated_category_set(config: &Config) -> BTreeSet<String>;

/// True if `path` equals `<root>/<name>` or starts with it, for any name in
/// `is_generated_category_set`. Used by `should_visit` and the in-place
/// pre-classify inside `collect_top_level_directories`.
pub fn is_generated_category(path: &Path, root: &Path, config: &Config) -> bool;

/// Per-rule validation. Called from `lib::validate_config`:
/// - `name` non-empty, NOT absolute, no `..` segment
/// - `extensions` non-empty
/// - `name` unique across rules (rejection, not silent last-write)
pub fn validate_categories(rules: &[CategoryRule]) -> anyhow::Result<()>;
```

### `src/lib.rs` changes

- `Config` (`src/lib.rs:23-34`) gains:
  ```rust
  #[serde(default)]
  pub categories: Vec<crate::categories::CategoryRule>,
  ```
- `validate_config` (`src/lib.rs:92-109`) prepends:
  ```rust
  crate::categories::validate_categories(&config.categories)?;
  ```
  Keeps existing extension and empty-path checks. Order matters: per-rule
  invariants reject malformed `[categories]` BEFORE the per-extension check
  so error messages stay category-specific.
- The constants `DEFAULT_CATEGORY = "Otros"` (`src/lib.rs:11`) and
  `NO_EXTENSION_CATEGORY = "Sin extensión"` (`src/lib.rs:12`) are REMOVED.
  They are replaced by `DEFAULT_CATEGORY: &str = "Other"` declared inside
  `categories.rs` (private to that module). `src/lib.rs` has no leftover
  reference because `category_for` and the fallbacks move with the constant.
- `default_categories`, `category_for`, `is_generated_category` move. Any
  remaining reference inside `src/lib.rs::tests` becomes
  `crate::categories::category_for` etc.
- `mod categories;` declaration is added at the top of `src/lib.rs`. Public
  re-export at `pub mod categories;` is unnecessary unless `main.rs` ever
  imports the types directly — it does not, so a private `mod` declaration
  with `pub(crate)` items suffices.

### Module-level test surface

`src/categories.rs` follows the project's inline-test convention (`src/lib.rs:483-544`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn built_in_coverage() { /* test_5 */ }
    #[test] fn supplement_merges() { /* test_6 */ }
    #[test] fn replace_substitutes() { /* test_7 */ }
    #[test] fn non_colliding_adds() { /* test_8 */ }
    #[test] fn extensions_wins_after_rules() { /* test_9 */ }
    #[test] fn generated_set_has_seven_names() { /* test_4 derived */ }
}
```

`src/lib.rs::tests` keeps its existing inline form and re-exports/uses
`crate::categories::*` only where the moved APIs are exercised (tests
`test_1`–`test_5`).
