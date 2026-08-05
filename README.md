# organiza en Rust

Motor multiplataforma para macOS, Linux y Windows. La lógica de clasificación vive en un binario Rust; los schedulers de cada sistema operativo solo lo ejecutan.

## Instalar

Hay tres canales de instalación, todos publicados desde el mismo tag de release:

```bash
# 1. crates.io
cargo install organiza

# 2. npm (wrapper con binarios por plataforma)
npm install -g @dallay/organiza

# 3. Docker (multi-arquitectura linux/amd64 + linux/arm64)
docker pull yacosta738/organiza
docker pull ghcr.io/dallay/organiza
```

También puedes descargar el binario de tu plataforma desde los assets del [GitHub Release](https://github.com/dallay/file-organizer/releases).

## Compilar

Requiere Rust estable:

```bash
cargo test
cargo build --release
```

El binario queda en `target/release/organiza` (`organiza.exe` en Windows).

## Configurar

```bash
mkdir -p "$HOME/.config/organiza"
cp config.example.toml "$HOME/.config/organiza/config.toml"
```

En Windows, copia el archivo a `%APPDATA%\\organiza\\config.toml`. Edita las carpetas y ejecuta:

```bash
organiza --config ~/.config/organiza/config.toml validate-config
```

## Ejecutar

```bash
organiza run --dry-run
organiza run
organiza run --verbose ~/Downloads
organiza run --config ./config.toml --log /dev/null
```

Las carpetas indicadas al final sustituyen a `source_directories`. El comportamiento predeterminado espera 60 segundos, ignora ocultos y renombra conflictos (`archivo (1).pdf`).

## Default categories

The binary ships seven flat English categories:

| Category | Extensions |
|----------|------------|
| `Text` | `txt`, `md`, `rtf`, `doc`, `docx`, `pages`, `pdf`, `xlsx`, `xls`, `pptx`, `ppt`, `key`, `numbers`, `csv`, `epub`, `odt`, `ods`, `odp`, `log`, `tex` |
| `Image` | `jpg`, `jpeg`, `png`, `gif`, `webp`, `heic`, `svg`, `tiff`, `tif` |
| `Video` | `mp4`, `mov`, `mkv`, `avi`, `webm`, `m4v` |
| `Audio` | `mp3`, `m4a`, `wav`, `flac`, `ogg`, `aac` |
| `Executable` | `dmg`, `pkg`, `msi`, `exe`, `deb`, `rpm` |
| `Compressed` | `zip`, `rar`, `7z`, `tar`, `gz`, `bz2`, `xz` |
| `Other` | Everything else, including files without an extension and code/source files (`.rs`, `.py`, `.js`, `.ts`, …). Add a `[[categories]]` rule below to carve out a `Code` category if you want one. |

Files at the top of a source directory that are not one of the seven built-in folders are moved to `Other/<dirname>/`. Empty, symlinked, and (when `ignore_hidden = true`) hidden directories are skipped.

## Customizing categories

Add `[[categories]]` blocks to extend or replace the built-ins. Without `replace`, the rule supplements the built-in category. With `replace = true`, the rule substitutes the built-in list. `[extensions]` is applied last and wins over both:

```toml
# Add a new `Code` category without touching the built-ins.
[[categories]]
name = "Code"
extensions = ["rs", "py", "ts", "js", "go"]

# Replace the `Text` built-in with a single extension.
[[categories]]
name = "Text"
extensions = ["onlytxt"]
replace = true

# Per-extension overrides win last.
[extensions]
md = "Docs"
```

## First-run behavior

If you run `organiza run` before creating a config file, the binary synthesizes `Config::default()` and auto-detects a Downloads directory. The lookup order is:

1. `ORGANIZA_DOWNLOADS` env var (if set and the path exists).
2. Linux: `XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs`.
3. macOS: `~/Downloads`.
4. Windows: `%USERPROFILE%/Downloads`, falling back to localized names (`Descargas`, `Téléchargements`, `Scaricati`, `下载`).

**One-time reclassification of legacy folders.** If you previously ran an older Spanish-defaults version, your `~/Downloads/Imágenes/`, `~/Downloads/Documentos/`, etc. will not be re-entered on the next run because only the seven English names are recognized as generated categories. Files inside those legacy folders are picked up once and moved under the new English-named categories. After that single pass, the legacy folders are empty and can be removed manually.

**One-time path change after rebrand.** The config directory moved from `~/.config/file-organizer` to `~/.config/organiza` (and `%APPDATA%\\organiza` on Windows), and the lock from `~/.cache/file-organizer.lock` to `~/.cache/organiza.lock`. Existing configs are not migrated automatically: copy your `config.toml` once to the new path and restart the scheduler. Legacy `Downloads` folders are handled as described above.

POSIX `launchd` and `systemd` schedulers that previously no-op'd (no config + no Downloads override) will begin organizing on each tick. The launchers in `platform/` now invoke `organiza run`.

## Automatización por plataforma

- **macOS:** usa Shortcuts o `launchd` para ejecutar `organiza run`.
- **Linux:** usa el `systemd` user timer incluido en `platform/linux/`.
- **Windows:** usa Task Scheduler con `schtasks`.

El lock se crea con `create_dir`, por lo que no depende de `flock` y funciona en los tres sistemas.

### macOS con launchd

```bash
mkdir -p "$HOME/.local/bin" "$HOME/Library/LaunchAgents"
cp target/release/organiza "$HOME/.local/bin/"
sed "s#TU_USUARIO#$(whoami)#" platform/macos/com.organiza.plist.example \
  > "$HOME/Library/LaunchAgents/com.organiza.plist"
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.organiza.plist"
```

Para detenerlo: `launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.organiza.plist"`.

### Linux con systemd user

```bash
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
cp target/release/organiza "$HOME/.local/bin/"
cp platform/linux/organiza.service platform/linux/organiza.timer \
  "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now organiza.timer
```

### Windows con Task Scheduler

Después de copiar `organiza.exe` a una ruta permanente y crear el TOML en `%APPDATA%\\organiza\\config.toml`:

```powershell
schtasks /Create /TN "organiza" /SC MINUTE /MO 5 `
  /TR "C:\\Users\\TU_USUARIO\\.local\\bin\\organiza.exe run" /F
```

## Alcance actual

El movimiento utiliza `rename`, que es atómico dentro del mismo volumen. Si el origen y destino están en volúmenes distintos, se informa del error en vez de borrar o copiar parcialmente el archivo.

## Development tooling

Contributor setup for local quality gates, agent-instruction sync, and CI. The core package is Rust-only; the npm wrapper (`npm/organiza`) is a thin TypeScript launcher that ships the per-platform binaries as optional dependencies.

### Required tools (exact versions)

| Tool | Version | Install |
|------|---------|---------|
| Rust (stable) | pinned by `rust-toolchain.toml` (components: `rustfmt`, `clippy`) | [rustup.rs](https://rustup.rs) |
| Node.js | >= 18 (CI uses 22 LTS; local 24 works) | [nodejs.org](https://nodejs.org) or fnm/nvm |
| Lefthook | 2.1.10 | macOS/Linux: `brew install lefthook`; Windows: `winget` or `scoop`; Linux distros: Debian/RPM/Alpine/Arch packages — see [lefthook docs](https://github.com/evilmartians/lefthook#install) for the exact package name per system |
| AgentSync | 1.45.2 (`@dallay/agentsync` on npm) | invoked via `npx`, no global install needed |

### One-time setup

```bash
lefthook install                                          # register git hooks from lefthook.yml
npx --yes @dallay/agentsync@1.45.2 apply                  # create generated instruction symlinks
```

Health check — run this locally to confirm generated instruction symlinks are in sync; CI runs the same command and fails on drift:

```bash
npx --yes @dallay/agentsync@1.45.2 status --json
```

### Hooks (lefthook.yml)

- `pre-commit`: `cargo fmt -- --check` — blocks on unformatted code.
- `pre-push`: `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo test` — blocks on failures.
- `post-checkout`, `post-merge`, `post-rewrite`: `npx --yes @dallay/agentsync@1.45.2 apply || true` — non-blocking refresh; a missing/broken AgentSync only surfaces via `agentsync status --json`.

### Agent instructions

`.agents/AGENTS.md` is the single canonical source. `agentsync apply` creates symlinks at `AGENTS.md` (repository root), `CLAUDE.md`, and `.github/copilot-instructions.md`. Generated destinations are ignored through the marker-managed block in `.gitignore`; ordinary ignores (`target/`, `.DS_Store`) live outside that block. OpenCode consumes the root `AGENTS.md` and has no separate target; no MCP config is generated.

### CI (.github/workflows/ci.yml)

All third-party actions are pinned to full commit SHAs with a version comment.

- `quality` (ubuntu): `cargo fmt -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- `test` (ubuntu, macos, windows): `cargo test`.
- `agentsync` (ubuntu): runs `agentsync apply --no-gitignore` in an isolated copy of the checkout so `.gitignore` is never persisted to the committed state, then `agentsync status --json` as the blocking drift assertion. Windows CI never creates AgentSync symlinks.

### Version ownership

Tool versions are pinned in this repository (see the table above; `rust-toolchain.toml` pins the Rust channel). Any version bump is a deliberate change requiring maintainer review. GitHub Actions are referenced by full commit SHA with a version comment; update the SHA and the comment together.

### Windows symlink limits

AgentSync uses symbolic links. On Windows, creating symlinks requires Developer Mode or elevated privileges (see [Microsoft: enable your device for development](https://learn.microsoft.com/en-us/windows/apps/get-started/enable-your-device-for-development)). CI does not require symlink creation on Windows; contributors should run `agentsync status --json` after `apply` to confirm the sync worked.

### Release pipeline (.github/workflows/release.yml)

Every third-party action is pinned to a full commit SHA with a version comment; external tools (cross, Node/npm, Docker buildx/QEMU) are pinned to exact versions. Nothing publishes until every preceding job passes.

Jobs, in order:

1. `release-please` — opens/updates the release PR (release-type rust, component `organiza`), bumps `Cargo.toml` and the npm wrapper versions, and creates the GitHub Release with the `organiza-<version>-<target>` binaries + `.sha256` assets.
2. `build-binaries` (8 targets) — linux x86_64/aarch64 (gnu + musl, cross-compiled with `cross`), darwin x86_64/aarch64, windows x86_64/aarch64. Uploads archives as workflow artifacts.
3. `upload-assets` — attaches the archives to the GitHub Release.
4. `publish-npm-binaries` (6 platform packages `@dallay/organiza-<os>-<arch>`) — gated on release-please result success; publishes with `--provenance`.
5. `publish-npm-base` — publishes `@dallay/organiza` (wrapper), gated on the six platform packages succeeding.
6. `publish-crates` — `cargo publish --locked` for the `organiza` crate.
7. `publish-docker` — multi-arch image (`linux/amd64`, `linux/arm64`) to Docker Hub `yacosta738/organiza` and GHCR `ghcr.io/dallay/organiza`, tagged `semver` + `latest`.
8. `release-summary` — aggregates the results of every publish job.

Secrets required (repo/organization secrets, or the configured environment):

| Secret | Used by |
|--------|---------|
| `GH_APP_ID`, `GH_APP_PRIVATE_KEY` | `release-please` (create-github-app-token) |
| `NPM_TOKEN` | `publish-npm-*` |
| `CARGO_REGISTRY_TOKEN` | `publish-crates` |
| `DOCKERHUB_TOKEN` (with `DOCKERHUB_USERNAME`) | `publish-docker` |
| `GHCR_TOKEN` | `publish-docker` (GHCR) |

`workflow_dispatch` with `dry_run: true` runs the pipeline end-to-end without publishing or attaching assets.

**Release rollback.** If a release is published with an error:

1. npm: `npm unpublish @dallay/organiza@<version> --force` within 72 hours of publish (and the corresponding `@dallay/organiza-<os>-<arch>` packages).
2. crates.io: `cargo yank --version <version>` (crates.io does not allow deletion).
3. Docker: re-tag the previous good image as `latest` and, if needed, remove the bad semver tag from both registries.
4. GitHub: delete the Release (and its assets) for the bad tag.

### Rollback

To remove the tooling:

1. Delete `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `lefthook.yml`, `rust-toolchain.toml`, `release-please-config.json`, `.release-please-manifest.json`, `Dockerfile`, `.dockerignore`, `npm/`, `scripts/`, and `.agents/`.
2. Run `lefthook uninstall` to remove the registered git hooks.
3. Remove the `# START AI Agent Symlinks` / `# END AI Agent Symlinks` block from `.gitignore` (keep `target/`, `.DS_Store`, and the npm ignores if desired).
4. Restore the reviewed root `AGENTS.md` from version control: its content is committed in `.agents/AGENTS.md`, so `git show HEAD:.agents/AGENTS.md > AGENTS.md` (after step 1) restores the file.

No application code or Cargo dependency rollback is required.
