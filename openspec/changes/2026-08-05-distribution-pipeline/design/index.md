# Design: Rebrand to organiza + Full Distribution Pipeline

**Change**: 2026-08-05-distribution-pipeline
**Date**: 2026-08-05
**Source-of-truth**: `proposal.md` (locked decisions); this design is the apply-phase blueprint.

Companion files: [`rationale.md`](./rationale.md) (ADRs 1–9), [`sequences.md`](./sequences.md) (3 mermaid diagrams), [`risks.md`](./risks.md) (register + mitigations).

## 1. Architecture overview

One `release-please` tag becomes five distribution channels, mirroring agentsync at full parity:

```
release-please tag (vX.Y.Z)
  ├── crates.io        cargo install organiza
  ├── npm (7 pkgs)     npm i -g @dallay/organiza
  ├── GitHub Release   organiza-{v}-{target}.{tar.gz|zip} + .sha256 (8 archives)
  ├── Docker Hub       yacosta738/organiza:{v|major.minor|major|latest}
  └── GHCR             ghcr.io/dallay/organiza:{v|major.minor|major|latest}
```

**Single source of truth for version**: the release-please manifest + release-type `rust` (bumps `Cargo.toml`). `extra-files` jsonpaths propagate the same version into the npm wrapper `package.json` and its 6 exact-pinned `optionalDependencies`. Nothing else invents a version. No organizer logic changes; the crate is pure Rust (no C deps), which simplifies cross/Docker.

**Plain npm, not pnpm** (ADR-2): the 7 npm packages are CI template-generated, not a developed workspace, so a root `package.json` workspace adds only churn. `npm publish --provenance` (npm ≥ 9) replaces pnpm's publish mechanics; the `pnpm/action-setup` steps from agentsync are dropped.

## 2. Component inventory

| Path | Action | Role |
|------|--------|------|
| `Cargo.toml` | Modify | `name = "organiza"`; publish metadata (repository/readme/keywords/categories/authors/exclude/description); `[profile.release]` |
| `LICENSE` | Create | MIT text (auto-included in crate + archives + Docker labels) |
| `.github/workflows/release.yml` | Create | 8-job pipeline (see §3) |
| `release-please-config.json`, `.release-please-manifest.json` | Create | Rust release-type, changelog sections, extra-files jsonpaths; manifest `{ ".": "0.1.0" }` |
| `npm/package.json.tmpl` | Create | envsubst template for the 6 platform packages (see §4) |
| `npm/organiza/package.json` | Create | Base wrapper: `bin.organiza → lib/index.js`, `optionalDependencies` = 6 exact `@dallay/organiza-*` |
| `npm/organiza/src/index.ts` | Create | Wrapper: PLATFORMS map + `spawnSync` (see §4) |
| `npm/organiza/scripts/sync-optional-deps.js` | Create | Sets all `@dallay/organiza-*` optionalDeps to one version |
| `npm/organiza/scripts/update-versions.js` | Create | Dev helper: bumps `npm/organiza/package.json` + runs sync-optional-deps (Cargo handled by release-please) |
| `npm/organiza/tsconfig.json` | Create | Compiles `src/index.ts` → `lib/index.js` |
| `Dockerfile`, `.dockerignore` | Create | 2-stage alpine+musl build; non-root + tini runtime (see §5) |
| `src/main.rs`, `src/lib.rs` | Modify | Clap `name = "organiza"`; crate refs `organiza::`; env `ORGANIZA_CONFIG`/`ORGANIZA_DOWNLOADS`; config dir `~/.config/organiza`; lock `~/.cache/organiza.lock` (+ tests updated) |
| `platform/linux/organiza.{service,timer}`, `platform/macos/com.organiza.plist.example` | Rename+Modify | Launchers invoke `organiza run`; scheduler semantics unchanged |
| `.agents/agentsync.toml` + regenerated `.agents/AGENTS.md` | Modify | Repo map / artifact path refs (`target/release/organiza`, config/lock paths) — edit the TOML, not the generated MD (drift job asserts) |
| `.gitignore` | Modify | Add `npm/**/node_modules/`, `npm/organiza/lib/`, `npm/organiza/*.tgz` OUTSIDE the AgentSync managed block |
| `README.md`, `openspec/config.yaml` | Modify | Brand, install channels (`cargo install organiza`, `npm i -g @dallay/organiza`, `docker pull`), release docs, config context |
| `.github/workflows/ci.yml` | Modify | Extend SHA-pinning/version-explicitness policy mention to release workflow; drift job unchanged |

## 3. Release pipeline — job graph

8 jobs (agentsync parity; proposal's "9 jobs" is a descriptive miscount — the graph below is complete):

```yaml
# .github/workflows/release.yml — job skeleton (pins: full SHA + # version comment)
permissions: { contents: write, issues: write, pull-requests: write,
               packages: write, id-token: write }          # release-please + docker
on: { push: { branches: [main] }, workflow_dispatch: { inputs: { dry_run: bool } } }
concurrency: { group: release-${{ github.ref }}, cancel-in-progress: false }

jobs:
  release-please:        # ubuntu; outputs release_created/version/tag_name
    steps: [create-github-app-token(APP_ID/APP_PRIVATE_KEY),
            checkout(fetch-depth: 0, token: app token),
            googleapis/release-please-action(config-file, target-branch: main)]
  build-binaries:        # needs: release-please; if new_release_created=='true'
    strategy: fail-fast: false; matrix: 8 targets (§6)
    steps: [checkout(ref: tag), dtolnay/rust-toolchain(targets),
            build (native x86_64-unknown-linux-gnu | cross for other 3 Linux | native macOS/Windows),
            prepare artifact (tar.gz|zip + sha256), upload-artifact]
  upload-assets:         # needs: [release-please, build-binaries]
    steps: [app-token, download-artifact(pattern organiza-*), softprops/action-gh-release]
  publish-npm-binaries:  # needs: [release-please, build-binaries]; if new_release_created
    strategy: matrix: 6 configs (§6); permissions: { contents: read, id-token: write }
    steps: [checkout(tag), download-artifact(per target), setup-node(24, npm registry),
            envsubst template → publish @dallay/organiza-{os}-{arch}]
  publish-npm-base:      # needs: [release-please, publish-npm-binaries]; if result=='success'
    steps: [checkout(tag), setup-node, npm install, npm run build (tsc), npm publish --provenance]
  publish-crates:        # needs: [release-please, build-binaries]
    steps: [checkout(tag), dtolnay/rust-toolchain, cargo publish --locked]
  publish-docker:        # needs: [release-please, build-binaries]
    steps: [checkout(tag), set up QEMU, set up buildx, login Docker Hub + GHCR,
            metadata-action(tags), build-push-action(platforms: linux/amd64,linux/arm64)]
  release-summary:       # needs: all; if always() && new_release_created
    steps: [append install commands to $GITHUB_STEP_SUMMARY]
```

`publish-npm-base` runs ONLY when every platform package published (`needs.publish-npm-binaries.result == 'success'`) — a missing binary fails the chain before the base wrapper ships (ADR-8).

## 4. npm topology

```
npm/
├── package.json.tmpl              # envsubst template (platform pkgs)
└── organiza/                      # base wrapper (committed)
    ├── package.json               # version + optionalDeps synced by release-please
    ├── tsconfig.json
    ├── src/index.ts               # wrapper (compiled to lib/index.js)
    └── scripts/
        ├── sync-optional-deps.js
        └── update-versions.js
```

**`npm/package.json.tmpl`** (exact shape; `envsubst` vars in CI: `node_pkg`, `node_version`, `node_os`, `node_arch`, `node_bin`):

```json
{
  "name": "@dallay/${node_pkg}",
  "publishConfig": { "access": "public" },
  "version": "${node_version}",
  "description": "Platform-specific binary for organiza (${node_os}-${node_arch})",
  "author": "Yuniel Acosta <yunielacosta738@gmail.com>",
  "license": "MIT",
  "repository": { "type": "git", "url": "git+https://github.com/dallay/file-organizer.git" },
  "homepage": "https://github.com/dallay/file-organizer#readme",
  "os": ["${node_os}"],
  "cpu": ["${node_arch}"],
  "engines": { "node": ">=18" },
  "bin": { "organiza": "bin/${node_bin}" },
  "files": ["bin"],
  "preferUnplugged": true,
  "scripts": { "prepublishOnly": "chmod -v +x bin/* || true" }
}
```

**Wrapper `npm/organiza/src/index.ts`** — the exact PLATFORMS map (keys mirror the 6-package matrix, `cygwin` aliased to windows):

```ts
const PLATFORMS: Record<string, string> = {
  "darwin-x64":     "@dallay/organiza-darwin-x64",
  "darwin-arm64":   "@dallay/organiza-darwin-arm64",
  "linux-x64":      "@dallay/organiza-linux-x64",
  "linux-arm64":    "@dallay/organiza-linux-arm64",
  "win32-x64":      "@dallay/organiza-windows-x64",
  "win32-arm64":    "@dallay/organiza-windows-arm64",
  "cygwin-x64":     "@dallay/organiza-windows-x64",
  "cygwin-arm64":   "@dallay/organiza-windows-arm64",
};
```

Resolution: `process.platform` + `process.arch` → key → `require.resolve("<pkg>/package.json")` → `join(pkgPath, "..", "bin", "organiza" + (win32/cygwin ? ".exe" : ""))`; `existsSync` guard with actionable reinstall message; `spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit", env: process.env })`; `process.exit(result.status ?? 1)`. Runtime sequence in `sequences.md` (b).

## 5. Dockerfile — stage outline

```
FROM rust:1-alpine AS builder          # musl libc by default on Alpine
  ARG TARGETPLATFORM/BUILDPLATFORM
  apk add musl-dev                     # pure-Rust deps: pkgconf NOT needed
  COPY Cargo.toml Cargo.lock ./        # dummy src/ → cargo build --release (layer cache)
  COPY src ./src
  RUN touch src/main.rs src/lib.rs && cargo build --release --locked
  RUN strip target/release/organiza && target/release/organiza --version
FROM alpine:3.23 AS runtime
  LABEL org.opencontainers.image.*     # title=organiza, source=github.com/dallay/file-organizer
  apk add ca-certificates tini
  addgroup/adduser -u 1000 organiza    # non-root
  COPY --from=builder /build/target/release/organiza /usr/local/bin/organiza
  USER organiza; WORKDIR /workspace
  ENTRYPOINT ["/sbin/tini", "--", "organiza"]; CMD ["--help"]
```

Images: `yacosta738/organiza` + `ghcr.io/dallay/organiza`; tags `{version}`, `{major}.{minor}`, `{major}`, `latest`; platforms `linux/amd64,linux/arm64`; `cache-from/to: type=gha`. `.dockerignore`: `target/`, `.git`, `npm/`, `openspec/`, `.github/`, `node_modules/`.

## 6. Target and package matrix (exact)

**build-binaries (8)** — archive `organiza-{release_version}-{target}.{archive}` + `.sha256`:

| target | runner | archive |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | tar.gz |
| `x86_64-unknown-linux-musl` | ubuntu-latest | tar.gz |
| `aarch64-unknown-linux-gnu` | ubuntu-latest | tar.gz |
| `aarch64-unknown-linux-musl` | ubuntu-latest | tar.gz |
| `x86_64-apple-darwin` | macos-latest | tar.gz |
| `aarch64-apple-darwin` | macos-latest | tar.gz |
| `x86_64-pc-windows-msvc` | windows-latest | zip |
| `aarch64-pc-windows-msvc` | windows-latest | zip |

Linux builds: native `cargo build` for `x86_64-unknown-linux-gnu`; `cross build` for the other three (musl static; aarch64). **No apt gcc-12/musl-tools steps** — pure-Rust deps need no host toolchain; cross containers provide them (ADR-6).

**publish-npm-binaries (6)** — package = `@dallay/organiza-{name}`, os/cpu from template:

| name | target | os | cpu |
|---|---|---|---|
| `linux-x64` | x86_64-unknown-linux-gnu | linux | x64 |
| `linux-arm64` | aarch64-unknown-linux-gnu | linux | arm64 |
| `darwin-x64` | x86_64-apple-darwin | darwin | x64 |
| `darwin-arm64` | aarch64-apple-darwin | darwin | arm64 |
| `windows-x64` | x86_64-pc-windows-msvc | win32 | x64 |
| `windows-arm64` | aarch64-pc-windows-msvc | win32 | arm64 |

Binary inside package: `bin/organiza` (`.exe` for win32). Base package `@dallay/organiza` depends on all six via `optionalDependencies` at the exact release version. musl variants are release-assets/Docker-only, NOT npm packages (npm matrix uses GNU targets — agentsync parity).

## 7. Version sync

`release-please-config.json`: `release-type: rust`, `include-component-in-tag: false`, changelog sections copied from agentsync, package `"."` → component `organiza`, `draft: false`. `extra-files` jsonpaths (all → release version):

- `npm/organiza/package.json` → `$.version`
- `npm/organiza/package.json` → `$.optionalDependencies['@dallay/organiza-linux-x64']` … (6 paths, one per platform)

No root `package.json` entry (plain npm, ADR-2). `.release-please-manifest.json` starts at `{ ".": "0.1.0" }` (matches current `Cargo.toml`). Cargo.toml version is handled natively by release-type `rust`.

## 8. Rebrand surface (exact rename map)

| Old | New |
|---|---|
| `file-organizer` crate/binary | `organiza` |
| `Cargo.toml` version line | moved by release-please |
| `src/main.rs:8` clap `name` | `"organiza"` |
| `file_organizer::` crate refs (main.rs:3,58) | `organiza::` |
| `FILE_ORGANIZER_CONFIG` (lib.rs:68) | `ORGANIZA_CONFIG` |
| `FILE_ORGANIZER_DOWNLOADS` (lib.rs:512, tests) | `ORGANIZA_DOWNLOADS` |
| config dir `~/.config/file-organizer` / `%APPDATA%\file-organizer` (lib.rs:76,82) | `~/.config/organiza` / `%APPDATA%\organiza` |
| lock `~/.cache/file-organizer.lock` / `LOCALAPPDATA/file-organizer.lock` (lib.rs:497,499) | `~/.cache/organiza.lock` / `LOCALAPPDATA/organiza.lock` |
| `platform/linux/file-organizer.{service,timer}` | `platform/linux/organiza.{service,timer}` (ExecStart `organiza run`) |
| `platform/macos/com.file-organizer.plist.example` | `platform/macos/com.organiza.plist.example` (ProgramArguments `organiza run`) |
| `config.example.toml` header | organiza |
| `.agents/agentsync.toml` instructions | organiza paths/artifacts (regenerate via `agentsync apply`) |
| README/AGENTS.md refs | organiza; install channels in §1 |

Existing users keep old config/lock paths on disk — **no auto-migration** (proposal out-of-scope); README documents the one-time path change (risk #6).

## 9. Sequence diagrams

See `sequences.md` — (a) release pipeline, (b) wrapper runtime, (c) npm install-time platform selection. The pipeline and wrapper diagrams are reproduced inline in this index above (§3, §4).

## 10. Rationale

See `rationale.md` — ADR-1…ADR-9 (rebrand, plain npm, template strategy, spawnSync, release-please extra-files, cross matrix, Docker stages, publish order, PR slicing).

## 11. Risks

See `risks.md` — 8-item register (crates.io immutability, GHCR PAT, missed refs, npm provenance, release-please first-run, secrets preconditions, PR size, config-path migration).
