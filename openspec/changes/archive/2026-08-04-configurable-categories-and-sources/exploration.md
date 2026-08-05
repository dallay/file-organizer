## Exploration: configurable-categories-and-sources

### Current State
The repository is a single Rust 2021 Cargo package with `src/lib.rs` (~544 lines) holding all reusable logic — `Config`, `default_categories()`, `category_for()`, `is_generated_category()`, `collect_files()`, `move_file()`, `Lock`, `Logger` — and `src/main.rs` (~83 lines) holding only Clap parsing. Tests are inline `mod tests` in `src/lib.rs` (`src/lib.rs:483-544`); there are four tests today (`tests::classifies_extensions_case_insensitively`, `tests::custom_extension_overrides_defaults`, `tests::unique_destination_preserves_extension`, `tests::run_moves_files_and_leaves_generated_categories_out_of_scan`). `default_categories()` (`src/lib.rs:325-378`) returns a flat `HashMap<String, String>` built with a helper `add()`. `category_for()` (`src/lib.rs:302-323`) checks `config.extensions` first, then `default_categories()`, then falls back to `DEFAULT_CATEGORY = "Otros"` (`src/lib.rs:11`). `NO_EXTENSION_CATEGORY = "Sin extensión"` (`src/lib.rs:12`). `is_generated_category()` (`src/lib.rs:195-206`) excludes generated dirs from scans; it composes built-ins + `config.extensions` values + `DEFAULT_CATEGORY` + `NO_EXTENSION_CATEGORY` and checks path equality/prefix. `is_hidden()` (`src/lib.rs:208-212`) only checks the leading dot. `Lock::acquire()` (`src/lib.rs:419-434`) uses `fs::create_dir` (atomic mkdir). `default_lock_path()` (`src/lib.rs:404-413`) is `~/.cache/file-organizer.lock` (Unix) / `%LOCALAPPDATA%/file-organizer.lock` (Windows). Dry-run skips both move and lock acquisition but still constructs `Logger` (`src/lib.rs:112-117`). `move_file()` uses `fs::rename` and intentionally does not fall back to copy/delete on cross-volume (`src/lib.rs:269-275`). `Config::default().source_directories` is `Vec::new()` (`src/lib.rs:39`); `main.rs:65` already bails with "no hay carpetas configuradas ni indicadas en la línea de comandos" when empty, so auto-detection must populate before that check. `validate_config()` (`src/lib.rs:92-109`) rejects absolute paths and `..` parts inside extension categories; new flat single-segment names are safe under it. Platform launchers (`platform/macos/com.file-organizer.plist.example`, `platform/linux/file-organizer.{service,timer}`) only invoke `run` with no args, relying on default config path lookup described in `src/lib.rs:57-75`.

### Affected Areas
- `src/lib.rs` — `default_categories()` (`src/lib.rs:325-378`), `category_for()` (`src/lib.rs:302-323`), `is_generated_category()` (`src/lib.rs:195-206`), `Config` (`src/lib.rs:23-48`), `validate_config()` (`src/lib.rs:92-109`), `tests` module (`src/lib.rs:483-544`).
- `src/main.rs:65` — the empty-source bail must change to trigger auto-detection first (or `lib.rs` must populate defaults during `load_config`).
- `src/lib.rs::run` (`src/lib.rs:111-156`) and `collect_files` (`src/lib.rs:158-176`) — must add a directory-move loop alongside `move_file`.
- `openspec/specs/` — currently empty; new domain spec for file organization is needed per `openspec/config.yaml`.
- `config.example.toml` — comment `tex = "Documentos/LaTeX"` (`config.example.toml:14-15`) is now obsolete; example should demonstrate `[[categories]]`.
- `README.md` — the "Reglas integradas" section (`README.md:40-42`) and config example describe Spanish categories and will become stale.
- `AGENTS.md` line 22 ("--dry-run skips moves and locking but still opens/writes the configured log unless --log /dev/null") remains true; line 27 ("Extension overrides are case-insensitive and replace built-in mappings") remains true; line 28 (lock semantics) remains true.
- `platform/macos/com.file-organizer.plist.example`, `platform/linux/file-organizer.{service,timer}` — do NOT need changes; they only invoke `run` and rely on default config lookup.
- `openspec/config.yaml` — no changes expected.

### Approaches

1. **Flat modules, no new files (`src/lib.rs` absorbs everything)** — replace `default_categories()` with the 7 flat maps, add `Config.categories` (`Vec<CategoryRule>`) with `replace: bool`, run resolution at top of `run()` (or end of `load_config`) for downloads, add a directory-handling loop in `collect_files` / a sibling `collect_top_dirs`. Implementation is straightforward but bloats `src/lib.rs` past 700 lines and mixes domain parsing with config plumbing.
   - Pros: minimal file churn; no new module boundary to argue about; preserves the current "everything in `lib.rs`" AGENTS.md invariant (`AGENTS.md:6`).
   - Cons: `src/lib.rs` becomes the second-largest file in the repo with multiple unrelated concerns; harder to test download resolution in isolation; harder to unit-test `replace` semantics without spinning up full `run()`.
   - Effort: Low

2. **Two new modules (`src/categories.rs`, `src/downloads.rs`) + slim `lib.rs`** — extract category resolution (built-ins, user `[[categories]]`, `extensions` last-write override, `is_generated_category` set composition) into `src/categories.rs`; extract XDG / Windows-localized / env-var auto-detection into `src/downloads.rs`. `lib.rs` keeps `Config`, `run`, `move_file`, lock, logger, dir-move helper. Tests for each module live next to the code.
   - Pros: clean module boundaries; per-module tests; download logic can be exercised without touching the filesystem loop; honors the existing test pattern (inline `mod tests` per file).
   - Cons: introduces two new files; requires `mod` declarations in `lib.rs` and one extra module to document; slightly more ceremony for callers.
   - Effort: Medium

3. **One new module (`src/categories.rs`) only, downloads inline** — categories get their own module (because serde + `replace` semantics + `is_generated_category` recomposition are the most logic-heavy piece), but the ~30-line platform-detect helper stays in `lib.rs` near `default_lock_path()`.
   - Pros: keeps platform-specific path resolution next to existing path helpers; smaller diff than Approach 2; downloads logic is genuinely tiny.
   - Cons: download logic still mixed with `lib.rs` orchestration; harder to test without env-var injection if it grows later.
   - Effort: Low/Medium

### Recommendation
Use **Approach 3**: extract `src/categories.rs` for category resolution and generated-set composition, keep downloads inline as a small helper next to `default_lock_path()` in `src/lib.rs`. Justification: AGENTS.md line 6 explicitly says keep "config validation, classification, traversal, locking, logging, and file operations reusable in `src/lib.rs`" — pulling downloads out is unnecessary (it's a config-resolution helper, ~30 lines, no independent test surface beyond env-var paths). Category resolution, by contrast, is growing into something with multiple inputs (built-ins + `[[categories]]` supplement/replace + `extensions` last-write) and is the most fragile piece — it earns its own file. AGENTS.md does not forbid new modules; only warns against moving CLI parsing out of `main.rs` (which we are not doing). The companion `mod tests` pattern fits the existing inline-test convention.

If the implementation discovers that downloads needs three or more `cfg!(target_os)` arms with non-trivial Windows-localized fallback logic, escalate to Approach 2.

### Risks (concrete, grounded)

1. **Coverage regression in `Text`**: the proposed list is missing `xls` (current `Documentos/Hojas de cálculo`, `src/lib.rs:343`) and `ppt` (current `Documentos/Presentaciones`, `src/lib.rs:347`). Both are listed in current built-ins and will silently fall through to `Other` after this change. Either add them to `Text` or explicitly document the demotion.

2. **All current `Código` extensions become `Other`**: `src/lib.rs:371-376` maps `java, kt, kts, js, ts, jsx, tsx, py, rb, go, rs, sh, zsh, json, yaml, yml, xml, html, css, sql`. None appear in the proposed 7 categories. Every existing user with `main.rs`, `index.js`, `Cargo.toml`, etc. in their Downloads will see those files land in `Other/`. This is the largest user-visible behavior change; the proposal must call it out explicitly (it is in "Other — catchall for unknown extensions", but the user already approved that wording).

3. **`is_generated_category` name collisions**: current implementation checks the **value** strings of `default_categories()` + `extensions` + the two constants (`src/lib.rs:195-206`). After this change, the new flat names (`Image`, `Video`, `Audio`, `Text`, `Other`, `Executable`, `Compressed`) replace Spanish names. A user with an existing top-level folder named e.g. `Image` will have it filtered from the scan (treated as a generated category) — that may be the intended outcome, but it must be tested. Conversely, if a user has `Other/` already with content, files in it will be skipped.

4. **Top-level directory → `Other/<dirname>/` collision with `is_generated_category`**: the new behavior needs a separate code path from `move_file` because the current `is_generated_category` check (`src/lib.rs:185-187`) would prevent entry into a top-level `Other/` if it ever became a generated-category path. Specifically: when scanning the root, the dir-move pass must NOT use `should_visit`'s generated-category filter for the depth-1 dir entry itself — it must look at all depth-1 dirs, classify by name (skip generated, skip empty, skip symlinks, respect `ignore_hidden`), then move the chosen ones. The recursive exclusion must then also exclude `Other/` so we never recurse into it.

5. **Dry-run on `Other/<dirname>/` moves**: `move_file` (`src/lib.rs:255-263`) only writes the "Se movería" log and returns; no directory needs to be created. For directories, dry-run behavior must mirror this exactly — log the planned move but do not `fs::create_dir_all`. Test must cover this.

6. **Cross-volume `rename` for directories**: same atomic-on-same-volume caveat as files (`src/lib.rs:269-275`, `AGENTS.md:29`). On macOS, APFS snapshots mean intra-volume renames are usually safe; on Windows, moving a directory across drives (e.g., user's Downloads is on D:, but a stray `Other/<name>` might span mountpoints) returns `ERROR_NOT_SAME_DEVICE`. The same error-message convention must apply.

7. **`on_conflict` policy on directories**: `move_file::Overwrite` checks `if requested_destination.is_dir()` (`src/lib.rs:245-251`) and skips; the dir path needs its own semantics — likely "skip if exists, never overwrite directories".

8. **`XDG_DOWNLOAD_DIR` parsing**: `~/.config/user-dirs.dirs` (when present) is a shell-sourceable file. Lines look like `XDG_DOWNLOAD_DIR="$HOME/Downloads"` — requires handling comments (`#`), stripping surrounding double quotes, and expanding `$HOME`. A naive line-grep will produce `"$HOME/Downloads"` literally and break the path.

9. **Windows-localized folder fallback**: the spec lists `Descargas`, `Téléchargements`, `Scaricati`, `下载`. Implementation must use `cfg!(windows)` and walk them in order, picking the first that exists. The current code uses `cfg!(windows)` only twice (`src/lib.rs:62, 405`) — no other OS-gated branching exists; this introduces a third and the first inside `run()`/`load_config()`.

10. **Auto-detection runs against an unwritable path silently**: e.g., headless server where `$HOME/Downloads` does not exist. Currently `main.rs:65` bails; if we auto-detect and the resolved path doesn't exist, the existing `is_dir` branch (`src/lib.rs:122-126`) already logs and counts it as a failure — that's acceptable, but the failure count then includes "auto-detection produced a non-existent path", which is a different category of error than "user misconfigured a path". Worth surfacing in the log line.

11. **`expand_home` on auto-detected paths**: the current `expand_home` (`src/lib.rs:386-395`) is called on config-loaded paths but not on env-var-derived ones. If `FILE_ORGANIZER_DOWNLOADS=~/Downloads` is set, the helper must be invoked.

12. **`load_config` ordering**: `validate_config` is called inside `load_config` (`src/lib.rs:88`) and validates non-empty source directories. If we auto-detect inside `load_config`, we need to do it before `validate_config` — OR we move auto-detection to `main.rs` and let `run()` consume a populated `Config`. The latter is cleaner because `load_config` should remain a pure TOML parser.

13. **Spanish vs English category names in user folders**: a user with an existing `~/Downloads/Imágenes/` and `~/Downloads/Documentos/` from prior runs will have those folders co-exist with new `Image/`, `Text/`. New categories are not seen as generated by the old names (they're not in `default_categories()` anymore), so the scan would walk INTO `Imágenes/` and find files there, then try to move them again to `Image/`. That could create `Image/foto.JPG` and `Image/foto (1).JPG` etc. The implementation must include a one-time migration or at minimum preserve the old folder names in `is_generated_category` (treat old names as generated too).

14. **Default `log_file` and lock path interaction with auto-detected Downloads**: `~/.cache/file-organizer.lock` is independent of any Downloads path; safe. `log_file` defaults to None; safe. If a user later configures `log_file = "~/Downloads/log.txt"`, the auto-detection pass must not move the log file — but with a `.txt` extension it absolutely would (Text category). The implementation must exclude the log file path from being moved (e.g., compare to `config.log_file` resolved path before moving). This is a NEW invariant not present today.

15. **Behavior change for launchd/systemd jobs**: the plist/service files only invoke `run` with no positional args. After this change, those jobs will start auto-detecting Downloads and processing it. Users who previously relied on the empty-defaults bail (`main.rs:65`) will see files actually moved. This is a documented behavior change but easy to overlook in PR review.

### Ready for Proposal
Yes. The orchestrator can start `sdd-propose` for `configurable-categories-and-sources`. The proposal must call out:
- Coverage gaps: missing `xls`, `ppt` in `Text`; entire `Código` set demoted to `Other`.
- Module decision: Approach 3 (new `src/categories.rs`, downloads inline).
- Where to invoke auto-detection: `main.rs` after `load_config` returns, before the empty-source bail.
- Backward-compat: keep old Spanish names in `is_generated_category` so prior organized folders are still excluded from scan.
- Behavior change for platform schedulers (launchd/systemd jobs that previously no-op'd will start running).
- Lock/log collision guard: exclude `config.log_file` from auto-classified moves.

## Detailed answers to orchestrator questions

### Q1. Coverage regression: does the proposed extension list cover every current extension?
**No.** Three gaps:
- `xls` — currently `Documentos/Hojas de cálculo` (`src/lib.rs:343`), not in proposed `Text`.
- `ppt` — currently `Documentos/Presentaciones` (`src/lib.rs:347`), not in proposed `Text`.
- Entire current `Código` set (`src/lib.rs:371-376`): `java, kt, kts, js, ts, jsx, tsx, py, rb, go, rs, sh, zsh, json, yaml, yml, xml, html, css, sql` — not in any proposed category. They will all map to `Other`.

### Q2. Module placement
- `[[categories]]` parsing → new `src/categories.rs` (Approach 3). Justified because the resolution logic spans built-ins + user rules + `replace` semantics + `extensions` last-write + `is_generated_category` recomposition, and benefits from isolated tests. `default_categories()` and `category_for()` and `is_generated_category` move with it; `Config` stays in `lib.rs` and gains a `categories: Vec<CategoryRule>` field.
- Download auto-detection → keep in `src/lib.rs` next to `default_lock_path()`. Justified because it's ~30 lines, no new third-party deps, and the platform branches already live there (`cfg!(windows)` at `src/lib.rs:62, 405`). If Windows-localized fallback grows past ~10 lines per locale, escalate to a separate `src/downloads.rs`.

### Q3. Cross-cutting constraints (file:line references)
- Lock semantics — `Lock::acquire` uses `fs::create_dir` (atomic mkdir), `AlreadyExists` → bail with "ya hay otra ejecución en curso" (`src/lib.rs:419-434`); `default_lock_path()` Unix=`$HOME/.cache/file-organizer.lock`, Windows=`%LOCALAPPDATA%/file-organizer.lock` (`src/lib.rs:404-413`); dry-run skips lock entirely (`src/lib.rs:112-116`).
- `rename` cross-volume — `fs::rename` only; no copy/delete fallback; error context includes source and destination (`src/lib.rs:269-275`); README documents this (`README.md:86`); AGENTS.md line 29 mandates it.
- `is_generated_category` filtering — `src/lib.rs:195-206` composes built-ins + `extensions.values()` + `DEFAULT_CATEGORY` + `NO_EXTENSION_CATEGORY` and checks `path == root.join(category)` or `path.starts_with(...)`. Must be updated for new flat names; old Spanish names should be retained for backward compatibility (see Risk 13).
- Hidden-file handling — `is_hidden()` only checks `name.starts_with('.')` (`src/lib.rs:208-212`); `should_visit` (`src/lib.rs:178-189`) and `should_process_file` (`src/lib.rs:191-193`) gate on it. Top-level dirs starting with `.` are already filtered at depth >=1 by `should_visit` if `config.ignore_hidden` is true (default).
- Dry-run — skips moves and locking but still constructs `Logger` and writes configured log (`src/lib.rs:112-117`, AGENTS.md:22). `--log /dev/null` or `--log NUL` short-circuits via `is_null_device` (`src/lib.rs:470-472`).
- `expand_home` — applies `~/` to source paths and log file in `load_config` (`src/lib.rs:82-87`). Auto-detected env-var values must also be expanded.
- `validate_config` — rejects empty paths, empty keys/categories, absolute category paths, `..` segments (`src/lib.rs:92-109`). New flat single-segment names are safe.
- `main.rs:62-67` — positional dirs override config; empty after override bails. Auto-detection must run before this bail.

### Q4. Existing tests that reference Spanish category names
- `tests::classifies_extensions_case_insensitively` (`src/lib.rs:497-503`): asserts `category_for("PHOTO.JPG") == "Imágenes"`, `category_for("README") == "Sin extensión"`, `category_for("data.custom") == "Otros"`. Must rewrite to expect new flat names (`"Image"`, `"Other"` for unknown ext, `"Other"` or equivalent for no-extension — note: the spec removes the `Sin extensión` distinction, no-extension files now go to `Other`).
- `tests::run_moves_files_and_leaves_generated_categories_out_of_scan` (`src/lib.rs:523-543`): creates `Imágenes/` at root and asserts `Imágenes/photo.JPG` and `Imágenes/already.JPG`. Must rewrite to create the equivalent new-category dir (e.g., `Image/`) and assert against that. Also exercises the `is_generated_category` exclusion path, which is exactly the behavior that needs verification post-refactor.
- `tests::custom_extension_overrides_defaults` (`src/lib.rs:505-510`) and `tests::unique_destination_preserves_extension` (`src/lib.rs:512-521`): do NOT reference Spanish names. The first will need a sanity check that `extensions` still overrides the new flat categories (it will, but the test should be confirmed). The second is orthogonal.
- Test helper `test_config` (`src/lib.rs:488-495`): no Spanish names; still valid.

### Q5. Platform launchers — does this change break them?
**No**, with one caveat.
- `platform/macos/com.file-organizer.plist.example` (`platform/macos/com.file-organizer.plist.example:8-11`): hardcodes the binary path and `run` subcommand. No `--config`, no positional dirs. After this change, the job will now auto-detect Downloads instead of bailing. **Behavior change, not breakage.** Document it in the PR.
- `platform/linux/file-organizer.service` (`platform/linux/file-organizer.service:5`): `ExecStart=%h/.local/bin/file-organizer run`. Same behavior change as above.
- `platform/linux/file-organizer.timer` (`platform/linux/file-organizer.timer:4-7`): 5-minute interval; unchanged.
- README.md Windows `schtasks` example (`README.md:79-82`): no Downloads reference. Unchanged.
- No paths or default config references in any launcher file that this change touches.

### Q6. CI / coverage / test inventory
- **No `tests/` directory**, no integration tests, no `benches/`. Confirmed by `ls /Users/acosta/Dev/file-organizer/tests /Users/acosta/Dev/file-organizer/benches` returning empty.
- **No `.github/workflows/`**, no CI. Confirmed by `ls /Users/acosta/Dev/file-organizer/.github` returning empty.
- **No coverage config**: `Cargo.toml` lists only `anyhow, clap, serde, toml, walkdir` and dev-dep `tempfile` (`Cargo.toml:8-16`); no `tarpaulin`, `grcov`, or codecov config.
- **What IS present**: local gate from `AGENTS.md:14` and `openspec/config.yaml:11-19`:
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
- **Test runner pattern**: `cargo test tests::<test_name> -- --exact` (e.g. `cargo test tests::classifies_extensions_case_insensitively -- --exact`, AGENTS.md:13).
- **Strict TDD**: `openspec/config.yaml:9, 36-38` mandates a failing test before production code; `apply.workflow` must use the same test command.

### Q7. Behavior risks (additional to those in the Risks section)
- **Top-level dir with the same name as a generated category** (e.g., user has `Image/` or `Other/` already at root): currently filtered by `is_generated_category` (`src/lib.rs:185-187, 195-206`). After this change, the new flat names take their place. A pre-existing `Image/` is silently treated as a generated category and skipped. Test must cover: top-level `Image/` exists, run() does not enter it, files inside are not moved.
- **`Other/` as a generated category on the source root**: same — pre-existing `Other/` is filtered. New top-level dirs would be moved TO `Other/<name>/`, but the dir-move code path must avoid recursing into `Other/` itself.
- **`~/Downloads` colliding with `log_file` or lock path**: lock is `~/.cache/file-organizer.lock` (`src/lib.rs:411`) — outside Downloads, safe. `log_file` default is `None`. If user configures `log_file = "~/Downloads/log.txt"`, dry-run still creates the file (`src/lib.rs:454` `OpenOptions::new().create(true)`) and the run pass will classify `log.txt` by `.txt` extension → `Text/`. **Add a guard**: skip the path if it equals the resolved `config.log_file`.
- **Recursive scan with `Other/<name>/`**: once we move a top-level dir into `Other/<name>/`, that path is now `root/Other/<name>/...`. `is_generated_category` filters `Other` at the top level so we never enter `Other/` recursively — the new tree inside is safe by construction. But the dir-move code must NOT move dirs into `Other/` if `Other/` is itself a generated category at the root — it must create `Other/` only on demand and skip if it already exists as a non-generated-category folder (counterintuitive but consistent with `is_generated_category` semantics).
- **Empty dirs at depth 1**: spec says skip; `entry.file_type().is_dir()` plus `WalkDir` will list them; we must check `fs::read_dir` length or use `entry.metadata().len()` (won't work for dirs) or use `WalkDir::IntoIter` to enumerate children — easiest is to call `fs::read_dir(entry.path())` and check `count() == 0`.
- **Symlinked top-level dirs**: spec says skip; `entry.file_type().is_symlink()` (or `metadata().is_symlink()` if `follow_links=false`, which is the walkdir default — `WalkDir::new(root)` without `follow_links(true)` does NOT follow symlinks and `file_type().is_symlink()` will be true for them).
- **First-run edge case**: on a fresh machine with no `~/.config/file-organizer/config.toml`, `load_config` returns `Err` (`src/lib.rs:78-79`). Auto-detection cannot trigger because the config can't be loaded. This is pre-existing behavior; the proposal may need to add a fallback for missing config (e.g., `Config::default()` + auto-detect if `load_config` fails AND no `--config` was supplied). This is a separate decision the user must make.