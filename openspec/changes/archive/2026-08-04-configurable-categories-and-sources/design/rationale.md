# Per-Decision Rationale

## 1. Precompose at `load_config`, not per-lookup

`category_for` runs once per file inside `collect_files`/`move_file`
(`src/lib.rs:158-176`). Per-rule `replace = true` semantics that mutated a
per-call map would re-walk the rules list on every file — a quadratic per
run. Precomposing once in `load_config` via `categories::apply_categories`
gives an O(1) hash lookup at runtime and keeps `replace` semantics where
they belong: load-time structure, not per-call mutation. The composed map
is also what `is_generated_category_set` reads, so the directory guard at
`src/lib.rs:185-187` sees the same source of truth as the file classifier.
Tests `test_6`/`test_7`/`test_8` exercise the precomposed state directly.

## 2. Why ONLY seven flat English names in `is_generated_category_set` (locked decision B)

Pro (re-classify once): legacy Spanish folders (`Imágenes/`, `Documentos/`,
`Vídeos`, `Comprimidos`, `Instaladores`, `Código`, `Audio`) lose their
"generated" status on next run, so they are re-entered, re-scanned, and
re-categorized once. After that pass, files live under the new English-named
categories and the legacy dirs can be removed manually.
Con (one-shot user event): users who built a workflow around the Spanish
names see one reclassification. Decision B in `intent.md:17` (owner-approved)
accepts this over a permanent Spanish alias set that would need maintenance
every time a category name changes. No automatic rename moves the legacy
directories; `is_generated_category` is a scan-time filter, not a rewriter.

## 3. Why `[[categories]]` array-of-tables, not `[categories.<name>]`

A `[categories]` table keyed by name can hold ONE rule body per name — TOML
tables cannot repeat keys. A user who wants `Text` with `replace = true`
AND a later supplemental rule needs TWO blocks. Array-of-tables preserves
declaration order AND per-rule `replace`, matching the locked decision in
`intent.md:14` and the scenarios in `category-configuration/spec.md:18-58`.
`CategoryRule` uses `#[serde(deny_unknown_fields)]` so `extentions` (typo)
errors at parse time rather than at first move. Cost: slightly heavier
deserialization than `HashMap<String, Vec<String>>`. Benefit: round-trips
through serde cleanly and matches TOML idiom (see `Cargo.toml:12` — `toml =
"0.8"` supports `[T]` and `[[T]]` cleanly).

## 4. Why path injection for the XDG test, NOT env-var override

`HOME` env mutation in a Cargo test serializes with other env-mutating tests
and breaks under any future `cargo test` parallelism flag (`--test-threads`,
`nextest`). Adding `home_override: Option<&Path>` to `default_downloads_path`
decouples test fixtures from process environment and preserves the
production public surface (`None`). Bonus: Windows and macOS arms become
testable from any host — test invocations can pass synthetic HOME paths
and assert the chosen arm's output (`downloads-autodetect/spec.md:50, 66, 75`).
The Linux test (test_19) does NOT set `HOME` at all; it points
`home_override` at a temp directory containing a synthetic
`user-dirs.dirs`.

## 5. Why ONE canonical log-path resolution in `run`, not per-entry

`fs::canonicalize` walks the filesystem. Calling it on every candidate in
`collect_files` and `collect_top_level_directories` would re-stat the source
tree on every entry. Resolving ONCE in `run` (immediately after `Logger::new`)
gives O(1) per-entry membership test via a `HashSet<PathBuf>` lookup. This
also protects the auto-detected `~/.cache/file-organizer.lock` neighbor
(`src/lib.rs:404-413`) from being treated as a log collision — lock lives
outside Downloads, so canonical matching keeps the invariant local to the
configured `log_file` only.

## 6. Why interleave the dir-move loop with the file loop in `run`

`src/lib.rs:158-176` collects files into a `Vec`, then `move_file` runs
sequentially. Doing the same for dirs would mean two passes; interleaving
lets us share logger state and dry-run/recency/conflict gating without
duplicating the outer skeleton. Trade-off: test failures must trace through
the interleaving; the apply phase writes at least four tests covering the
sequence (test_13/test_14/test_15/test_16; test_12 exercises the happy path).
The interleaving is documented in `sequences.md` diagram (b).

## 7. Why `load_config` STAYS a pure parser

Auto-detection requires reading the runtime environment
(`FILE_ORGANIZER_DOWNLOADS`, `XDG_*`) and the user's `HOME`. Side effects
in a function named `load_config` violate the principle of least surprise
(`src/lib.rs:77-90`) and make tests that need to load without env-var
leakage harder to write. Pushing the missing-default branch into `main.rs`
keeps `load_config` isomorphic to TOML→struct and its behavior deterministic
per its inputs. The fallback at `src/main.rs:50` distinguishes "user said
nothing and the default is absent" (test_23: synthesize default) from "user
said `--config X` and X is absent" (test_24: surface error). The branching
requires `cli.config.clone()` once, but that cost is paid once per process
startup.

## 8. Why `Cli.config` is cloned before `unwrap_or_else`

`main.rs:50` currently does `cli.config.unwrap_or_else(default_config_path)`
and discards the `Some` branch's user intent. To distinguish "user said
`--config X` and X is missing" (errors — test_24) from "user said nothing
and the default doesn't exist" (falls through to `Config::default()` —
test_23), we need both the resolved path AND the original `Option`. A
single `cli.config.clone()` is cheaper than threading the `Option` through
every downstream call to `run`, and clone-once-at-startup is invisible.
