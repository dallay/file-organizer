# File Organizer en Rust

Motor multiplataforma para macOS, Linux y Windows. La lógica de clasificación vive en un binario Rust; los schedulers de cada sistema operativo solo lo ejecutan.

## Compilar

Requiere Rust estable:

```bash
cargo test
cargo build --release
```

El binario queda en `target/release/file-organizer` (`file-organizer.exe` en Windows).

## Configurar

```bash
mkdir -p "$HOME/.config/file-organizer"
cp config.example.toml "$HOME/.config/file-organizer/config.toml"
```

En Windows, copia el archivo a `%APPDATA%\\file-organizer\\config.toml`. Edita las carpetas y ejecuta:

```bash
file-organizer --config ~/.config/file-organizer/config.toml validate-config
```

## Ejecutar

```bash
file-organizer run --dry-run
file-organizer run
file-organizer run --verbose ~/Downloads
file-organizer run --config ./config.toml --log /dev/null
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

If you run `file-organizer run` before creating a config file, the binary synthesizes `Config::default()` and auto-detects a Downloads directory. The lookup order is:

1. `FILE_ORGANIZER_DOWNLOADS` env var (if set and the path exists).
2. Linux: `XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs`.
3. macOS: `~/Downloads`.
4. Windows: `%USERPROFILE%/Downloads`, falling back to localized names (`Descargas`, `Téléchargements`, `Scaricati`, `下载`).

**One-time reclassification of legacy folders.** If you previously ran an older Spanish-defaults version, your `~/Downloads/Imágenes/`, `~/Downloads/Documentos/`, etc. will not be re-entered on the next run because only the seven English names are recognized as generated categories. Files inside those legacy folders are picked up once and moved under the new English-named categories. After that single pass, the legacy folders are empty and can be removed manually.

POSIX `launchd` and `systemd` schedulers that previously no-op'd (no config + no Downloads override) will begin organizing on each tick. Existing launchers in `platform/` are unchanged; the README is the only documentation to update.

## Automatización por plataforma

- **macOS:** usa Shortcuts o `launchd` para ejecutar `file-organizer run`.
- **Linux:** usa el `systemd` user timer incluido en `platform/linux/`.
- **Windows:** usa Task Scheduler con `schtasks`.

El lock se crea con `create_dir`, por lo que no depende de `flock` y funciona en los tres sistemas.

### macOS con launchd

```bash
mkdir -p "$HOME/.local/bin" "$HOME/Library/LaunchAgents"
cp target/release/file-organizer "$HOME/.local/bin/"
sed "s#TU_USUARIO#$(whoami)#" platform/macos/com.file-organizer.plist.example \
  > "$HOME/Library/LaunchAgents/com.file-organizer.plist"
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.file-organizer.plist"
```

Para detenerlo: `launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.file-organizer.plist"`.

### Linux con systemd user

```bash
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
cp target/release/file-organizer "$HOME/.local/bin/"
cp platform/linux/file-organizer.service platform/linux/file-organizer.timer \
  "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now file-organizer.timer
```

### Windows con Task Scheduler

Después de copiar `file-organizer.exe` a una ruta permanente y crear el TOML en `%APPDATA%\\file-organizer\\config.toml`:

```powershell
schtasks /Create /TN "File Organizer" /SC MINUTE /MO 5 `
  /TR "C:\\Users\\TU_USUARIO\\.local\\bin\\file-organizer.exe run" /F
```

## Alcance actual

El movimiento utiliza `rename`, que es atómico dentro del mismo volumen. Si el origen y destino están en volúmenes distintos, se informa del error en vez de borrar o copiar parcialmente el archivo.

## Development tooling

Contributor setup for local quality gates, agent-instruction sync, and CI. No Cargo or npm dependency is added to this Rust-only package.

### Required tools (exact versions)

| Tool | Version | Install |
|------|---------|---------|
| Rust (stable) | pinned by `rust-toolchain.toml` (components: `rustfmt`, `clippy`) | [rustup.rs](https://rustup.rs) |
| Node.js | >= 18 (CI uses 22 LTS; local 24 works) | [nodejs.org](https://nodejs.org) or fnm/nvm |
| Lefthook | 2.1.10 | `brew install lefthook` (macOS) or see [lefthook docs](https://github.com/evilmartians/lefthook#install) |
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

### Rollback

To remove the tooling:

1. Delete `.github/workflows/ci.yml`, `lefthook.yml`, `rust-toolchain.toml`, and `.agents/`.
2. Run `lefthook uninstall` to remove the registered git hooks.
3. Remove the `# START AI Agent Symlinks` / `# END AI Agent Symlinks` block from `.gitignore` (keep `target/` and `.DS_Store` if desired).
4. Restore the reviewed root `AGENTS.md` from version control: its content is committed in `.agents/AGENTS.md`, so `git show HEAD:.agents/AGENTS.md > AGENTS.md` (after step 1) restores the file.

No application code or Cargo dependency rollback is required.
