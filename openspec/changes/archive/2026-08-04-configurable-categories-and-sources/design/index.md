# Design: Configurable Categories and Sources

**Change**: configurable-categories-and-sources
**Date**: 2026-08-04
**Source-of-truth**: six delta specs at `specs/*/spec.md`; locked decisions at `intent.md`.

Companion files: [`categories-module.md`](./categories-module.md) (signatures), [`sequences.md`](./sequences.md) (diagrams), [`rationale.md`](./rationale.md) (per-decision), [`risks.md`](./risks.md) (register + mitigations).

## 1. Module split and boundaries

CONFIRM `src/categories.rs` (NEW). `default_categories`, `category_for`, `is_generated_category`, `CategoryRule` (de)serialization, `apply_categories` move out of `src/lib.rs`. `Config.categories: Vec<CategoryRule>` and `validate_config` STAY in `src/lib.rs:23-34` / `src/lib.rs:92-109`; the latter DELEGATES per-rule checks to `categories::validate_categories(&config.categories)`. Constants `DEFAULT_CATEGORY` and `NO_EXTENSION_CATEGORY` are REPLACED by a single `Other` constant inside `categories.rs` (no-extension files now map to `Other` per `classification/spec.md:31`). Full signatures in `categories-module.md`.

## 2. Config schema delta

`Config.categories: Vec<CategoryRule>` (Toml `[[categories]]`). Shape:

```toml
[[categories]]
name = "Text"
extensions = ["foo", "bar"]          # supplement (no `replace`)
[[categories]]
name = "Text"
extensions = ["onlytxt"]
replace = true                        # substitutes built-ins for "Text"
[extensions]
md = "Docs"                           # applied LAST
```

Why `[[categories]]`, not `[categories.<name>]`: two `Text` rules with different `replace` flags cannot coexist under one TOML key. Array-of-tables preserves declaration order AND per-rule `replace` (`category-configuration/spec.md:18-22`). `CategoryRule` uses `#[serde(deny_unknown_fields)]` so typos (`extentions`) error at parse time, not at first move.

## 3. Resolution order

Precomposition runs ONCE per `load_config` (`apply_categories: &Config -> HashMap<String, String>`):

```text
map  := default_categories()                         // built-ins
for rule in config.categories:                       // declaration order
    if rule.replace: retain only entries whose v != rule.name
    for ext in rule.extensions: map[ext.lower()] = rule.name
for (ext, name) in config.extensions:                // last-write wins
    map[ext.lower()] = name
```

Lookup: `category_for(path, config) = apply_categories(config).get(ext).unwrap_or("Other")` — O(1) per file. The same composed map is read by `is_generated_category_set`, so the recursive-scan guard at `src/lib.rs:185-187` and the dir-move pre-classify see one source of truth.

## 4. `is_generated_category` set composition

Seven flat English names ONLY: `Text`, `Other`, `Executable`, `Compressed`, `Audio`, `Video`, `Image`. The 13 legacy Spanish names are EXCLUDED per locked decision B in `intent.md:17` (one-shot reclassification). Reasoning: `rationale.md` §2.

## 5. Directory-movement path

NEW `src/lib.rs::collect_top_level_directories(root, config, skip_paths: &HashSet<PathBuf>) -> Result<Vec<(PathBuf, PathBuf)>>` returns `(source, destination)` pairs. NEW `move_dir` sibling of `move_file` at `src/lib.rs:227-277` mirrors the conflict policy (`Overwrite+is_dir → skip`). Pre-classify reuses `is_generated_category` so a top-level `Other/` is NEVER a moving source — only a destination parent — fixing test_14. Skips: symlinks (`entry.file_type().is_symlink()` under default `follow_links(false)`), hidden (`is_hidden` + `ignore_hidden`), empty (`fs::read_dir(entry).count() == 0`). Dry-run: log `Se movería`, NO `create_dir_all`, NO `rename` (`src/lib.rs:255-263` parity).

## 6. Downloads auto-detection

`pub fn default_downloads_path(home_override: Option<&Path>) -> Option<PathBuf>` lives in `src/lib.rs` next to `default_lock_path` (`src/lib.rs:404-413`). Production caller passes `None`; tests pass `Some(temp.path())` to inject synthetic HOME without env mutation (rationale §4). Algorithm: env `FILE_ORGANIZER_DOWNLOADS` (after `expand_home`) → on Linux, `read_xdg_download_dir(home)` parses `XDG_DOWNLOAD_DIR="$HOME/..."` (strip outer quotes, replace literal `$HOME`) → on mac `<home>/Downloads` → on Windows, first existing of `%USERPROFILE%/Downloads` then localized (`Descargas`, `Téléchargements`, `Scaricati`, `下载`). `cfg!(target_os)` gates join existing gates at `src/lib.rs:62, 405`. Returns `None` only when no arm produced an existing path.

## 7. `main.rs` first-run fallback

`load_config` STAYS a pure parser. New branch in `src/main.rs:50`:

```text
config_path = cli.config.clone().unwrap_or_else(default_config_path)
config = match (cli.config.is_some(), config_path.exists()):
    (false, false) => Config::default()    # test_23: missing default
    _              => load_config(&config_path)?   # test_24: missing --config errors
if config.source_directories.is_empty() && command.directories.is_empty():
    if let Some(d) = default_downloads_path(None):
        config.source_directories.push(d)
```

The `(true, false)` arm hits `load_config`'s `with_context` at `src/lib.rs:78-79` and surfaces the existing missing-path error verbatim.

## 8. Log-file collision guard

DECISION: canonicalize ONCE in `run` (after `Logger::new`). `canonical_log = fs::canonicalize(config.log_file.as_deref()).ok()` becomes part of `skip_paths: HashSet<PathBuf>` passed to BOTH `collect_files` and `collect_top_level_directories`. Each candidate entry canonicalizes in the filter step and drops on match. Protects `log_file = "~/Downloads/log.txt"` (exploration risk 14) without per-iteration `canonicalize`.

## 9. Concurrency / lock semantics

NO CHANGE. `Lock::acquire` (`src/lib.rs:419-434`) keeps `fs::create_dir` atomicity. `default_lock_path` (`src/lib.rs:404-413`) unchanged. Dry-run skip at `src/lib.rs:112-116` unchanged.

## 10. CLI / launcher impact

NO edits to `platform/macos/com.file-organizer.plist.example`, `platform/linux/file-organizer.{service,timer}`, or `README.md:75-82` Windows `schtasks` example. The plist/service invoke `file-organizer run` with no args — they will now hit the auto-detect + first-run-default branch and begin organizing Downloads on each tick. `README.md:46-48` MUST add a "first-run behavior" callout; PR description callout required (rationale §10; risk #6).

## 11. Sequence diagrams

See `sequences.md` — (a) resolution with `[[categories]]` supplement + `[extensions]` override; (b) top-level dir move with conflict policy + dry-run parity, including the test_14 `Other/` guarantee.

## 12. Rationale

See `rationale.md` — eight paragraphs covering precomposition, flat-set decision, array-of-tables, path injection, single-canonical resolve, dir-move interleaving, `load_config` purity, `cli.config` clone.

## 13. Risks and mitigations

See `risks.md` — register covering `Other/` recursion, log-collision, first-run ordering, explicit `--config` errors, legacy-Spanish reclassification, scheduler behavior, XDG parsing, cross-volume `rename`, skips, `Code` demotion, `extensions = []`, duplicate names, auto-detection, symlink canonicalization, `deny_unknown_fields`.

## 14. Citation policy

Line-number citations inside spec scenarios (`src/lib.rs:302-323`, `src/lib.rs:195-206`, etc.) refer to PRE-refactor code. Apply phase keeps these AS-IS for traceability and ADDS new `src/categories.rs::LINE` citations. When both old and new locations exist, the post-refactor `categories.rs` citation is canonical.
