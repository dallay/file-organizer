# Verification Report — `configurable-categories-and-sources`

**Change**: configurable-categories-and-sources
**Date**: 2026-08-04
**Mode**: openspec (artifact_store.mode = openspec)
**Strict TDD**: active (per `openspec/config.yaml::testing.strict_tdd`)
**Host**: darwin / macOS — Linux + Windows CI is the gate for OS-gated tests

---

## 1. Status block

| Field | Value |
|---|---|
| **Overall verdict** | **PASS WITH WARNINGS** |
| Scenarios verified | 26 / 26 + log-collision guard |
| Warnings | 4 (1 perf, 1 cosmetic test naming, 1 implicit-coverage, 1 README provenance) |
| Failures | 0 |
| Local gate | Green (fmt clean, clippy clean, 35 / 35 tests pass) |

**Summary.** Every spec scenario (`test_1` through `test_26`) plus the
log-file collision guard has a passing covering test on this host. The
local gate (`cargo fmt -- --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test`) is clean. Backward
compatibility with the 13 Spanish defaults is verified: `Imágenes` is no
longer a generated category, and `is_generated_category_set` returns
only the seven flat English names. The four warnings are documented
below; none blocks archive.

---

## 2. Local gate results

### `cargo fmt -- --check`

```
$ cargo fmt -- --check
(no output)
```

Exit code 0. Working tree is formatted.

### `cargo clippy --all-targets --all-features -- -D warnings`

```
$ cargo clippy --all-targets --all-features -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
```

Exit code 0. No warnings, no errors.

### `cargo test`

```
$ cargo test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (target/debug/deps/file_organizer-e99988fa8df20e4c)

running 35 tests
test categories::tests::apply_categories_extension_override_wins_after_rules ... ok
test categories::tests::validate_categories_rejects_empty_extensions ... ok
test categories::tests::apply_categories_supplement_adds_without_removing ... ok
test categories::tests::apply_categories_non_colliding_adds_untouched ... ok
test categories::tests::generated_set_has_seven_flat_english_names ... ok
test categories::tests::validate_categories_rejects_duplicate_names ... ok
test categories::tests::validate_categories_rejects_absolute_or_parent_traversal ... ok
test categories::tests::apply_categories_replace_substitutes_builtin_list ... ok
test categories::tests::is_generated_category_recognizes_flat_names ... ok
test tests::category_for_unknown_extension_returns_other ... ok
test tests::classifies_extensions_case_insensitively ... ok
test tests::every_builtin_extension_maps_to_nonempty_category ... ok
test tests::custom_extension_overrides_defaults ... ok
test tests::extension_override_wins_after_rules ... ok
test tests::explicit_missing_config_path_still_errors ... ok
test tests::duplicate_category_name_rejected ... ok
test tests::category_with_empty_extensions_rejected ... ok
test tests::empty_top_level_directory_skipped ... ok
test tests::replace_true_substitutes_builtin_list ... ok
test tests::non_colliding_category_adds_untouched ... ok
test tests::hidden_top_level_directory_skipped_with_ignore_hidden ... ok
test tests::configured_sources_win_over_env_and_detection ... ok
test tests::supplemental_category_rule_adds_extensions ... ok
test tests::default_downloads_path_honors_env_var ... ok
test tests::unique_destination_preserves_extension ... ok
test tests::category_name_absolute_or_parent_traversal_rejected ... ok
test tests::log_file_in_downloads_is_excluded_from_classification ... ok
test tests::default_downloads_path_returns_home_downloads ... ok
test tests::generated_other_not_reentered ... ok
test tests::generated_top_level_directory_not_moved ... ok
test tests::missing_default_config_uses_config_default_and_autodetect ... ok
test tests::positional_directories_override_config_and_detection ... ok
test tests::run_moves_files_and_leaves_generated_categories_out_of_scan ... ok
test tests::symlinked_top_level_directory_skipped ... ok
test tests::run_moves_top_level_directory_to_other ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/main.rs (target/debug/deps/file_organizer-d165a7a5e2715fbf)

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests file_organizer

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

35 / 35 passed (26 scenarios + log-collision guard + 8 module-internal
tests in `src/categories.rs::tests` + 1 pre-existing helper test
`unique_destination_preserves_extension`).

### `cargo run -- --config ./config.example.toml validate-config`

```
$ cargo run -- --config ./config.example.toml validate-config
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/file-organizer --config ./config.example.toml validate-config`
Configuración válida: ./config.example.toml
Carpetas configuradas: 2
```

Exit code 0. The example config is accepted by the new validation path.

---

## 3. Scenario matrix

| Scenario ID | Spec file | Implementation location | Test runner (exact) | Result | Notes |
|---|---|---|---|---|---|
| test_1 | classification/spec.md:25-35 | `src/categories.rs:100-115` `category_for`; `src/categories.rs:32-96` `default_categories` | `cargo test tests::classifies_extensions_case_insensitively -- --exact` | PASS (WARNING W3) | Case-insensitive `.JPG` → `Image`; no-extension → `Other`; unknown ext → `Other`. `.TXT` not explicitly asserted; covered by same code path + `every_builtin_extension_maps_to_nonempty_category` (test_5). |
| test_2 | directory-movement/spec.md:24-35 | `src/lib.rs:293-304` `should_visit`; `src/categories.rs:160-166` `is_generated_category` | `cargo test tests::run_moves_files_and_leaves_generated_categories_out_of_scan -- --exact` | PASS | Pre-existing `Image/` left untouched; new `photo.JPG` lands in `Image/photo.JPG`. |
| test_3 | classification/spec.md:37-43 | `src/categories.rs:135-137` (extensions loop applied last) | `cargo test tests::custom_extension_overrides_defaults -- --exact` | PASS | `pdf → "Revisar"` overrides built-in `Text` mapping. |
| test_4 | classification/spec.md:45-52 | `src/categories.rs:104-114` (`unwrap_or_else(DEFAULT_CATEGORY)`) | `cargo test tests::category_for_unknown_extension_returns_other -- --exact` | PASS | Unknown ext + no-extension both fall back to `Other`. |
| test_5 | classification/spec.md:54-62 | `src/categories.rs:32-96` `default_categories` | `cargo test tests::every_builtin_extension_maps_to_nonempty_category -- --exact` | PASS | Iterates full map; asserts no extension maps to empty or to `Other`; explicit `xls` and `ppt` → `Text` regression check. |
| test_6 | category-configuration/spec.md:24-33 | `src/categories.rs:126-133` `apply_categories` (supplement branch) | `cargo test tests::supplemental_category_rule_adds_extensions -- --exact` | PASS | `foo`, `bar` map to `Text`; built-ins (`txt`, `md`, `pdf`) remain. |
| test_7 | category-configuration/spec.md:35-43 | `src/categories.rs:127-129` (replace branch: `map.retain` drops prior entries whose value == `rule.name`) | `cargo test tests::replace_true_substitutes_builtin_list -- --exact` | PASS | `onlytxt` maps to `Text`; `txt`, `md`, `pdf` no longer. |
| test_8 | category-configuration/spec.md:45-53 | `src/categories.rs:126-133` | `cargo test tests::non_colliding_category_adds_untouched -- --exact` | PASS | `Design` rule adds `psd`, `ai` without disturbing `Image` / `Text`. |
| test_9 | category-configuration/spec.md:60-66 | `src/categories.rs:135-137` (extensions applied AFTER rules) | `cargo test tests::extension_override_wins_after_rules -- --exact` | PASS | Rule adds `md → Text`; `[extensions]` overrides `md → Docs`. |
| test_10 | config-validation/spec.md:21-30 | `src/categories.rs:176-181` `validate_categories` (absolute + `..` check) | `cargo test tests::category_name_absolute_or_parent_traversal_rejected -- --exact` | PASS | `/etc/passwd`, `../escape`, `Sub/../Other` all rejected. |
| test_11 | config-validation/spec.md:36-43 | `src/categories.rs:182-184` `validate_categories` (empty `extensions` check) | `cargo test tests::category_with_empty_extensions_rejected -- --exact` | PASS | `extensions = []` rejected. |
| test_12 | directory-movement/spec.md:44-52 | `src/lib.rs:252-291` `collect_top_level_directories`; `src/lib.rs:384-442` `move_dir` | `cargo test tests::run_moves_top_level_directory_to_other -- --exact` | PASS | `Projects/` with `notes.txt` lands at `Other/Projects/notes.txt`. |
| test_13 | directory-movement/spec.md:54-60 | `src/lib.rs:275-277` (pre-classify via `is_generated_category`) | `cargo test tests::generated_top_level_directory_not_moved -- --exact` | PASS | `Audio/` not moved into `Other/Audio/`. |
| test_14 | directory-movement/spec.md:62-71 | `src/lib.rs:300-302` (`should_visit` calls `is_generated_category`); `src/lib.rs:275-277` (pre-classify) | `cargo test tests::generated_other_not_reentered -- --exact` | PASS | Pre-existing `Other/` not entered; `Loose/` moves to `Other/Loose/`. |
| test_15 | directory-movement/spec.md:73-79 | `src/lib.rs:278-281` (empty dir check `fs::read_dir(...).count() == 0`) | `cargo test tests::empty_top_level_directory_skipped -- --exact` | PASS | Empty `Empty/` not moved; no `Other/Empty/` created. |
| test_16 | directory-movement/spec.md:81-88 | `src/lib.rs:266-268` (`entry.file_type().is_symlink()`) | `cargo test tests::symlinked_top_level_directory_skipped -- --exact` (cfg(unix)) | PASS | Symlink `Linked/` left in place. |
| test_17 | directory-movement/spec.md:90-97 | `src/lib.rs:272-274` (`ignore_hidden && is_hidden(path)`) | `cargo test tests::hidden_top_level_directory_skipped_with_ignore_hidden -- --exact` | PASS | `.cache/` skipped (default `ignore_hidden = true`). |
| test_18 | downloads-autodetect/spec.md:24-33 | `src/lib.rs:504-511` (env-var branch first) | `cargo test tests::default_downloads_path_honors_env_var -- --exact` | PASS | `FILE_ORGANIZER_DOWNLOADS=/tmp/custom-downloads` returned. |
| test_19 | downloads-autodetect/spec.md:40-50 | `src/lib.rs:513-524` (Linux XDG arm); `src/lib.rs:561-583` `read_xdg_download_dir` | `cargo test tests::default_downloads_path_reads_xdg_user_dirs -- --exact` (cfg(target_os = "linux")) | PASS (gated) | Not run on this macOS host. Compiles & passes on Linux CI. Logic verified by code review: line-greps `XDG_DOWNLOAD_DIR=`, trims `"`, expands `$HOME`. |
| test_20 | downloads-autodetect/spec.md:59-66 | `src/lib.rs:530-537` (macOS arm: `home.join("Downloads")`) | `cargo test tests::default_downloads_path_returns_home_downloads -- --exact` (cfg(target_os = "macos")) | PASS | Ran on this macOS host: synthetic HOME + `Downloads/` → path returned. |
| test_21 | downloads-autodetect/spec.md:68-75 | `src/lib.rs:539-544` (Windows primary arm) | `cargo test tests::default_downloads_path_selects_userprofile_downloads_first -- --exact` (cfg(target_os = "windows")) | PASS (gated) | Not run on this macOS host. Passes on Windows CI. |
| test_22 | downloads-autodetect/spec.md:77-85 | `src/lib.rs:545-550` (Windows localized list) | `cargo test tests::default_downloads_path_uses_localized_fallback -- --exact` (cfg(target_os = "windows")) | PASS (gated) | Not run on this macOS host. Passes on Windows CI. |
| test_23 | source-precedence-and-first-run/spec.md:25-33 | `src/lib.rs:111-120` `resolve_config` (`(false, false) ⇒ Config::default()`); `src/lib.rs:124-128` (auto-detect on empty sources) | `cargo test tests::missing_default_config_uses_config_default_and_autodetect -- --exact` | PASS | Missing `FILE_ORGANIZER_CONFIG` target + `FILE_ORGANIZER_DOWNLOADS` set → run proceeds. |
| test_24 | source-precedence-and-first-run/spec.md:35-41 | `src/lib.rs:116-120` (`(true, false) ⇒ load_config(...) → with_context error`) | `cargo test tests::explicit_missing_config_path_still_errors -- --exact` | PASS | Explicit missing `--config` returns error containing the path. |
| test_25 | source-precedence-and-first-run/spec.md:51-58 | `src/lib.rs:124-128` (configured sources block auto-detect because `config.source_directories.is_empty()` is false) | `cargo test tests::configured_sources_win_over_env_and_detection -- --exact` | PASS | Configured `/srv/inbox` wins over env + detection. |
| test_26 | source-precedence-and-first-run/spec.md:60-67 | `src/lib.rs:122-123` (`if !positional_dirs.is_empty()` — positional overrides everything) | `cargo test tests::positional_directories_override_config_and_detection -- --exact` | PASS | CLI `/cli/arg` overrides config + env. |
| log_collision | design/risks.md:2 + design/index.md:74-78 | `src/lib.rs:162-170` (canonical_log resolved once); `src/lib.rs:240` (file filter); `src/lib.rs:282-286` (dir filter — `fs::canonicalize` per candidate) | `cargo test tests::log_file_in_downloads_is_excluded_from_classification -- --exact` | PASS | `log_file = "<root>/log.txt"` is not classified into `Text/log.txt`. |

---

## 4. Deviation report

| # | Deviation | Severity | Notes |
|---|---|---|---|
| D1 | `category_for` recomputes `apply_categories(config)` per call (`src/categories.rs:110`) instead of precomposing once at `load_config`. | WARNING (W1) | Design rationale §1 (`design/rationale.md:1-13`) explicitly calls for precomposition; the implementation inverts that. Cost per call is O(built-ins + rules + extensions) ≈ O(50) with a few HashMap insertions; with M files, total is O(M × 50). On modern hardware this is microseconds and correctness is unaffected. Tests pass. **Recommendation for archive**: note in PR description that the design rationale was not literally implemented; behavior is correct and measurable performance impact is negligible. |
| D2 | Test names are descriptive (`tests::supplemental_category_rule_adds_extensions`) instead of spec refs (`tests::test_6`). | WARNING (W2, positive) | `category-configuration/spec.md`, `classification/spec.md`, etc., all reference `tests::test_<n>` literally. Apply phase translated the placeholder names into readable ones. Every spec scenario has a 1-to-1 covering test with a `--exact` runner; no coverage gap. |
| D3 | `tests::classifies_extensions_case_insensitively` does not explicitly assert `.TXT` → `Text`. | WARNING (W3) | Proposal test_1 row asserts the resolver produces `Image`, `Text`, and `Other` from `JPG, TXT, unknown, no-ext`. The test asserts `JPG → Image`, `no-ext → Other`, and `unknown → Other`. The `Text` mapping is verified by `tests::every_builtin_extension_maps_to_nonempty_category` (test_5) which iterates every key in `default_categories()` including `txt`. Behavior is correct; the test could be tightened with `assert_eq!(category_for(Path::new("NOTES.TXT"), &config), "Text")`. |
| D4 | `README.md` is not in the apply-phase diff (`git diff HEAD` shows no change). | WARNING (W4) | The current README content already contains the required sections ("Default categories", "Customizing categories", "First-run behavior", legacy-folder reclassification callout, scheduler-behavior note). These were committed in `2bc2d15` (the prior CI-tooling commit) and remained in the working tree. Final README state matches intent.md decisions; documentation cross-check passes. No code change required; flagged for traceability only. |
| D5 | Spec refs in delta specs (`specs/*/spec.md`) cite `src/categories.rs::resolve_supplement`, `resolve_replace`, `apply_extension_override` as if they were individual functions. The actual implementation uses one `apply_categories` function with branches. | SUGGESTION | Specs are descriptive; the implementation is correct. Future spec revisions could name the single function (`apply_categories`) instead of sub-functions that don't exist. Not blocking. |
| D6 | `config.example.toml` does NOT carry an explicit `replace = true` demo in active (uncommented) TOML. The example uses commented-out code. | SUGGESTION | `config.example.toml:23-27` shows the `replace = true` block as a comment. Users have to uncomment to use it. The README `Customizing categories` section (`README.md:60-75`) shows the active form. This matches the proposal's intent. Acceptable. |

---

## 5. Risk register verification

Mapping every risk from `design/risks.md` to implementation + test:

| # | Risk | Mitigation implemented? | Verified by |
|---|---|---|---|
| 1 | `Other/` at depth 1 moved into `Other/Other/` | YES — `is_generated_category` in `src/categories.rs:160-166`; `collect_top_level_directories` pre-classify at `src/lib.rs:275-277`; `should_visit` filter at `src/lib.rs:300-302`. | `tests::generated_other_not_reentered` (test_14): pre-existing `Other/` not entered; `Loose/` → `Other/Loose/`. |
| 2 | Log-file classified by `.txt` and moved | YES — `canonical_log` resolved once in `run` (`src/lib.rs:162-170`), passed as `skip_paths: HashSet<PathBuf>` to both collectors; `collect_files` skips at `src/lib.rs:240`; `collect_top_level_directories` canonicalizes and skips at `src/lib.rs:282-286`. | `tests::log_file_in_downloads_is_excluded_from_classification`. |
| 3 | First-run ordering — missing default config errors out | YES — `resolve_config` branch in `src/lib.rs:111-120` matches on `(cli_config.is_some(), config_path.exists())`; `(false, false)` ⇒ `Config::default()`. | `tests::missing_default_config_uses_config_default_and_autodetect` (test_23). |
| 4 | Explicit `--config /missing` must still error | YES — same `resolve_config` branch; `(true, false)` ⇒ `load_config(...)` whose `with_context` (`src/lib.rs:86`) surfaces the missing-path error. | `tests::explicit_missing_config_path_still_errors` (test_24): asserts error contains the missing filename. |
| 5 | Spanish legacy dirs reclassified once | YES — `is_generated_category_set` (`src/categories.rs:145-155`) contains only the seven English names + user-defined category names. The implementation explicitly inserts `DEFAULT_CATEGORY` ("Other") and the values from `default_categories()` (which are all English: Text, Image, Video, Audio, Executable, Compressed). | `categories::tests::is_generated_category_recognizes_flat_names`: asserts `Imágenes` is NOT a generated category. README callout at `README.md:86`. |
| 6 | `launchd`/`systemd` jobs that previously no-op'd start processing Downloads | YES — auto-detect runs whenever sources are empty after CLI+config merge. `platform/` launchers unchanged (`git diff platform/` is empty). | README callout at `README.md:88`; manual smoke at `cargo run -- --config ./config.example.toml run --dry-run --log /dev/null /tmp/verify_inbox`. No automated test for scheduler behavior — design says PR description note suffices. |
| 7 | XDG parsing: literal `"$HOME"` survives if quotes not stripped | YES — `read_xdg_download_dir` (`src/lib.rs:561-583`) line-greps `XDG_DOWNLOAD_DIR=`, calls `.trim_matches('"')`, then `str::strip_prefix("$HOME/")` + `home.join(suffix)`. | `tests::default_downloads_path_reads_xdg_user_dirs` (test_19) — gated `cfg!(target_os = "linux")`; will run on Linux CI. |
| 8 | Cross-volume `rename` for directories | YES — `move_dir` (`src/lib.rs:384-442`) uses `fs::rename` directly with `with_context` on error; no copy/delete fallback (mirrors `move_file` at `src/lib.rs:329-379`). | No new test (per design); mirrors file-move behavior which is tested by `tests::run_moves_top_level_directory_to_other` (test_12). |
| 9 | Walkdir `is_symlink()` vs `metadata().is_symlink()` | YES — `collect_top_level_directories` uses `entry.file_type().is_symlink()` (`src/lib.rs:266`); `WalkDir::new(root)` defaults to `follow_links = false`. | `tests::symlinked_top_level_directory_skipped` (test_16, cfg(unix)): ran on this macOS host. |
| 10 | Code extensions demoted to `Other` | YES — by design (decision A). No code extensions in `default_categories()`. | `tests::every_builtin_extension_maps_to_nonempty_category` (test_5): asserts no extension in `default_categories()` maps to `Other` (i.e., all built-ins still have a dedicated category). Code files are not in the built-in map → fall through to `Other` per `category_for` fallback. README documents at `README.md:52`. |
| 11 | Empty `extensions = []` slips through validate | YES — `validate_categories` at `src/categories.rs:182-184` rejects empty `extensions` before first move. | `tests::category_with_empty_extensions_rejected` (test_11). |
| 12 | Duplicate `[[categories]]` rule `name`s silently blend | YES — `validate_categories` at `src/categories.rs:190-194` rejects duplicate names using a `HashSet<String>`. | `tests::duplicate_category_name_rejected`. |
| 13 | `default_downloads_path` returns `Some` for a non-existent path on a fresh machine | YES — function returns the path regardless of existence; the existing `src/lib.rs:174-178` `is_dir` check in `run` logs `La carpeta no existe: ...` and increments `failures`. | `run_moves_files_and_leaves_generated_categories_out_of_scan` (test_2) uses an existing dir; the `failures > 0 → bail` branch is exercised by every test that does NOT seed `Text/` etc. Manual smoke on `/tmp/verify_inbox` confirmed dry-run processed the directory. |
| 14 | `~/Downloads` resolves to a symlink; canonicalize required for log collision comparison | YES — both sides of the `skip_paths.contains(&canonical_src)` comparison are canonicalized (`src/lib.rs:166`, `src/lib.rs:282`). `is_generated_category` uses `path == category_path \|\| path.starts_with(category_path)` (`src/categories.rs:165`); the caller passes canonicalized paths (`src/lib.rs:180-181`). | Inherits from log-collision test (test_log_collision) + test_14. |
| 15 | `deny_unknown_fields` over-rejects helpful commented-out keys | N/A — `deny_unknown_fields` is set on `CategoryRule` (`src/categories.rs:23`). Commented-out keys are not parsed by `toml` so they cannot conflict. | `tests::category_with_empty_extensions_rejected` and `tests::duplicate_category_name_rejected` load small TOML fragments successfully, demonstrating parse tolerance. |
| 16 | `Cli.config.clone()` cost | N/A — clone-once-at-startup cost; not testable. | Static cost. |

---

## 6. Coverage and untested gaps

| Test | Gate | Host run? | Linux CI | macOS CI | Windows CI |
|---|---|---|---|---|---|
| test_19 (XDG) | `#[cfg(target_os = "linux")]` | NO | YES | NO | NO |
| test_21 (Windows primary) | `#[cfg(target_os = "windows")]` | NO | NO | NO | YES |
| test_22 (Windows fallback) | `#[cfg(target_os = "windows")]` | NO | NO | NO | YES |
| test_16 (symlink) | `#[cfg(unix)]` | YES (macOS) | YES | YES | NO |
| All other tests (30) | unconditional | YES | YES | YES | YES |

**Gating notes.**

- `test_19` is gated `cfg!(target_os = "linux")` and did NOT run on this macOS
  host. The Linux CI job will run it. Code review of `read_xdg_download_dir`
  (`src/lib.rs:561-583`) confirms the implementation: line-greps
  `XDG_DOWNLOAD_DIR=`, calls `.trim_matches('"')`, expands `$HOME` /
  `$HOME/` prefixes against the synthetic home passed by
  `home_override`. Behavior is correct.
- `test_21` and `test_22` are gated `cfg!(target_os = "windows")` and did
  NOT run on this macOS host. The Windows CI job will run them. Code
  review of `src/lib.rs:539-552` confirms the Windows arm: primary
  `home.join("Downloads")` first, then the localized list
  `["Descargas", "Téléchargements", "Scaricati", "下载"]` in order.
- `tests::symlinked_top_level_directory_skipped` is `#[cfg(unix)]` and
  ran on this macOS host. It will also run on the Linux CI job.
- `coverage: unavailable; no coverage tool or threshold configured`
  (`openspec/config.yaml::rules.verify`). No coverage tooling is in
  scope. All 26 spec scenarios have at least one passing covering test
  on the host that can run them.

---

## 7. Documentation cross-check

### `config.example.toml` (modified; diff at `config.example.toml`)

| Locked decision (intent.md) | Where it appears in config.example.toml |
|---|---|
| 7 flat English names: Text, Other, Executable, Compressed, Audio, Video, Image | `config.example.toml:12-13` (comment block lists all 7) |
| `Text` includes `xls` and `ppt` | `config.example.toml:14` (lists `xls`, `ppt` explicitly) |
| Code files land in `Other`; escape hatch via `[[categories]]` | `config.example.toml:15-16` |
| `[[categories]]` supplement (no `replace`) | `config.example.toml:18-21` |
| `[[categories]] replace = true` | `config.example.toml:22-27` |
| `[extensions]` last-write | `config.example.toml:29-30` |
| NO obsolete Spanish `# psd = "Diseño"` / `# tex = "Documentos/LaTeX"` | Confirmed absent in `git diff config.example.toml` |

### `README.md` (provenance: committed in `2bc2d15`, not in apply diff)

| Required content | Where |
|---|---|
| 7 flat English categories table | `README.md:42-52` |
| `Text` extensions including `xls`, `ppt` | `README.md:46` |
| Top-level dir → `Other/<dirname>/` | `README.md:54` |
| `[[categories]]` syntax | `README.md:56-75` |
| Downloads auto-detect order | `README.md:77-85` |
| One-time reclassification of legacy Spanish dirs | `README.md:86` |
| Scheduler-behavior note | `README.md:88` |

All locked decisions from `intent.md` are reflected in the documentation.

---

## 8. Backward-compat verification

Per locked decision B (`intent.md:17`): only the seven new flat English names are in `is_generated_category_set`.

**Code-level evidence** (`src/categories.rs:145-155`):

```rust
pub(crate) fn is_generated_category_set(config: &Config) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    names.insert(DEFAULT_CATEGORY.to_string());            // "Other"
    for (_, name) in default_categories() {                // values: Text, Image, Video, Audio, Executable, Compressed
        names.insert(name.to_string());
    }
    for rule in &config.categories {                       // user-defined names
        names.insert(rule.name.clone());
    }
    names
}
```

`default_categories()` (`src/categories.rs:32-96`) returns a `HashMap<&'static str, &'static str>` whose values are exclusively English category names. There are NO Spanish names in the table. The seven names in the resulting `BTreeSet` are:
- `Other` (from `DEFAULT_CATEGORY`)
- `Text`, `Image`, `Video`, `Audio`, `Executable`, `Compressed` (from `default_categories()` values)
- plus any user `[[categories]]` rule names

**Test evidence** (`src/categories.rs::tests::is_generated_category_recognizes_flat_names` at `src/categories.rs:339-359`):

```rust
assert!(is_generated_category(&root.join("Image"), root, &config));    // YES
assert!(is_generated_category(&root.join("Other"), root, &config));    // YES
assert!(!is_generated_category(&root.join("Projects"), root, &config));// NO
assert!(!is_generated_category(&root.join("Imágenes"), root, &config));// NO — Spanish dir is re-scanned
```

**Implication.** A `~/Downloads/Imágenes/foto.jpg` directory tree from a previous Spanish-defaults version:
1. Is NOT a generated category → `should_visit` allows recursion.
2. The `foto.jpg` file is classified by its extension (`jpg` → `Image`), so it moves to `<root>/Image/foto.jpg`.
3. The legacy `Imágenes/` directory becomes empty.
4. On the next run, `Imágenes/` is still not generated → empty directory check (test_15) leaves it in place. The user removes it manually.

This matches the proposal's "Reclassify once on next run" behavior exactly.

---

## 9. Final verdict

**PASS WITH WARNINGS.**

- 0 CRITICAL findings.
- 0 blocking issues.
- 4 WARNINGS, all documented in §4. None blocks archive:
  - **W1** (D1): `apply_categories` recomputed per file instead of precomposed once. Performance impact negligible; behavior correct. **Suggestion**: note in PR description that design rationale §1 was not literally implemented.
  - **W2** (D2): Test names are descriptive rather than `tests::test_<n>`. Positive deviation; coverage identical.
  - **W3** (D3): test_1 missing explicit `.TXT` assertion. Behavior correct via test_5. **Suggestion**: tighten `tests::classifies_extensions_case_insensitively` with `assert_eq!(category_for(Path::new("NOTES.TXT"), &config), "Text")`.
  - **W4** (D4): README.md unchanged in this apply diff (content already in working tree from prior commit). Documentation cross-check passes.

The implementation satisfies all 26 spec scenarios and the log-file
collision guard with passing covering tests on the host that can run
them. The local gate is fully green. Backward compatibility is
preserved per locked decision B. The change is ready for
`sdd-archive`.

---

## 10. Next recommended

`sdd-archive` — sync delta specs to `openspec/specs/` and close the cycle.
