# Tasks: Rebrand to organiza + Full Distribution Pipeline

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~700–850 total (PR1 ≈ 250–350; PR2 ≈ 400–500) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1: rebrand + metadata + docs. PR2: pipeline + npm + Docker |
| Delivery strategy | ask-on-risk |
| Chain strategy | github-stacked-prs |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: github-stacked-prs (resolved by user: PR1 base=main, PR2 base=PR1 branch)
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Rebrand + release hygiene | PR1 | base=trunk; build green + drift pass before any release tag |
| 2 | Distribution pipeline | PR2 | base=PR1 branch; additive only; release-please PR never merged |

## Phase 1: Infrastructure — rebrand + metadata (PR1)

- [x] 1.1 `Cargo.toml`: `name = "organiza"`; add repository, readme, keywords, categories, authors, exclude, description; `[profile.release]` (lto, codegen-units=1, strip=true). RDP-1 Rebranded crate.
- [x] 1.2 Create `LICENSE` (MIT, Yuniel Acosta). RDP-1 License present.
- [x] 1.3 `src/main.rs`: clap `name = "organiza"` (:8); `file_organizer::` → `organiza::` (:3, :58). RDP-1.
- [x] 1.4 `src/lib.rs`: `FILE_ORGANIZER_CONFIG`→`ORGANIZA_CONFIG` (:68, :1122); `FILE_ORGANIZER_DOWNLOADS`→`ORGANIZA_DOWNLOADS` (:512, tests :1029–1173); config dirs (:76, :82) and locks (:497, :499) → `organiza`; update test comments (:683, :688). RDP-1.
- [x] 1.5 `config.example.toml` header (:2–3) → `~/.config/organiza` / `%APPDATA%\organiza`. RDP-1.

## Phase 2: Implementation — launchers, docs, ignores (PR1)

- [x] 2.1 Rename launchers → `organiza run`: `platform/linux/file-organizer.{service,timer}` → `organiza.*` (ExecStart); `platform/macos/com.file-organizer.plist.example` → `com.organiza.plist.example` (ProgramArguments). RDP-1.
- [x] 2.2 `.gitignore`: add `npm/**/node_modules/`, `npm/organiza/lib/`, `npm/organiza/*.tgz` OUTSIDE AgentSync block (lines 5–17). Risk #10.
- [x] 2.3 `README.md`: brand to organiza (binary, config paths, launchers, scheduler cmds); remove "No Cargo or npm dependency" statement; add install channels + release jobs/secrets/rollback doc. CQV Documentation+rollback.
- [x] 2.4 `.agents/agentsync.toml`: organiza paths/artifacts; regenerate `.agents/AGENTS.md` via `agentsync apply`. Risk #3.

## Phase 3: Testing — PR1 gate

- [x] 3.1 Local gate: `cargo fmt -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `cargo metadata` reports `organiza`. RDP-1.
- [x] 3.2 Grep gate: `rg "file-organizer" -g '!openspec/**'` matches only allowed repo-URL/README refs; `agentsync status --json` clean. Risk #3.

## Phase 4: Infrastructure — npm wrapper (PR2)

- [x] 4.1 `npm/package.json.tmpl`: envsubst template (node_pkg/version/os/arch/bin), os+cpu fields, publishConfig public, bin.organiza. RDP-4.
- [x] 4.2 `npm/organiza/package.json`: base wrapper, bin→lib/index.js, 6 exact optionalDeps `@dallay/organiza-*@0.1.0`, engines node>=18. RDP-3/4.
- [x] 4.3 `npm/organiza/tsconfig.json`: compile src/index.ts → lib/index.js. RDP-3.
- [x] 4.4 `npm/organiza/src/index.ts`: PLATFORMS map (8 keys incl. cygwin), require.resolve + spawnSync, unsupported-platform error, reinstall message. RDP-3 scenarios.
- [x] 4.5 `npm/organiza/scripts/{sync-optional-deps,update-versions}.js`. RDP-2/4.

## Phase 5: Implementation — release automation (PR2)

- [x] 5.1 `release-please-config.json` (release-type rust, component organiza, extra-files jsonpaths: wrapper $.version + 6 optionalDeps, changelog sections) + `.release-please-manifest.json` `{ ".": "0.1.0" }`. RDP-2 Version sync.
- [x] 5.2 `.github/workflows/release.yml`: 8 jobs (release-please→build-binaries 8-target matrix→upload-assets, publish-npm-binaries 6 configs w/ envsubst+guards, publish-npm-base gated on result success, publish-crates, publish-docker, release-summary); permissions + id-token; concurrency; dry_run dispatch; actions full-SHA+comment; explicit tool versions. RDP-2/5/6/7/8; CD-4; CQV pinning+gated-publish.
- [x] 5.3 `Dockerfile`: rust:1-alpine builder (musl-dev, dummy src cache, `--locked`, strip) → alpine:3.23 runtime (tini, uid 1000, OCI labels, ENTRYPOINT tini -- organiza). CD-2/3.
- [x] 5.4 `.dockerignore`: target/, .git, npm/, openspec/, .github/, node_modules/. CD-4.

## Phase 6: Implementation — config context (PR2)

- [x] 6.1 `openspec/config.yaml` context: npm wrapper + TS in stack; testing gate adds `tsc --noEmit` for wrapper. CQV.

## Phase 7: Testing — PR2 gate

- [x] 7.1 `tsc --noEmit` in npm/organiza; `npm pack --dry-run` template + `tar -tzvf` executable-bit check. RDP-3/4; risk #9.
- [x] 7.2 `cargo build --release` produces `organiza`; `docker build --platform linux/amd64` smoke. RDP-1; CD-2.
- [x] 7.3 actionlint (if available) on release.yml; grep-audit: no floating action tags. CQV Unpinned-action.

## Phase 8: Verification — spec scenarios (sdd-verify)

- [ ] 8.1 RDP scenarios: rebranded crate, license present, version-sync PR diff (8 refs), wrapper spawn + unsupported, exact pins, dependency-order gate, assets + .sha256, crates gate, --provenance.
- [ ] 8.2 CD + CQV scenarios: image tags, multi-arch, non-root+tini, registry-auth halt; pinned actions, gated publish, documented recovery + release rollback.
- [ ] 8.3 Final gates: cargo gate, tsc, grep audit, agentsync drift clean; update state.yaml (tasks complete → apply).
