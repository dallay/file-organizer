# Verification Report: 2026-08-05-distribution-pipeline

**Change**: 2026-08-05-distribution-pipeline
**Verified**: 2026-08-05
**Verifier**: sdd-verify sub-agent
**Mode**: openspec
**Strict TDD**: active per openspec/config.yaml (`strict_tdd: true`) — runner `cargo test` exists and ran green; TDD module checks applied (rebrand regression tests `package_is_named_organiza`, `default_config_path_uses_organiza_directory`, `cli_name_is_organiza` exist and pass).

## Verdict: PASS WITH WARNINGS

The single CRITICAL finding (F1 — machine-specific `file:` URL for `@dallay/organiza-darwin-arm64`) was fixed and re-verified: all six platform optionalDependencies now pin exactly to `0.1.0` (RDP-4 PASS). Fix committed as `e7d5aaa` on `feat/organiza-distribution` with the repo's tab style preserved and `npx tsc --noEmit` green. Two non-blocking warnings remain (W2 inert `dry_run` input, W3 redundant musl host steps).

---

## A. Per-spec coverage

### A1. release-distribution-pipeline

| Req | Status | Evidence |
|-----|--------|----------|
| RDP-1 Rebranded crate (binary `organiza`, metadata `organiza`) | **PASS** | `cargo metadata --no-deps` → `package name: organiza`, targets `['organiza','organiza']`; `cargo build --release` → `target/release/organiza`; `./target/release/organiza --version` → `organiza 0.1.0`. `Cargo.toml` name/license/repository/readme/keywords/categories/authors/exclude/`[profile.release]` all present. |
| RDP-1 License present | **PASS** | `LICENSE` at repo root (MIT, Yuniel Acosta); `Cargo.toml` `license = "MIT"`. |
| RDP-2 release-please version sync (one version across Cargo.toml + 7 npm refs) | **PASS** | `release-please-config.json`: `release-type: rust`, 7 extra-files jsonpaths (`$.version` + 6 `$.optionalDependencies['@dallay/organiza-*']`) matching the actual package.json structure; `.release-please-manifest.json` `{ ".": "0.1.0" }` matches Cargo.toml. Runtime proof: `scripts/update-versions.js 1.2.3` in sandbox copy → Cargo.toml 1.2.3 + wrapper 1.2.3 + all 6 optionalDeps exactly 1.2.3. |
| RDP-3 Wrapper spawns binary | **PASS** (runtime) | On darwin-arm64: `node lib/index.js --version` → `organiza 0.1.0`, exit 0; `node lib/index.js --definitely-not-a-flag` → clap error on stderr, exit **2** (child exit-code passthrough confirmed). `src/index.ts` uses `require.resolve('<pkg>/package.json')` + `spawnSync(binary, argv.slice(2), { stdio: "inherit", env })` + `process.exit(result.status ?? 1)`. |
| RDP-3 Unsupported platform error | **PASS** (static) | `PLATFORMS[platformKey]` undefined → `throw new Error('Unsupported platform: <key>\n\nSupported platforms:\n…\n\nPlease open an issue at https://github.com/dallay/file-organizer/issues')`; caught in `run()` → `console.error` + `process.exit(1)`. Message names the unsupported target and carries the issues URL (src/index.ts:42). Runtime path not exercisable without faking `process.platform`; code path is direct. |
| RDP-4 Six platform packages + exact pins | **PASS** (after fix) | **Fixed**: `@dallay/organiza-darwin-arm64` reset from machine-specific `file:` URL to exact `0.1.0` via `node scripts/sync-optional-deps.js 0.1.0` (orchestrator, tab style preserved) — commit `e7d5aaa`. Re-verified: all 6 optionalDeps `0.1.0`, JSON valid, `npx tsc --noEmit` green. The 6 platform packages exist as template configs in the workflow matrix (linux-x64/arm64, darwin-x64/arm64, windows-x64/arm64). |
| RDP-5 Publish order + gating | **PASS** | `publish-npm-binaries` needs `[release-please, build-binaries]` (publishes 6 platform pkgs first, `--provenance`); `publish-npm-base` needs `[release-please, publish-npm-binaries]` with `if: needs.publish-npm-binaries.result == 'success'` — a failed platform package (matrix `fail-fast: false`) fails the matrix job and **skips** the base publish. Assets job (`upload-assets`) needs `[release-please, build-binaries]`, runs with npm publish (spec allows "before or with"). |
| RDP-6 Cross-compiled assets + checksums | **PASS** | 8-target matrix (linux gnu+musl x86_64/aarch64, darwin x64/arm64, windows x86_64/aarch64-msvc); tar.gz for Unix / zip for Windows; `.sha256` via `shasum -a 256` / `Get-FileHash -Algorithm SHA256`; `upload-artifact` with `if-no-files-found: error`. |
| RDP-7 crates.io publish | **PASS** | `publish-crates` job: `cargo publish --locked` with `CARGO_REGISTRY_TOKEN`; crate carries `license = "MIT"` + `repository` (verified in Cargo.toml). |
| RDP-8 npm provenance | **PASS** | Both npm jobs run `npm publish --provenance --access public --no-git-checks`; jobs declare `permissions: { contents: read, id-token: write }`; `setup-node` with npmjs `registry-url`. |

### A2. container-distribution

| Req | Status | Evidence |
|-----|--------|----------|
| CD-1 Image naming + tags | **PASS** | `metadata-action` images: `yacosta738/organiza`, `ghcr.io/dallay/organiza`; tags `type=semver {{version}}/{{major}}.{{minor}}/{{major}}` + `type=raw,value=latest`. |
| CD-2 Multi-arch build | **PASS** | `build-push-action` `platforms: linux/amd64,linux/arm64` with `setup-qemu-action` + `setup-buildx-action`; single manifest, `cache-from/to: type=gha`. |
| CD-3 Runtime safety | **PASS** | Dockerfile: `rust:1-alpine` builder (musl static, dummy-src layer cache, `cargo build --release --locked`, `strip`) → `alpine:3.23` runtime; `apk add tini ca-certificates`; `ENTRYPOINT ["/sbin/tini","--","organiza"]`; `USER organiza` (uid 1000); `CMD ["--help"]`. |
| CD-4 Build hygiene + credentials | **PASS** | `.dockerignore` excludes `target/`, `.git/`, `node_modules/`, `npm/`, `openspec/`, `.github/`; minimal final stage. Docker Hub login uses `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN`; GHCR login uses `GITHUB_TOKEN` (org-PAT fallback `GHCR_PAT` documented in risks.md #2 — accepted design decision). Invalid credentials fail the `publish-docker` job; the needs-chain halts dependent jobs, no partial publish per channel. |

### A3. ci-quality-validation (delta)

| Req | Status | Evidence |
|-----|--------|----------|
| CQV Release workflow pinning | **PASS** | All 13 `uses:` entries are full commit SHAs with `# version` comments (create-github-app-token #v3, checkout #v7.0.1, release-please-action #v5.0.0, rust-toolchain #v1, upload/download-artifact #v7/#v8, action-gh-release #v3.0.0, setup-node #v7.0.0, qemu #v4.0.0, buildx #v4.1.0, login-action #v4.2.0, metadata-action #v6.1.0, build-push-action #v7.2.0). `rg 'uses: [^#]*@[^0-9a-f]{40}'` → **zero floating refs**. Explicit tool versions: cross 0.2.5, Node 24, buildx/QEMU via pinned actions. Publish jobs only run after release-please + build-binaries succeed. |
| CQV Gated publish | **PASS** | All publish jobs `needs` release-please and build-binaries; `publish-npm-base` additionally gates on `publish-npm-binaries.result == 'success'`; skipped/`if:`-guarded chain semantics verified (a failing/skipped dependency skips dependents). |
| CQV Documentation + rollback | **PASS** | README documents: required tools + exact versions table, one-time setup, AgentSync drift policy, Windows symlink limits, release pipeline jobs (release-please, binaries, assets, npm, crates.io, Docker) with a secrets table (`GH_APP_ID/GH_APP_PRIVATE_KEY/NPM_TOKEN/CARGO_REGISTRY_TOKEN/DOCKERHUB_TOKEN/DOCKERHUB_USERNAME/GHCR_TOKEN`), workflow-tooling removal rollback ("No application code or Cargo dependency rollback is required"), and release rollback (npm unpublish ≤72 h, cargo yank, image retag, release deletion). |

---

## B. Gates run

| Gate | Command | Result |
|------|---------|--------|
| Format | `cargo fmt -- --check` (branch feat/organiza-rebrand) | PASS |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | PASS (0 warnings) |
| Rust tests | `cargo test` | PASS — 38 unit + 1 cli test, 0 failures; rebrand regression tests present (RDP-1 guards) |
| RDP-1 runtime | `cargo metadata --no-deps` / `cargo build --release` / `./target/release/organiza --version` | PASS — `organiza 0.1.0` |
| npm install | `npm install` in npm/organiza (plain npm) | PASS exit 0; 5 platform deps "UNMET OPTIONAL DEPENDENCY" (expected pre-first-release); darwin-arm64 installs from local `file:` path (see F1) |
| npm install (CI-like, empty cache, tmp tgz absent) | `npm install --cache <fresh>` after removing the file: target | PASS exit 0 but warns `tarball data … seems to be corrupted` (2×) and does NOT install darwin-arm64 — proves the committed `file:` URL is machine-specific |
| TypeScript | `npx tsc --noEmit` (typescript 7.0.2, module/moduleResolution nodenext) | PASS |
| Wrapper runtime | `node lib/index.js --version`; `node lib/index.js --badflag` | PASS — prints version, exit 0 / exit 2 (child code passthrough) |
| YAML lint | `python3 -c "yaml.safe_load(open('.github/workflows/release.yml'))"` | PASS — parses; 8 jobs, `on` = push main + workflow_dispatch dry_run (KeyError `'on'` is a PyYAML 1.1 boolean quirk, not a file defect) |
| JSON validity | `python3 -c "json.load(…)"` for release-please-config/manifest/package.json.tmpl | PASS |
| Template pack + exec bit | `envsubst < npm/package.json.tmpl` → `npm pack --dry-run` + `tar -tzvf` | PASS — tarball contains `-rwxr-xr-x … package/bin/organiza` (owner-execute bit present; risk #9 guard replicated) |
| Version sync scripts | `node scripts/update-versions.js 1.2.3` in sandbox copy | PASS — Cargo.toml + wrapper + all 6 optionalDeps → exactly 1.2.3 |
| Action pin audit | `rg 'uses: [^#]*@[^0-9a-f]{40}' release.yml` | PASS — zero floating refs |
| Grep audit | `git grep file-organizer` outside openspec | PASS — only repo URL, README migration callout, test negative-assertions, legacy `.atl/skill-registry.md` metadata; no launcher/config/binary-name refs |
| AgentSync drift | post-checkout hook on both branch switches | PASS — `Sync complete / Errors: 0` both times; `.agents/AGENTS.md` carries organiza paths |

### Branch chain integrity

| Check | Result |
|-------|--------|
| `main..feat/organiza-rebrand` commits | Exactly 3: `a9e9349` docs, `8d48905` feat rebrand, `6faf2a9` chore launchers ✓ |
| `feat/organiza-rebrand..feat/organiza-distribution` commits | Exactly 5: `8cbdd39` npm wrapper, `86fcbc5` release pipeline, `d4ca5e1` Dockerfile, `016352f`+`f4c50da` tasks/state docs ✓ |
| PR2 diff vs PR1 additive | `git diff feat/organiza-rebrand feat/organiza-distribution` touches only new PR2 files + tasks.md/state.yaml/config.yaml/.gitignore (+1 line); no PR1 file reverted ✓ |
| PR1 gate on branch state | fmt/clippy/test all green at `6faf2a9` ✓ |

---

## C. Correctness table

| Finding | Judge A | Judge B | Severity | Status |
|---------|---------|---------|----------|--------|
| Committed `file:` URL for darwin-arm64 optionalDependency | ✅ | ✅ | CRITICAL | Confirmed (F1) |
| `dry_run` dispatch input declared but never consumed; README claims it suppresses publishing | ✅ | ✅ | WARNING | Confirmed (F2) |
| release.yml installs host `musl-tools` + rustup targets for musl targets that build via `cross` (design §6/ADR-6 says no host toolchain steps) | ✅ | ❌ | WARNING (design coherence) | Confirmed — harmless, redundant |
| `cargo install cross` recompiles from source each release run (~2–4 min) | ✅ | ✅ | SUGGESTION | Confirmed |
| README documents `npm i -g`/`cargo install`/docker but not `npx organiza` | ✅ | ❌ | SUGGESTION | Confirmed |
| `.atl/skill-registry.md` still describes the project as `file-organizer` | ✅ | ❌ | SUGGESTION (INFO) | Confirmed — legacy generated metadata |
| setup-node `cache` input referencing uncommitted `npm/organiza/package-lock.json` | — | — | RESOLVED | No `cache:` input exists in final release.yml (grep); lockfile gitignored; CI regenerates it per run → no failure risk |
| README "CI uses 22 LTS" vs release.yml node 24 | ✅ | ❌ | INFO | Confirmed — statement refers to ci.yml (node 22), release pipeline uses 24 per design |

---

## D. Warnings (non-blocking)

- **W1 (WARNING)**: `npm/organiza/package.json` — see Failures F1 (the only blocking item; listed here for completeness of the fix).
- **W2 (WARNING)**: `dry_run` input in `release.yml` is declared but never referenced by any step (grep shows a single occurrence at the input declaration). README says "workflow_dispatch with `dry_run: true` runs the pipeline end-to-end without publishing or attaching assets" — this is currently false. The input is inert; a manual dispatch behaves exactly like a push to main (release-please is idempotent, so in practice it would just re-open/refresh the release PR). **Fix**: pass `dry-run: ${{ inputs.dry_run || false }}` to `googleapis/release-please-action` (release-please supports `dry-run`), or delete the input and soften the README claim.
- **W3 (WARNING, design coherence)**: `build-binaries` installs host `musl-tools` and rustup targets for musl targets that are actually built with `cross` (design §6: "No apt gcc-12/musl-tools steps — pure-Rust deps need no host toolchain; cross containers provide them", ADR-6). Redundant and harmless, but it deviates from the approved design. **Fix (optional)**: drop the musl-tools step and the `targets:` input for cross-built targets.
- **W4 (SUGGESTION)**: `cargo install cross --locked --version 0.2.5` compiles cross from source every release (no cache). Use `dtolnay/install` or the `cross` GitHub Action / a cached `~/.cargo/bin` to shave minutes.
- **W5 (SUGGESTION)**: README covers `npm install -g @dallay/organiza` but not the `npx @dallay/organiza` one-shot channel (which the release summary advertises). One line to add.
- **W6 (INFO)**: `.atl/skill-registry.md` is generated legacy metadata still naming `file-organizer`; regenerate or ignore (not launchers/config/docs, so allowed by the audit rule).

---

## E. Failures (blocking)

### F1 — CRITICAL: `@dallay/organiza-darwin-arm64` optionalDependency is a machine-specific `file:` URL (RDP-4 violated as committed)

**Status: RESOLVED** — fixed by orchestrator (commit `e7d5aaa` on `feat/organiza-distribution`) using the exact fix below; re-verified RDP-4 PASS with `tsc --noEmit` green.

**Location**: `npm/organiza/package.json` (fixed on `feat/organiza-distribution`)

```json
"optionalDependencies": {
  "@dallay/organiza-darwin-arm64": "file:../../../../../../var/folders/zz/d4kl1hfj1j15nxm43d24px300000gn/T/tmp.XsZYeE5nam/dallay-organiza-darwin-arm64-0.1.0.tgz",
  "@dallay/organiza-darwin-x64": "0.1.0",
  ...
}
```

**Impact** (assessed, not speculative):
- Spec RDP-4 Scenario "Exact optional dependencies" fails as committed: "every optionalDependency is pinned to exactly X.Y.Z" — darwin-arm64 is a temp-file path, not `0.1.0`.
- On any machine other than this one (including CI), `npm install` warns `tarball data … seems to be corrupted` (verified with an empty npm cache) and the darwin-arm64 binary is not installed. It does **not** hard-fail the pipeline: optionalDependencies tolerate failure, `publish-npm-base`'s `node -e` step rewrites every optionalDep to `${{ release_version }}` before publish, and release-please's jsonpath `$.optionalDependencies['@dallay/organiza-darwin-arm64']` would replace it on the first release PR — so the published package is correct.
- Residual risk if the base package is ever published outside the release pipeline (manual `npm publish` from main): the broken URL ships.

**Exact fix** (one line): reset the pin, e.g. run from `npm/organiza`:

```bash
node scripts/sync-optional-deps.js 0.1.0
```

which sets `"@dallay/organiza-darwin-arm64": "0.1.0"` (script runtime-proven in this verification). Amend the commit `8cbdd39` on `feat/organiza-distribution`, then re-run `npm install` (clean) + `npx tsc --noEmit` + re-inspect optionalDependencies.

---

## F. Final

- **Verdict**: PASS WITH WARNINGS (F1 CRITICAL resolved and re-verified; W2/W3 non-blocking warnings remain)
- **Spec compliance**: release-distribution-pipeline 10/10 (RDP-4 fixed); container-distribution 4/4; ci-quality-validation 3/3
- **Gates**: all green (fmt, clippy, 39 cargo tests, tsc, YAML/JSON lint, template pack exec-bit, version-sync scripts, wrapper spawn + exit passthrough, action-pin audit, grep audit, agentsync drift)
- **Branch integrity**: PASS (PR1 = 3 commits, PR2 = 6 commits incl. fix `e7d5aaa`, additive chain)
- **next_recommended**: archive (PASS WITH WARNINGS — W2/W3 reviewed and accepted as non-blocking)
