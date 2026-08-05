---
change: configurable-categories-and-sources
schema: spec-driven
date: 2026-08-04
status: proposed
summary: Make categories user-configurable and auto-detect Downloads without weakening explicit CLI or extension overrides.
source: intent.md
---

# Proposal: Configurable Categories and Sources

## Why

`src/lib.rs:325-378` hard-codes 13 Spanish, nested defaults; `src/lib.rs:302-323` cannot add categories without a Rust edit. `Config::default()` leaves sources empty (`src/lib.rs:36-47`), and `src/main.rs:60-67` requires TOML or positional directories. The result is poor first-run behavior and an unnecessarily localized default experience.

## What changes

- Replace defaults with flat `Text`, `Other`, `Executable`, `Compressed`, `Audio`, `Video`, `Image`; include locked `Text` extensions `xls` and `ppt`; map code files and unknown/no-extension files to `Other`.
- Add `[[categories]]`: supplement by default, `replace = true` substitutes; apply `[extensions]` last (locked decisions).
- Resolve sources as locked: `FILE_ORGANIZER_DOWNLOADS` → Linux `XDG_DOWNLOAD_DIR` → macOS `~/Downloads` → Windows `%USERPROFILE%/Downloads` and localized fallbacks. Missing default config uses `Config::default()`; explicit missing `--config` still errors. Config sources beat detection; CLI positional sources beat config.
- Move eligible depth-1 directories to `Other/<dirname>/`; skip generated, empty, symlinked, or hidden directories. Preserve `recursive: bool`; do not add `max_depth`.
- Recognize only the seven new names as generated categories, so legacy Spanish directories are re-scanned/reclassified once (locked decision).

## Affected areas

The following is quoted from `intent.md`’s **In scope** section:

- `src/lib.rs` (split: extract `src/categories.rs`; keep auto-detect + lock + logger + run + main entry helpers here).
- `src/categories.rs` (new module: `default_categories`, `category_for`, `is_generated_category`, `CategoryRule` deserialization).
- `src/main.rs` (wire Config::default() fallback for missing default path; auto-detect for empty `source_directories`).
- `config.example.toml` (demo `[[categories]]`, drop obsolete Spanish comment).
- `README.md` (replace "Reglas integradas" Spanish list with the 7 flat categories; document `[[categories]]`; document Downloads auto-detect; document one-time re-classification of legacy Spanish dirs).
- Inline tests in `src/lib.rs::tests` (rewrite Spanish-name assertions; add new tests per spec).

## Out of scope

Quoted verbatim from `intent.md`:

- Configurable maximum depth (`max_depth`) — separate future change.
- New runtime dependencies. No new crates in `Cargo.toml`.
- Wider i18n (`default_locale`, message translations) — not asked for.
- Telemetry, hooks, IPC, plugin system — not asked for.
- Changes to install/launcher layout in `platform/` — unchanged; behavior change (auto-detect) is documented in README.

## Approach

Use Approach 3 from exploration: extract classification, rule deserialization, resolution, and generated-set composition into `src/categories.rs`; keep the small platform-aware Downloads helper beside `default_lock_path()` in `src/lib.rs`. Runtime resolution is config rules in order, built-in fallback, then `[extensions]` last-write override. Auto-detection runs in `main.rs` after `load_config` and before the existing empty-source bail. Directory moves mirror file conflict handling and dry-run logging (`src/lib.rs:111-156`), with no directory creation or rename during dry-run. Add no crates.

## Test coverage plan

Each snippet is a strict-TDD scenario; write it failing before production code.

| # | Given | When | Then |
|---:|---|---|---|
| 1 | JPG, TXT, unknown, and no-extension files | classify case-insensitively | results include `Image`, `Text`, and `Other` |
| 2 | root `Image/` plus a root photo | run | photo moves; existing categorized files stay |
| 3 | `pdf -> Review` in `[extensions]` | classify PDF | `Review` wins |
| 4 | unknown extension | classify | returns `Other` |
| 5 | every built-in extension | classify each | every mapping remains covered |
| 6 | supplemental category rule | resolve | built-ins remain and new extensions map |
| 7 | `replace = true` rule | resolve category | built-in list is substituted |
| 8 | non-colliding category name | resolve | category is added; others unchanged |
| 9 | category rule plus extension override | resolve | `[extensions]` wins last |
| 10 | category name absolute or containing `..` | validate | validation fails |
| 11 | category with empty extensions | validate | validation fails |
| 12 | non-generated non-empty top-level directory | run | moves to `Other/<dirname>/` |
| 13 | generated top-level directory | run | directory is not moved |
| 14 | generated `Other/` with contents | recursive run | `Other/` is not re-entered |
| 15 | empty top-level directory | run | directory is skipped |
| 16 | symlinked top-level directory | run | directory is skipped |
| 17 | hidden top-level directory and `ignore_hidden` | run | directory is skipped |
| 18 | `FILE_ORGANIZER_DOWNLOADS` | detect sources | its path is selected |
| 19 | mocked `user-dirs.dirs` with `XDG_DOWNLOAD_DIR` | detect on Linux | quoted `$HOME` path is expanded and selected |
| 20 | macOS environment/home | detect | `~/Downloads` is returned |
| 21 | Windows profile Downloads exists | detect | `%USERPROFILE%/Downloads` is selected first |
| 22 | Windows primary missing, localized fallback exists | detect | first localized fallback is selected |
| 23 | missing default config, no `--config` | run | `Config::default()` plus detection is used |
| 24 | missing explicit `--config` path | run/validate | command errors |
| 25 | configured source directories plus env detection | run | configured sources win |
| 26 | positional directories plus config/detection | run | positional directories win |

## Risks

- **Coverage and demotion:** missing `xls`/`ppt` would regress to `Other`; the entire former `Código` set intentionally demotes to `Other` (`src/lib.rs:364-376`).
- **Legacy/collisions:** new-only generated names can reclassify legacy Spanish trees and skip user folders named like generated categories; duplicate destinations are possible.
- **Traversal/moves:** `Other/<dirname>` must not recurse, must skip empty/symlink/hidden dirs, respect conflicts, and preserve atomic same-volume `rename`/cross-volume errors (`src/lib.rs:269-275`).
- **Path resolution:** XDG quoting/`$HOME`, Windows localized fallbacks, and `~/` expansion are fragile; nonexistent auto-detected roots must remain visible failures.
- **Safety/operations:** exclude resolved `log_file` from classification; preserve dry-run/no-lock semantics (`src/lib.rs:112-117`); launchd/systemd jobs will begin processing Downloads instead of no-oping.

## Rollback plan

Quoted verbatim from `intent.md`:

The change is a single feature branch; reverting the merge commit (`git revert`) cleanly restores the old 13-category defaults, the old CLI behavior, and the old `load_config` failure mode. No data migration is required because files moved by the new version are still in their categorized subfolders — the user may see `Imágenes/` re-scanned once on the next run after revert, identical to the previous run-once-then-skip behavior.

A targeted rollback without revert is also possible: re-introduce the 13 Spanish names into `default_categories()` and zero out the `[categories]` parsing path. The auto-detection can be disabled per-call by setting `FILE_ORGANIZER_DOWNLOADS=` empty if undesired.

Additionally, revert `config.example.toml` and README changes with the code; no dependency, schema, or launcher migration is required.

## Open questions

None. Locked decisions in `intent.md` govern unresolved-looking choices, including legacy Spanish directory handling and directory conflict behavior.
