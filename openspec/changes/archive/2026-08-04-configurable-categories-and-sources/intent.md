# sdd-propose intent — configurable-categories-and-sources

## What we are committing to

A single durable change that turns the existing 13-category Spanish defaults into a user-tunable configuration while removing the friction of "must edit TOML before first run".

## Decisions locked in this round

| Topic | Decision | Owner-approved this session |
|---|---|---|
| Default category set | 7 flat English names: `Text`, `Other`, `Executable`, `Compressed`, `Audio`, `Video`, `Image` | yes |
| `Text` extension set | txt, md, rtf, doc, docx, pages, pdf, xlsx, **xls**, pptx, **ppt**, key, numbers, csv, epub, odt, ods, odp, log, tex | yes |
| Code/source files | No built-in `Code` category; they map to `Other` (per Option A) | yes |
| `[[categories]]` semantics | Supplement built-ins by default; `replace = true` substitutes per-category | yes |
| `[extensions]` | Continues to work as a fine-grained last-write override, applied after `[[categories]]` | yes |
| Default source resolution | Env `FILE_ORGANIZER_DOWNLOADS` → Linux `XDG_DOWNLOAD_DIR` (from `~/.config/user-dirs.dirs`) → mac `~/Downloads` → Windows `%USERPROFILE%/Downloads` with localized fallbacks (`Descargas`, `Téléchargements`, `Scaricati`, `下载`) | yes |
| Backward compat with old Spanish dirs | Only the 7 new names are in `is_generated_category` (Option B). Old dirs like `~/Downloads/Imágenes/` get re-scanned and reclassified on next run | yes |
| First-run without `config.toml` | Use `Config::default()` and run auto-detection when the default config path does not exist; explicit `--config` still errors (Option A) | yes |
| Directory movement | Top-level (depth 1) dirs that are not recognized as generated categories move to `Other/<dirname>/`. Same `on_conflict` policy as files. Empty dirs and symlinked dirs are skipped; `ignore_hidden` is respected | yes |
| Depth control | Keep `recursive: bool` for this change. Configurable `max_depth` is out of scope (separate future change) | yes |
| TDD workflow | Strict TDD per `openspec/config.yaml::apply`. Existing tests in `src/lib.rs::tests` that reference Spanish category names must be rewritten; new coverage per behavior | yes |

## Out of scope (explicit)

- Configurable maximum depth (`max_depth`) — separate future change.
- New runtime dependencies. No new crates in `Cargo.toml`.
- Wider i18n (`default_locale`, message translations) — not asked for.
- Telemetry, hooks, IPC, plugin system — not asked for.
- Changes to install/launcher layout in `platform/` — unchanged; behavior change (auto-detect) is documented in README.

## In scope

- `src/lib.rs` (split: extract `src/categories.rs`; keep auto-detect + lock + logger + run + main entry helpers here).
- `src/categories.rs` (new module: `default_categories`, `category_for`, `is_generated_category`, `CategoryRule` deserialization).
- `src/main.rs` (wire Config::default() fallback for missing default path; auto-detect for empty `source_directories`).
- `config.example.toml` (demo `[[categories]]`, drop obsolete Spanish comment).
- `README.md` (replace "Reglas integradas" Spanish list with the 7 flat categories; document `[[categories]]`; document Downloads auto-detect; document one-time re-classification of legacy Spanish dirs).
- Inline tests in `src/lib.rs::tests` (rewrite Spanish-name assertions; add new tests per spec).

## Approach (one paragraph, high level)

Two new sources of truth for classification exist after this change: the built-in `default_categories()` table and the user-supplied `[[categories]]` array. Resolution order at runtime: (1) iterate `config.categories` in order, applying each entry — supplement or replace per `replace` flag; (2) fall back to `default_categories()` for any extension not yet covered; (3) after both, apply `[extensions]` as a last-write override. The decision live in a new `src/categories.rs` to keep `src/lib.rs` thin and the resolution rules testable in isolation. Downloads auto-detection stays inline in `src/lib.rs` near `default_lock_path()` because it is ~30 lines, has no extra deps, and shares the same OS gating (`cfg!(windows)`). Directory movement is implemented as a parallel collector inside `run()` that mirrors the file loop's conflict policy and dry-run semantics, skipping empty and symlinked entries and respecting `ignore_hidden`.

## Affected modules and ownership

- `src/categories.rs` (NEW, owns classification rules and the `is_generated_category` set composition).
- `src/lib.rs` (loses `default_categories`, `category_for`, `is_generated_category`; gains `default_downloads_path`, refactored `run`, new `collect_top_level_directories`, refactored `load_config` to expose a "missing default config is OK" branch).
- `src/main.rs` (apply `Config::default()` when default config path is absent and `--config` is unset; trigger `default_downloads_path` after `load_config` if `source_directories` is empty).
- `config.example.toml` (English categories in comments; demo `[[categories]]`).
- `README.md` (update default categories, extension syntax, scheduler behavior note).

## Rollback plan

The change is a single feature branch; reverting the merge commit (`git revert`) cleanly restores the old 13-category defaults, the old CLI behavior, and the old `load_config` failure mode. No data migration is required because files moved by the new version are still in their categorized subfolders — the user may see `Imágenes/` re-scanned once on the next run after revert, identical to the previous run-once-then-skip behavior.

A targeted rollback without revert is also possible: re-introduce the 13 Spanish names into `default_categories()` and zero out the `[categories]` parsing path. The auto-detection can be disabled per-call by setting `FILE_ORGANIZER_DOWNLOADS=` empty if undesired.

## Test coverage plan (TDD-mapped)

Each behavior will have at least one failing test written before the production change.

1. **Rewrite**: `tests::classifies_extensions_case_insensitively` — expect new `Image` / `Other` / `Text` strings.
2. **Rewrite**: `tests::run_moves_files_and_leaves_generated_categories_out_of_scan` — use `Image/` instead of `Imágenes/`.
3. **Update**: `tests::custom_extension_overrides_defaults` — verify override still wins with new flat names.
4. **NEW**: `category_for` returns `Other` for unknown extensions.
5. **NEW**: `category_for` covers all extensions in `default_categories()` (no regression).
6. **NEW**: `[[categories]]` supplement adds extensions without removing built-ins.
7. **NEW**: `[[categories]] replace = true` substitutes the built-in list for that category.
8. **NEW**: New category name (no built-in collision) is added without touching others.
9. **NEW**: `[extensions]` override applied last, wins over `[[categories]]`.
10. **NEW**: `Text` rejects absolute path or `..` in `name` of `[[categories]]` (existing validator generalized).
11. **NEW**: `[[categories]]` with empty `extensions` array fails validation.
12. **NEW**: Top-level directory moves to `Other/<dirname>/` when not a generated category.
13. **NEW**: Top-level directory that matches a generated category name is not moved.
14. **NEW**: `Other/` (already a generated category) is not re-entered on recursive scan.
15. **NEW**: Empty top-level directory is skipped.
16. **NEW**: Symlinked top-level directory is skipped.
17. **NEW**: `ignore_hidden = true` skips dotfile dirs.
18. **NEW**: `default_downloads_path` honors `FILE_ORGANIZER_DOWNLOADS`.
19. **NEW**: `default_downloads_path` reads `XDG_DOWNLOAD_DIR` from a mocked `user-dirs.dirs`.
20. **NEW**: `default_downloads_path` returns `~/Downloads` on macOS path.
21. **NEW (gated `cfg!(windows)`)**: `default_downloads_path` returns `%USERPROFILE%/Downloads` first on Windows.
22. **NEW (gated `cfg!(windows)`)**: localized fallback used when primary missing.
23. **NEW**: `Config::default()` is used when the default config path does not exist and `--config` is unset.
24. **NEW**: explicit `--config` to a missing path still errors.
25. **NEW**: explicit `source_directories` in config wins over env var and detection.
26. **NEW**: existing CLI positional dirs override `source_directories` after auto-detect.

## Risks (from sdd-explore)

1. **Coverage regression** — mitigated by Test #5 above; explicit `xls` and `ppt` added to `Text` per user "xlsx, pptx, etc.".
2. **Code files land in `Other`** — by design (decision A). Documented in README; escape hatch via `[[categories]]`.
3. **`is_generated_category` name collisions** — new flat 7-name set; old Spanish dirs not in the set so re-classified once on next run (decision B).
4. **`XDG_DOWNLOAD_DIR` parsing** — strip surrounding quotes and expand `$HOME`; covered by Test #19.
5. **`log_file` collision with auto-detected Downloads** — guard added comparing canonicalized source roots against canonical log path; covered by an additional test.
6. **`launchd`/`systemd` behavior change** — launchers that previously no-op'd will start organizing on next tick. Documented explicitly in README + PR description.
7. **First-run edge case** — `load_config` on a missing default path now returns `Config::default()`. Existing CLI test of explicit missing path still errors (Test #24).
8. **Dry-run for directory moves** — mirror files: log `Se movería` line, no `fs::create_dir_all`, no `fs::rename`.
9. **Recursive scan safety with `Other/<name>/`** — `is_generated_category` filters `Other` at depth ≥ 1; inside-tree items are not re-scanned.
10. **Empty dir / symlink dir skips** — covered by Tests #15–#17.

## sdd-propose prompt (delegated)

This file is the orchestrator-authored prompt that will be handed to `sdd-propose`. The sub-agent should:

- Write its formal `proposal.md` to `openspec/changes/configurable-categories-and-sources/proposal.md`.
- Use the decisions table above as authoritative (do not re-open them).
- Reference this intent file in the proposal frontmatter.
- Output the standard envelope: status, executive_summary, artifacts (path of `proposal.md`), next_recommended, risks.

## Next phase

`sdd-spec` — write delta specs with Given/When/Then scenarios and RFC 2119 keywords, sourcing from the test coverage plan above.
