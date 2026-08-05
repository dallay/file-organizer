# Archive Notes — `configurable-categories-and-sources`

**Change**: configurable-categories-and-sources
**Archived**: 2026-08-04
**Verification verdict**: PASS WITH WARNINGS (0 critical, 0 blocking, 4 warnings)
**Local gate**: green (`cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` all clean)

This file captures the four non-blocking warnings from `verification.md`.
None blocks archive; each is recorded here so a future SDD cycle can
address it without re-deriving the context.

---

## W1 — `apply_categories` is recomputed per call instead of precomposed once

- **Severity**: WARN
- **Source**: `verification.md` §4 D1; `design/rationale.md` §1.
- **What**: `category_for` (`src/categories.rs:100-114`) recomputes
  `apply_categories(config)` on every call rather than precomposing the
  resolved map exactly once at `load_config` as the design rationale
  prescribes. Cost per call is O(built-ins + rules + extensions) ≈ O(50)
  with a small number of `HashMap` insertions; with M files, the total
  work is O(M × 50). On modern hardware this is microseconds and
  correctness is unaffected.
- **Verification**: all 26 spec scenarios + log-collision guard pass.
- **Mitigation**: future-change hint — address in a follow-up SDD
  cycle. Either precompose at `load_config`/`resolve_config` and pass
  the composed map through, or memoize the first call inside a
  `OnceCell` keyed by `Config`. Behavior MUST stay identical;
  benchmarks SHOULD show the precomposed path is faster but the user
  must not see a behavior change.

---

## W2 — Test names are descriptive instead of `tests::test_<N>` (positive deviation)

- **Severity**: WARN (positive)
- **Source**: `verification.md` §4 D2.
- **What**: the delta specs at
  `specs/*/spec.md` reference tests as `tests::test_<n>`
  (e.g., `tests::test_6`). The apply phase translated each placeholder
  into a descriptive name (e.g., `tests::supplemental_category_rule_adds_extensions`).
  Every spec scenario has a 1-to-1 covering test with a working
  `--exact` runner; no coverage gap.
- **Verification**: every `cargo test tests::<name> -- --exact` listed
  in `verification.md` §3 passes.
- **Mitigation**: future-change hint — the canonical specs in
  `openspec/specs/*/spec.md` record both the stable scenario id
  (`test_<n>`) AND the actual descriptive test name with the runner
  command. Future specs SHOULD continue using descriptive names; the
  `test_<n>` ids remain as stable cross-document references.

---

## W3 — `tests::classifies_extensions_case_insensitively` lacks an explicit `.TXT` → `Text` assertion

- **Severity**: WARN
- **Source**: `verification.md` §4 D3; `classification/spec.md:25-35`.
- **What**: proposal test_1 row asserts that the resolver produces
  `Image`, `Text`, and `Other` from `JPG, TXT, unknown, no-ext`. The
  rewritten test asserts `JPG → Image`, `no-ext → Other`, and
  `unknown → Other`. The `Text` mapping is verified by the
  `tests::every_builtin_extension_maps_to_nonempty_category` companion
  test (test_5) which iterates every key in `default_categories()`
  including `txt`. Behavior is correct, but the test could be tightened
  with an explicit `assert_eq!(category_for(Path::new("NOTES.TXT"),
  &config), "Text")`.
- **Verification**: behavior correct via test_5 (every built-in maps to
  a non-empty, non-Other category). Test passes on the host that ran it.
- **Mitigation**: future-change hint — address in the next SDD cycle
  that touches `src/categories.rs::category_for` or
  `src/lib.rs::tests::classifies_extensions_case_insensitively`. Add
  the explicit `.TXT` assertion and re-run the exact runner.

---

## W4 — `README.md` is not in the apply-phase diff (provenance only)

- **Severity**: WARN
- **Source**: `verification.md` §4 D4; `verification.md` §7 (docs
  cross-check).
- **What**: `git diff HEAD` for the apply phase shows no change to
  `README.md`. The current README content already contains the seven
  flat English categories table, the `[[categories]]` syntax block,
  the Downloads auto-detect order, the one-time legacy Spanish
  reclassification callout, and the scheduler-behavior note. Those
  sections were committed in `2bc2d15` (the prior CI-tooling commit)
  and remained in the working tree. Final README state matches
  `intent.md` decisions and the locked `config.example.toml` content.
- **Verification**: documentation cross-check at `verification.md` §7
  passes for every locked decision in `intent.md`.
- **Mitigation**: future-change hint — none required. Document the
  provenance in the PR description so reviewers can trace the README
  to `2bc2d15` and confirm the working-tree state matches the spec.
  If a future change touches the README, keep the same seven-category
  table and callouts; do not reintroduce Spanish defaults or the
  obsolete `Reglas integradas` heading.

---

## Aggregate verification cross-reference

- Local gate (`cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`): green.
- Test results: 35 / 35 pass (26 spec scenarios + log-collision guard + 8 module-internal `categories::tests` + 1 pre-existing helper test).
- OS-gated tests not run on the macOS host:
  - `tests::default_downloads_path_reads_xdg_user_dirs` (`cfg(target_os = "linux")`) — code-reviewed, expected to pass on Linux CI.
  - `tests::default_downloads_path_selects_userprofile_downloads_first` and `tests::default_downloads_path_uses_localized_fallback` (`cfg(target_os = "windows")`) — code-reviewed, expected to pass on Windows CI.
- Suggestion-level deviations D5 and D6 in `verification.md` §4 are not warnings; they are notes about spec wording and example-toml commenting. Captured here for completeness:
  - D5: `category-configuration/spec.md` references `resolve_supplement` / `resolve_replace` / `apply_extension_override` as if they were separate functions; the implementation uses one `apply_categories` function with branches. The canonical spec at `openspec/specs/category-configuration/spec.md` references `apply_categories` consistently.
  - D6: `config.example.toml:23-27` shows `replace = true` as a commented-out block rather than active TOML. The README shows the active form. Acceptable.
