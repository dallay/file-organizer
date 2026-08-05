# Downloads Auto-Detection

**Capability**: downloads-autodetect
**Status**: stable
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.
**Source of truth**: merged from `openspec/changes/archive/2026-08-04-configurable-categories-and-sources/specs/downloads-autodetect/spec.md`.

## Purpose

Defines how the binary selects a Downloads source directory when no
`source_directories` is configured and no CLI positional directories are
given. The lookup order is: `FILE_ORGANIZER_DOWNLOADS` → Linux
`XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs` → macOS
`~/Downloads` → Windows `%USERPROFILE%/Downloads` with localized
fallbacks (`Descargas`, `Téléchargements`, `Scaricati`, `下载`). The
selected path is `expand_home`-expanded before use. Tests inject a
synthetic HOME via the `home_override` parameter to avoid mutating
process environment.

## Requirements

### Requirement: Environment Variable Override

The system MUST honor the `FILE_ORGANIZER_DOWNLOADS` environment
variable when set and non-empty. The variable value MUST be
`expand_home`-expanded before use. No other auto-detection step MUST
run when this variable is present.

#### Scenario: FILE_ORGANIZER_DOWNLOADS is selected (test_18)

- GIVEN `FILE_ORGANIZER_DOWNLOADS=/tmp/custom-downloads` and the
  directory exists
- WHEN `default_downloads_path` is called
- THEN the result is `/tmp/custom-downloads` (or its `~/`-expanded
  equivalent).
- Ref: `src/lib.rs::default_downloads_path` env-var branch
  (`src/lib.rs:504-511`).
- Test runner: `cargo test tests::default_downloads_path_honors_env_var -- --exact`.

### Requirement: Linux XDG_DOWNLOAD_DIR Resolution

On Linux, when `FILE_ORGANIZER_DOWNLOADS` is unset, the system MUST
read `XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs`. The parser
MUST strip surrounding double quotes and expand `$HOME`.

#### Scenario: Mocked user-dirs.dirs XDG value is expanded (test_19)

- GIVEN a temp HOME with `user-dirs.dirs` containing
  `XDG_DOWNLOAD_DIR="$HOME/Downloads"`
- AND `FILE_ORGANIZER_DOWNLOADS` is unset
- WHEN `default_downloads_path` runs on Linux
- THEN the result is `<HOME>/Downloads`
- AND no literal `"$HOME"` survives in the path.
- Ref: `src/lib.rs::default_downloads_path` Linux arm
  (`src/lib.rs:513-524`) and `src/lib.rs::read_xdg_download_dir`
  (`src/lib.rs:561-583`). Test gated
  `#[cfg(target_os = "linux")]`.
- Test runner: `cargo test tests::default_downloads_path_reads_xdg_user_dirs -- --exact`.

### Requirement: Platform Default Downloads

When no env var and no XDG value apply, the system MUST select
`~/Downloads` on macOS and `%USERPROFILE%/Downloads` on Windows.
Localized folder fallbacks (`Descargas`, `Téléchargements`,
`Scaricati`, `下载`) MUST be tried only on Windows when
`%USERPROFILE%/Downloads` does not exist.

#### Scenario: macOS returns ~/Downloads (test_20)

- GIVEN a macOS environment with HOME set and no overrides
- WHEN `default_downloads_path` runs
- THEN the result is `<HOME>/Downloads`.
- Ref: `src/lib.rs::default_downloads_path` macOS arm
  (`src/lib.rs:530-537`). Test gated
  `#[cfg(target_os = "macos")]`.
- Test runner: `cargo test tests::default_downloads_path_returns_home_downloads -- --exact`.

#### Scenario: Windows selects %USERPROFILE%/Downloads first (test_21)

- GIVEN a Windows environment with `%USERPROFILE%` set and the
  directory exists
- WHEN `default_downloads_path` runs
- THEN the result is `%USERPROFILE%/Downloads`.
- Ref: `src/lib.rs::default_downloads_path` Windows primary arm
  (`src/lib.rs:539-544`). Test gated
  `#[cfg(target_os = "windows")]`.
- Test runner: `cargo test tests::default_downloads_path_selects_userprofile_downloads_first -- --exact`.

#### Scenario: Windows localized fallback is used when primary missing (test_22)

- GIVEN a Windows environment with `%USERPROFILE%` set BUT
  `%USERPROFILE%/Downloads` does not exist
- AND a localized folder (e.g. `Descargas`) exists under
  `%USERPROFILE%`
- WHEN `default_downloads_path` runs
- THEN the first localized fallback that exists is returned.
- Ref: `src/lib.rs::default_downloads_path` Windows fallback list
  (`src/lib.rs:545-550`,
  `["Descargas","Téléchargements","Scaricati","下载"]` walked in
  order). Test gated `#[cfg(target_os = "windows")]`.
- Test runner: `cargo test tests::default_downloads_path_uses_localized_fallback -- --exact`.
