# Delta Spec: downloads-autodetect

**Change**: configurable-categories-and-sources
**Status**: Proposed
**RFC 2119**: All requirements use MUST / SHALL / SHOULD / MAY per `openspec/config.yaml::rules.specs`.

## Purpose

Defines how the binary selects a Downloads source directory when no
`source_directories` is configured and no CLI positional directories are
given. The lookup order, locked in `intent.md`, is: `FILE_ORGANIZER_DOWNLOADS`
→ Linux `XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs` → macOS
`~/Downloads` → Windows `%USERPROFILE%/Downloads` with localized fallbacks
(`Descargas`, `Téléchargements`, `Scaricati`, `下载`).

## ADDED Requirements

### Requirement: Environment Variable Override

The system MUST honor the `FILE_ORGANIZER_DOWNLOADS` environment variable
when set and non-empty. The variable value MUST be `expand_home`-expanded
before use. No other auto-detection step MUST run when this variable is
present.

#### Scenario: FILE_ORGANIZER_DOWNLOADS is selected (test_18)

- GIVEN `FILE_ORGANIZER_DOWNLOADS=/tmp/custom-downloads` and the directory
  exists
- WHEN `default_downloads_path` is called
- THEN the result is `/tmp/custom-downloads` (or its `~/`-expanded
  equivalent)
- Ref: new `src/lib.rs::default_downloads_path`; test at
  `src/lib.rs::tests::test_18`.

### Requirement: Linux XDG_DOWNLOAD_DIR Resolution

On Linux, when `FILE_ORGANIZER_DOWNLOADS` is unset, the system MUST read
`XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs`. The parser MUST strip
surrounding double quotes and expand `$HOME`.

#### Scenario: Mocked user-dirs.dirs XDG value is expanded (test_19)

- GIVEN a temp HOME with `user-dirs.dirs` containing
  `XDG_DOWNLOAD_DIR="$HOME/Downloads"`
- AND `FILE_ORGANIZER_DOWNLOADS` is unset
- WHEN `default_downloads_path` runs on Linux
- THEN the result is `<HOME>/Downloads`
- AND no literal `"$HOME"` survives in the path
- Ref: new `src/lib.rs::read_xdg_download_dir`; test at
  `src/lib.rs::tests::test_19`. Test gated `cfg!(target_os = "linux")`.

### Requirement: Platform Default Downloads

When no env var and no XDG value apply, the system MUST select
`~/Downloads` on macOS, `%USERPROFILE%/Downloads` on Windows. Localized
folder fallbacks (`Descargas`, `Téléchargements`, `Scaricati`, `下载`)
MUST be tried only on Windows when `%USERPROFILE%/Downloads` does not
exist.

#### Scenario: macOS returns ~/Downloads (test_20)

- GIVEN a macOS environment with HOME set and no overrides
- WHEN `default_downloads_path` runs
- THEN the result is `<HOME>/Downloads`
- Ref: new `src/lib.rs::default_downloads_path` (macOS arm); test at
  `src/lib.rs::tests::test_20`. Test gated `cfg!(target_os = "macos")`.

#### Scenario: Windows selects %USERPROFILE%/Downloads first (test_21)

- GIVEN a Windows environment with `%USERPROFILE%` set and the directory
  exists
- WHEN `default_downloads_path` runs
- THEN the result is `%USERPROFILE%/Downloads`
- Ref: new `src/lib.rs::default_downloads_path` (Windows arm); test at
  `src/lib.rs::tests::test_21`. Test gated `cfg!(windows)`.

#### Scenario: Windows localized fallback is used when primary missing (test_22)

- GIVEN a Windows environment with `%USERPROFILE%` set BUT
  `%USERPROFILE%/Downloads` does not exist
- AND a localized folder (e.g. `Descargas`) exists under `%USERPROFILE%`
- WHEN `default_downloads_path` runs
- THEN the first localized fallback that exists is returned
- Ref: new `src/lib.rs::default_downloads_path` (Windows fallback list);
  test at `src/lib.rs::tests::test_22`. Test gated `cfg!(windows)`.
