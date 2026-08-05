# Architecture Decisions (ADRs)

Each ADR: **Choice** / **Alternatives** / **Rationale**. Locked decisions from `proposal.md` are not revisited; these capture the WHY for apply.

## ADR-1: Rebrand crate+binary to `organiza`

| Option | Tradeoff | Decision |
|---|---|---|
| Keep `file-organizer` | crate name is likely taken on crates.io; hyphenated crate names are deprecated by crates.io policy | **Rename** to `organiza` |
| `file-organizer-cli` suffix | long, forgettable, still risks collision | rejected |

**Rationale**: crates.io does not accept new crates with hyphens; the exact name `file-organizer` is already taken. `organiza` is short, typable (`cargo install organiza`), matches the Spanish-language product voice, and frees npm scope via `@dallay/organiza`. The repo stays `github.com/dallay/file-organizer` (no rename churn on GitHub redirects/issues), so `repository` metadata points at the existing URL.

## ADR-2: Plain npm, NOT pnpm

| Option | Tradeoff | Decision |
|---|---|---|
| pnpm workspace (agentsync) | needs root `package.json` + `pnpm-workspace.yaml`; every publish job installs pnpm; workspace exists only to be template-generated | **Plain npm** |
| Plain npm + committed packages | committed 6 platform dirs would leak per-release binaries into git history | rejected (keeps template) |

**Rationale**: agentsync's pnpm workspace hosts a *developed* monorepo. Ours has exactly one hand-written package (the wrapper); the other 6 are materialized in CI from `package.json.tmpl`. A workspace adds lockfiles, install steps, and toolchain churn with zero benefit. `npm publish --provenance` (npm ≥ 9) gives the same OIDC provenance agentsync gets from pnpm; only the `pnpm/action-setup` steps are dropped.

## ADR-3: Template + envsubst in CI, not committed platform packages

| Option | Tradeoff | Decision |
|---|---|---|
| Committed `@dallay/organiza-*` dirs | version drift between dirs; binaries in git; 6× package.json maintenance | rejected |
| CI `envsubst < npm/package.json.tmpl` | one template, one version, binaries only inside published tarballs | **Adopted** (agentsync parity) |

**Rationale**: the platform package is a pure function of (version, os, arch, bin name). Materializing it in the `publish-npm-binaries` job from one template guarantees every platform package carries the same fields and the same release version. `os`/`cpu` fields are set from the matrix config, so npm install selects exactly one package per host (sequence (c)). Binary extraction reuses the agentsync validate-exactly-one-archive + executable-bit-in-tarball guard to avoid publishing a broken package.

## ADR-4: Wrapper via `require.resolve` + `spawnSync`

| Option | Tradeoff | Decision |
|---|---|---|
| `spawn` (async) + event handlers | more code, signal/stdio plumbing; the CLI is short-lived | rejected |
| Locate binary by string join alone | breaks under npm hoisting/pnpm layouts | rejected |
| `require.resolve("<pkg>/package.json")` + `spawnSync` | resolves the real install location; synchronous child with inherited stdio; exit-code passthrough | **Adopted** (agentsync parity) |

**Rationale**: `require.resolve` finds the platform package wherever npm placed it (hoisted or nested); `join(pkgPath, "..", "bin", name)` derives the binary. `spawnSync` with `stdio: "inherit"` makes the wrapper a transparent proxy: args pass through (`process.argv.slice(2)`), the exit status propagates, and Ctrl-C/termination semantics stay with the child. Node ≥ 18 engines field matches `node:child_process` availability.

## ADR-5: release-please with `extra-files` jsonpaths as the single version source

| Option | Tradeoff | Decision |
|---|---|---|
| A custom `update-versions.js` in CI | second source of truth; a script can silently diverge from release-please's computed version | rejected for CI |
| release-type `rust` + `extra-files` jsonpaths | Cargo.toml bumped natively; wrapper version + 6 optionalDeps pinned to the SAME release version in the release PR | **Adopted** (agentsync parity) |

**Rationale**: `release-type: rust` bumps `Cargo.toml` itself. The `extra-files` block pins `npm/organiza/package.json` `$.version` and each `$.optionalDependencies['@dallay/organiza-…']` to the identical version — so the release PR diffs all 8 version references atomically and reviewers see the exact version before any publish. The committed `update-versions.js` remains only as a local/dev convenience, not part of CI. First-run behavior: manifest starts at `0.1.0`; the first merged conventional-commit set produces the first release PR (risk #5).

## ADR-6: 8-target matrix — native where cheap, `cross` for Linux cross-compiles

| Option | Tradeoff | Decision |
|---|---|---|
| cargo-zigbuild everywhere | extra tool; unnecessary for a pure-Rust crate | rejected |
| Native + cross split (agentsync) | native x86_64 GNU is fast & container-free; `cross build` handles musl×2 + aarch64 GNU in one container | **Adopted**, minus apt steps |
| Keep agentsync's apt gcc-12/musl-tools/aarch64-gcc steps | installs host toolchains never used by pure-Rust deps | dropped |

**Rationale**: the crate links no C code, so host `gcc`/`musl-tools` installs are dead weight — `cross` containers supply the full cross toolchains (musl static linking for the two `-musl` targets; aarch64 GNU for the ARM one). macOS builds natively with the target added via `dtolnay/rust-toolchain`; Windows MSVC builds natively (Rust ships the MSVC cross linker for `aarch64-pc-windows-msvc`). `fail-fast: false` so one target failure doesn't cancel the others. musl targets exist for the Docker image + static release assets, not for npm (npm uses the GNU matrix — agentsync parity).

## ADR-7: Docker multi-stage, alpine+musl, non-root, tini

| Option | Tradeoff | Decision |
|---|---|---|
| Ubuntu/debian runtime | 2–4× image size for no benefit | rejected |
| Single-stage rust image | ships toolchain + source in production | rejected |
| `rust:1-alpine` builder → `alpine:3.23` runtime | ~10 MB musl static binary; `apk add musl-dev` only (pure Rust); `strip`; non-root uid 1000; `tini` entrypoint for signal handling; OCI labels | **Adopted** (agentsync parity, minus pkgconf) |

**Rationale**: Alpine's musl produces a statically-linked binary that runs on any glibc host too. The builder stage copies `Cargo.toml`/`Cargo.lock` + a dummy `src/` first so dependency layers cache; `--locked` keeps reproducibility. `tini` guarantees SIGTERM reaches the `organiza` process (relevant for scheduler-style usage). `USER organiza` + `WORKDIR /workspace` makes volume mounts user-owned. `cache-from/to: type=gha` keeps multi-arch rebuilds fast.

## ADR-8: Publish order and failure semantics

| Option | Tradeoff | Decision |
|---|---|---|
| Publish base before/parallel to binaries | `@dallay/organiza@X` would reference platform packages that may not exist → broken installs | rejected |
| Binaries → gate → base → crates → docker | base job runs only if `needs.publish-npm-binaries.result == 'success'`; a single missing archive fails the chain (validate-exactly-one + executable-bit guard) before the wrapper ships | **Adopted** (agentsync parity) |

**Rationale**: npm install of the base at version X resolves optionalDependencies at exactly X — they must exist first. `release-please → build-binaries` gates ALL publishers (`if new_release_created == 'true'`), so a dry-run dispatch or a no-op push publishes nothing. crates.io and Docker depend only on `[release-please, build-binaries]` and run in parallel with npm; `release-summary` aggregates every result with `if: always()`. Fail-fast on missing binary: the platform publish step errors if zero or >1 archives match, if extraction yields no binary, or if the packed tarball lacks the executable bit (prevents a silently-broken package).

## ADR-9: PR slicing — rename PR first, then pipeline PR

| Option | Tradeoff | Decision |
|---|---|---|
| One mega-PR (rename + pipeline) | >400 changed lines; reviewer load spikes; rename failures contaminate pipeline review | rejected |
| PR1 rename+metadata+docs → PR2 pipeline+npm+Docker | each PR autonomous: PR1 green (`cargo test`, fmt, clippy, agentsync drift) before any release tag; PR2 adds only additive files | **Adopted** (delivery strategy: chained PRs) |

**Rationale**: the rename is mechanical but touches every file (`Cargo.toml`, both `src/*.rs` + tests, `platform/`, docs, agentsync source) — a self-contained, reviewable unit that unblocks `cargo build` producing `organiza`. The pipeline is entirely additive (new workflow, templates, Dockerfile, LICENSE, config files) and cannot break the build. Order matters: never merge a release-please PR (its tag triggers all publishing) until PR1 is green and the drift job passes. Forecast: PR1 ≈ 250–350 changed lines, PR2 ≈ 400–500 — split keeps each within the review budget (risk #8).
