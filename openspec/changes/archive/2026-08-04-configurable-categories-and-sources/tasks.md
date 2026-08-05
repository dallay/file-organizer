# Tasks: configurable-categories-and-sources

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~800–1100 (new `src/categories.rs` + `src/lib.rs`/`src/main.rs` rewrite + README + `config.example.toml` + 26 tests) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1: Phase 1 (module extraction). PR2: Phases 2+3 (classification + `[[categories]]`). PR3: Phase 4 (directory movement). PR4: Phases 5+6 (downloads + first-run). PR5: Phases 7–9 (log guard + docs + verify). |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Extract `src/categories.rs`; rewrite Spanish-named tests; flat 7 names | PR1 | base=trunk; pure refactor; gate stays green at every step |
| 2 | `[[categories]]` schema + supplement/replace/last-write + validation | PR2 | base=trunk; new `Config.categories` field; deny_unknown_fields |
| 3 | Top-level directory movement to `Other/<name>/` | PR3 | base=trunk; new `collect_top_level_directories` + `move_dir`; interleaved with file loop |
| 4 | Downloads auto-detect + first-run fallback + source precedence | PR4 | base=trunk; new `default_downloads_path`; main.rs branch on `(cli.config.is_some(), config_path.exists())` |
| 5 | Log-collision guard + docs + final verify | PR5 | base=trunk; canonical log path; README + `config.example.toml`; final gate |

## Phase 0 — Environment / preflight

- 0.1 VERIFY: confirm `cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` all green before any edit. Cite `openspec/config.yaml::rules.apply` — the strict-TDD contract that every RED task below follows (test-first; one exact-name runner; no production code without a failing test).

## Phase 1 — Module extraction (no behavior change)

- 1.1 RED [test_1]: rewrite `tests::classifies_extensions_case_insensitively` (`src/lib.rs:497-503`) to expect `Image`/`Text`/`Other`. V: `cargo test tests::classifies_extensions_case_insensitively -- --exact` fails (`default_categories` still returns Spanish names).
- 1.2 RED [test_2]: rewrite `tests::run_moves_files_and_leaves_generated_categories_out_of_scan` (`src/lib.rs:523-543`) to seed `Image/`. V: `cargo test tests::run_moves_files_and_leaves_generated_categories_out_of_scan -- --exact` fails.
- 1.3 GREEN: create empty `src/categories.rs`; add `mod categories;` at `src/lib.rs:10`; move `default_categories` (`:325-378`), `category_for` (`:302-323`), `is_generated_category` (`:195-206`) per `design/categories-module.md`; drop `DEFAULT_CATEGORY = "Otros"` (`:11`) and `NO_EXTENSION_CATEGORY = "Sin extensión"` (`:12`); declare `pub(crate) const DEFAULT_CATEGORY: &str = "Other"` in `categories.rs`. V: tests 1+2 rewrites pass.
- 1.4 REFACTOR [test_3]: rewrite `default_categories` body in `src/categories.rs` to flat 7 names (`Image`/`Video`/`Audio`/`Text`/`Other`/`Executable`/`Compressed`); `Text` MUST include `xls` and `ppt` per `intent.md`. Replace `NO_EXTENSION_CATEGORY` fallback in `category_for` with `DEFAULT_CATEGORY`. V: `tests::custom_extension_overrides_defaults` (`:505-510`) still passes against `Text` mapping; local gate green.

## Phase 2 — Classification + flat 7 coverage

- 2.1 RED [test_4]: add `tests::category_for_unknown_extension_returns_other` in `src/lib.rs::tests`; assert `category_for("data.unknownext", &Config::default()) == "Other"`. V: `cargo test tests::category_for_unknown_extension_returns_other -- --exact` fails (still `Otros`).
- 2.2 GREEN: `category_for` in `src/categories.rs` returns `DEFAULT_CATEGORY` for both unknown and no-extension inputs. V: `cargo test tests::category_for_unknown_extension_returns_other -- --exact` passes.
- 2.3 RED [test_5]: add `tests::every_builtin_extension_maps_to_nonempty_category` iterating every key of `crate::categories::default_categories()`; assert no extension maps to empty or to `Other`. V: fails if any demotion (covers `xls`/`ppt` regression risk per `risks.md` #1).
- 2.4 GREEN: confirm `Text` includes `xls` and `ppt` per `intent.md`. V: `cargo test tests::every_builtin_extension_maps_to_nonempty_category -- --exact`; local gate green.

## Phase 3 — `[[categories]]` schema + validation

- 3.1 RED [test_6]: add `tests::supplemental_category_rule_adds_extensions`: `[[categories]] name="Text" extensions=["foo","bar"]` (no `replace`) resolves `foo`, `bar`, AND `txt`/`md` to `Text`. V: fails (no `Config.categories` field).
- 3.2 GREEN: add `CategoryRule` (`#[serde(deny_unknown_fields)]`) + `Config.categories: Vec<CategoryRule>` (`src/lib.rs:23-34`); implement `apply_categories` supplement branch in `src/categories.rs`. V: `cargo test tests::supplemental_category_rule_adds_extensions -- --exact`.
- 3.3 RED [test_7]: add `tests::replace_true_substitutes_builtin_list`: rule `name="Text" replace=true extensions=["onlytxt"]`; only `onlytxt` maps to `Text`; `pdf`/`md` no longer. V: fails (no `replace` field).
- 3.4 GREEN: add `replace: bool` field on `CategoryRule`; implement replace branch in `apply_categories` discarding prior entries where value == `rule.name`. V: `cargo test tests::replace_true_substitutes_builtin_list -- --exact` passes.
- 3.5 RED [test_8]: add `tests::non_colliding_category_adds_untouched`: rule `name="Design" extensions=["psd","ai"]` adds Design without disturbing Image/Text/Other. V: fails.
- 3.6 GREEN: `apply_categories` handles non-colliding names as additive entries. V: `cargo test tests::non_colliding_category_adds_untouched -- --exact` passes.
- 3.7 RED [test_9]: add `tests::extension_override_wins_after_rules`: rule `Text extensions=["md"]` + `[extensions] md="Docs"` → `md` resolves to `Docs`. V: fails (extension applied before rules).
- 3.8 GREEN: `apply_categories` applies `[extensions]` last per `design/index.md:32-42`. V: `cargo test tests::extension_override_wins_after_rules -- --exact` passes.
- 3.9 RED [test_10]: add `tests::category_name_absolute_or_parent_traversal_rejected`: rules `name="/etc/passwd"`, `name="../escape"`, `name="Sub/../Other"` cause `load_config` to `Err`. V: fails (no `validate_categories`).
- 3.10 GREEN: `validate_categories` in `src/categories.rs` rejects absolute paths and `..` segments; hook into `validate_config` (`src/lib.rs:92-109`) before the per-extension check. V: `cargo test tests::category_name_absolute_or_parent_traversal_rejected -- --exact` passes; `cargo run -- --config bad.toml validate-config` errors.
- 3.11 RED [test_11]: add `tests::category_with_empty_extensions_rejected`: rule `name="EmptyCat" extensions=[]`; `validate_config` errors. V: fails.
- 3.12 GREEN: `validate_categories` rejects empty `extensions` arrays; add `tests::duplicate_category_name_rejected` (regression for `risks.md` #12). V: tests 6–11 + duplicate-name all pass; local gate green.

## Phase 4 — Directory movement

- 4.1 RED [test_12]: add `tests::run_moves_top_level_directory_to_other`: seed `Projects/` with one file; `run` produces `Other/Projects/` containing it. V: fails (no `collect_top_level_directories`).
- 4.2 GREEN: add `collect_top_level_directories` + `move_dir` in `src/lib.rs` per `design/sequences.md` diagram (b); interleave with file loop in `run` (`src/lib.rs:133-145`). V: `cargo test tests::run_moves_top_level_directory_to_other -- --exact` passes.
- 4.3 RED [test_13]: add `tests::generated_top_level_directory_not_moved`: seed `Audio/` at depth 1; `run` leaves it. V: fails.
- 4.4 GREEN: `move_dir` pre-classify skips when `is_generated_category(entry.path(), root, config)` is true. V: `cargo test tests::generated_top_level_directory_not_moved -- --exact` passes.
- 4.5 RED [test_14]: add `tests::generated_other_not_reentered`: seed `Other/` with files + `Loose/`; recursive `run` moves `Loose/` to `Other/Loose/` and never enters `Other/`. V: fails.
- 4.6 GREEN: `should_visit` (`src/lib.rs:178-189`) and `collect_top_level_directories` reuse `is_generated_category_set` from `categories.rs`. V: `cargo test tests::generated_other_not_reentered -- --exact` passes.
- 4.7 RED [test_15]: add `tests::empty_top_level_directory_skipped`: seed `Empty/`; no `Other/Empty/` created. V: fails.
- 4.8 GREEN: `move_dir` skips when `fs::read_dir(entry.path())?.count() == 0`. V: `cargo test tests::empty_top_level_directory_skipped -- --exact` passes.
- 4.9 RED [test_16]: add `tests::symlinked_top_level_directory_skipped`: seed symlink `Linked/`; symlink remains. V: fails.
- 4.10 GREEN: `move_dir` skips when `entry.file_type().is_symlink()` (default `follow_links=false`). V: `cargo test tests::symlinked_top_level_directory_skipped -- --exact` passes.
- 4.11 RED [test_17]: add `tests::hidden_top_level_directory_skipped_with_ignore_hidden`: seed `.cache/` with `ignore_hidden=true`; `.cache/` stays. V: fails.
- 4.12 GREEN: `move_dir` honors `config.ignore_hidden && is_hidden(entry.path())` skip. V: tests 12–17 all pass; local gate green.

## Phase 5 — Downloads auto-detection

- 5.1 RED [test_18]: add `tests::default_downloads_path_honors_env_var`: `FILE_ORGANIZER_DOWNLOADS=/tmp/custom` (env-set); `default_downloads_path(Some(temp))` returns it. V: fails (function missing).
- 5.2 GREEN: add `pub fn default_downloads_path(home_override: Option<&Path>) -> Option<PathBuf>` in `src/lib.rs` next to `default_lock_path` (`:404-413`); env-var branch first, `expand_home`-applied. V: `cargo test tests::default_downloads_path_honors_env_var -- --exact` passes.
- 5.3 RED [test_19, cfg!(linux)]: add `tests::default_downloads_path_reads_xdg_user_dirs`: temp `user-dirs.dirs` contains `XDG_DOWNLOAD_DIR="$HOME/Downloads"`; result is `<HOME>/Downloads` with no literal `"$HOME"`. V: fails.
- 5.4 GREEN: add `read_xdg_download_dir(home)` reading `XDG_DOWNLOAD_DIR=`, stripping quotes, expanding `$HOME`. V: `cargo test tests::default_downloads_path_reads_xdg_user_dirs -- --exact` passes on Linux.
- 5.5 RED [test_20, cfg!(macos)]: add `tests::default_downloads_path_returns_home_downloads`: temp HOME with `Downloads/`; result is `<HOME>/Downloads`. V: fails.
- 5.6 GREEN: macOS arm `home.join("Downloads")`. V: `cargo test tests::default_downloads_path_returns_home_downloads -- --exact` passes on macOS host.
- 5.7 RED [test_21, cfg!(windows)]: add `tests::default_downloads_path_selects_userprofile_downloads_first`: temp HOME with `Downloads/`; result is `<HOME>/Downloads`. V: fails.
- 5.8 GREEN: Windows primary arm. V: `cargo test tests::default_downloads_path_selects_userprofile_downloads_first -- --exact` passes on Windows host.
- 5.9 RED [test_22, cfg!(windows)]: add `tests::default_downloads_path_uses_localized_fallback`: temp HOME has `Descargas/` but no `Downloads/`; result is `<HOME>/Descargas`. V: fails.
- 5.10 GREEN: Windows fallback list `["Descargas","Téléchargements","Scaricati","下载"]` walked in order. V: tests 18–22 green on their respective hosts.

## Phase 6 — First-run fallback + precedence

- 6.1 RED [test_23]: add `tests::missing_default_config_uses_config_default_and_autodetect`: default path absent + no `--config`; `run` proceeds with `Config::default()` + `default_downloads_path`. V: fails (`load_config` errors per `src/lib.rs:78-79`).
- 6.2 GREEN: refactor `src/main.rs:50-67` to clone `cli.config`, match `(cli.config.is_some(), config_path.exists())` per `design/index.md:60-66`; `(false, false)` falls through to `Config::default()` + auto-detect. V: `cargo test tests::missing_default_config_uses_config_default_and_autodetect -- --exact` passes.
- 6.3 RED [test_24]: add `tests::explicit_missing_config_path_still_errors`: `--config /nonexistent/path.toml` exits with error naming the path. V: fails if auto-detect accidentally triggers.
- 6.4 GREEN: `(true, false)` arm hits `load_config`'s `with_context` (`src/lib.rs:78-79`) verbatim. V: `cargo test tests::explicit_missing_config_path_still_errors -- --exact` passes.
- 6.5 RED [test_25]: add `tests::configured_sources_win_over_env_and_detection`: config `source_directories=["/srv/inbox"]` + `FILE_ORGANIZER_DOWNLOADS=/tmp/dl` + no CLI; only `/srv/inbox` processed. V: fails.
- 6.6 GREEN: `main.rs:62-67` precedence: positional > config > env > detection. V: `cargo test tests::configured_sources_win_over_env_and_detection -- --exact` passes.
- 6.7 RED [test_26]: add `tests::positional_directories_override_config_and_detection`: config + env + positional `/cli/arg`; only `/cli/arg` processed. V: fails.
- 6.8 GREEN: CLI positional dirs continue to override (existing behavior preserved). V: `cargo test tests::positional_directories_override_config_and_detection -- --exact` passes; local gate green.

## Phase 7 — Log-file collision guard

- 7.1 RED [design risk #11]: add `tests::log_file_in_downloads_is_excluded_from_classification`: config `log_file="<root>/log.txt" source_directories=["<root>"]`; log file is created (Logger) but NOT classified/moved. V: fails.
- 7.2 GREEN: `run` (`src/lib.rs:111-156`) computes `canonical_log = fs::canonicalize(config.log_file.as_deref()).ok()` once after `Logger::new`; passes `skip_paths: HashSet<PathBuf>` to BOTH `collect_files` and `collect_top_level_directories` per `design/index.md:74-78`. V: 7.1 passes; local gate green.

## Phase 8 — Documentation

- 8.1 DOC: rewrite `config.example.toml:12-15` replacing Spanish `# psd = "Diseño"` / `# tex = "Documentos/LaTeX"` with an English `[[categories]]` demo block from `design/index.md:17-27` (one supplement + one `replace = true` + `[extensions]` last-write).
- 8.2 DOC: rewrite `README.md:40-42` ("Reglas integradas") to list the seven flat English names with `Text` extensions (must include `xls` and `ppt`). Add "first-run behavior" callout near `README.md:46-48` documenting auto-detect + one-shot reclassification of legacy Spanish dirs per `risks.md` #5.

## Phase 9 — Final verification

- 9.1 VERIFY: full local gate `cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`; confirm scenarios 1–26 all green via `--exact` runner; `cargo run -- --config ./config.example.toml validate-config` exits 0; smoke `cargo run -- --config ./config.example.toml run --dry-run --log /dev/null /tmp/inbox` on a fresh tempdir and assert 0 unexpected moves. Update `state.yaml`: `tasks` complete, next=`apply`.

## Cross-cutting checklist

- Local gate (`fmt`, `clippy`, `test`) green at every phase end (`AGENTS.md:14`, `openspec/config.yaml::rules.verify`).
- New tests use runner pattern `cargo test tests::<name> -- --exact` (`AGENTS.md:13`, `openspec/config.yaml::rules.apply::test_command`).
- No new crates in `Cargo.toml`; no edits to `Cargo.toml` or `Cargo.lock` (`intent.md::Out of scope`).
- Commit messages use conventional commits without `Co-Authored-By` or AI attribution (global rule).
- `platform/` launchers untouched; behavior change (auto-detect starts processing on `launchd`/`systemd` ticks) documented in README + PR description (`risks.md` #6).
- Citation policy: keep pre-refactor `src/lib.rs:LINE` cites for traceability AND add `src/categories.rs::LINE` cites in new tasks (`design/index.md:99-100`).
- Phases 3 and 4 sit at the ~12 sub-task boundary; each sub-task is single-file/single-function scope and should fit one session. If either grows during apply, split into Phase 3a/3b (schema vs validation) and Phase 4a/4b (move vs skips).
