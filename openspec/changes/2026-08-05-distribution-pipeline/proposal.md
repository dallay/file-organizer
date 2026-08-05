# Proposal: Rebrand to organiza with Full Distribution Pipeline

## Intent

Rename the CLI from `file-organizer` to `organiza` and add distribution parity with the agentsync release pipeline: release-please versioning, GitHub Release assets, crates.io publish, npm wrapper + 6 platform packages, multi-arch Docker images. Unlocks `cargo install organiza`, `npm i -g @dallay/organiza`, container use. No organizer logic changes.

## Scope

### In Scope
- Rebrand crate/binary to `organiza`; Cargo publish metadata (repository, readme, keywords, categories, authors, exclude, `[profile.release]`, LICENSE).
- Release automation: release-please (rust), release assets, crates.io publish.
- npm: `@dallay/organiza` wrapper + 6 platform packages (envsubst template, exact-pinned optionalDependencies, version-sync, provenance).
- Docker: multi-stage musl Dockerfile, `.dockerignore`, multi-arch images (Docker Hub + GHCR).
- Brand updates: `platform/` launchers, README, AGENTS.md, `.gitignore`, openspec config.

### Out of Scope
- Organizer logic, config schema, scheduler behavior, dependencies.
- Repo rename (stays `github.com/dallay/file-organizer`).
- Windows Task Scheduler beyond README; existing-install auto-migration.

## Capabilities

### New Capabilities
- `release-distribution-pipeline`: rebrand + publish metadata, release-please, release assets, crates.io + npm publishing.
- `container-distribution`: multi-arch Docker images on Docker Hub and GHCR.

### Modified Capabilities
- `ci-quality-validation`: extend SHA-pinning/version-explicitness to the release workflow; document release jobs in README.

## Approach

Package manager: **plain npm** — agentsync needs pnpm for its developed workspace; our 7 packages are template-generated, so a workspace adds only churn. `npm publish --provenance` (npm >= 9) keeps publish mechanics; no root package.json, wrapper at `npm/organiza/`.
Artifacts: `npm/package.json.tmpl`, `npm/organiza/` (package.json, src/index.ts, scripts/{sync-optional-deps,update-versions}.js), `release-please-config.json` + `.release-please-manifest.json`, `.github/workflows/release.yml` (9 jobs: release-please → binaries → assets → npm → crates.io → Docker), `Dockerfile`, `.dockerignore`, `LICENSE`, updated `Cargo.toml`/`platform/`/docs.
Images: `yacosta738/organiza` (Docker Hub), `ghcr.io/dallay/organiza` (GHCR; needs org PAT, fallback repo-scoped with `GITHUB_TOKEN`). Actions pinned to full SHA; lefthook/agentsync conventions preserved. Delivery: chain PR1 (rename + metadata + docs) then PR2 (pipeline + npm + Docker).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `Cargo.toml`, `platform/` | Modified | Rename, publish metadata, `[profile.release]`, launcher paths. |
| `npm/`, `Dockerfile`, `.dockerignore`, `LICENSE` | New | Wrapper/template/platform pkgs; container build; MIT license. |
| `.github/workflows/release.yml`, `release-please-config.json` + manifest | New | 9-job pipeline; Rust versioning + jsonpaths. |
| `.github/workflows/ci.yml` | Modified | Pin policy; drift refs. |
| `README.md`, `.agents/AGENTS.md`, `.gitignore`, `openspec/config.yaml` | Modified | Brand, install channels, release docs, ignores, context. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| crates.io versions non-deletable | Med | Dry-run publish + green gate; document yank. |
| GHCR org package needs org PAT | Med | Fallback repo-scoped with `GITHUB_TOKEN`. |
| Missed hardcoded `file-organizer` refs | Med | Repo-wide grep post-rename; drift job. |
| PR exceeds 400-line budget | High | Chain PR1 + PR2. |

## Rollback Plan

`git revert` the rename + pipeline commits. Never merge the release-please PR — its tag triggers all publishing. If one escaped: unpublish npm (72h), yank crates.io, retag prior images, delete the GitHub Release. Launchers change atomically with the rename; revert restores `file-organizer` installs.

## Dependencies

Secrets: `NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`, `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN`, GitHub App `APP_ID`/`APP_PRIVATE_KEY`. Tooling: Rust stable, cross, buildx/QEMU, npm >= 9, release-please.

## Success Criteria

- [ ] `cargo install organiza`, `npm i -g @dallay/organiza`, and `docker run` all work from one release-please tag.
- [ ] No `file-organizer` references remain except repo name/history.
- [ ] Rename PR green before any release tag.
