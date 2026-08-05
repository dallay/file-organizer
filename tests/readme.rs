//! Regression tests for the Spanish → English translation of `README.md`.
//!
//! This suite pins down the exact content changes made when the README was
//! translated to English: the new English headings/paragraphs must be
//! present, the old Spanish text must be gone, the code samples that were
//! *not* supposed to change must still be intact, and the `YOUR_USERNAME`
//! placeholder introduced by the translation must actually match the
//! template file it documents.

use std::fs;
use std::path::Path;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn readme() -> String {
    let path = repo_root().join("README.md");
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// Headings that the translation introduced.
const NEW_ENGLISH_HEADINGS: &[&str] = &[
    "## Install",
    "## Compile",
    "## Configure",
    "## Run",
    "## Platform automation",
    "### macOS with launchd",
    "### Linux with systemd user",
    "### Windows with Task Scheduler",
    "## Current scope",
];

/// Headings that existed in the Spanish version and must not linger.
const OLD_SPANISH_HEADINGS: &[&str] = &[
    "## Instalar",
    "## Compilar",
    "## Configurar",
    "## Ejecutar",
    "## Automatización por plataforma",
    "### macOS con launchd",
    "### Linux con systemd user",
    "### Windows con Task Scheduler",
    "## Alcance actual",
];

/// Spanish sentences/fragments from the pre-translation README that should
/// have been fully replaced by their English counterparts.
const OLD_SPANISH_PHRASES: &[&str] = &[
    "Motor multiplataforma para macOS, Linux y Windows",
    "La lógica de clasificación vive en un binario Rust",
    "Hay tres canales de instalación",
    "wrapper con binarios por plataforma",
    "multi-arquitectura linux/amd64",
    "También puedes descargar el binario de tu plataforma",
    "Requiere Rust estable",
    "El binario queda en",
    "Las instrucciones de **Configurar**",
    "En Windows, copia el archivo a",
    "Las carpetas indicadas al final sustituyen",
    "usa Shortcuts o `launchd`",
    "usa el `systemd` user timer",
    "usa Task Scheduler con `schtasks`",
    "El lock se crea con `create_dir`",
    "Para detenerlo:",
    "Después de copiar",
    "El movimiento utiliza `rename`",
    "TU_USUARIO",
];

#[test]
fn readme_contains_new_english_headings() {
    let readme = readme();
    for heading in NEW_ENGLISH_HEADINGS {
        assert!(
            readme.contains(heading),
            "expected README.md to contain the English heading {heading:?}"
        );
    }
}

#[test]
fn readme_does_not_contain_old_spanish_headings() {
    let readme = readme();
    for heading in OLD_SPANISH_HEADINGS {
        assert!(
            !readme.contains(heading),
            "README.md still contains the old Spanish heading {heading:?}"
        );
    }
}

#[test]
fn readme_does_not_contain_leftover_spanish_phrases() {
    let readme = readme();
    for phrase in OLD_SPANISH_PHRASES {
        assert!(
            !readme.contains(phrase),
            "README.md still contains untranslated Spanish text: {phrase:?}"
        );
    }
}

#[test]
fn readme_intro_paragraph_is_translated() {
    let readme = readme();
    assert!(readme.contains(
        "Multiplatform engine for macOS, Linux, and Windows. The classification \
         logic lives in a Rust binary; the schedulers of each operating system \
         only execute it."
    ));
}

#[test]
fn readme_current_scope_paragraph_is_translated() {
    let readme = readme();
    assert!(readme.contains(
        "The move operation uses `rename`, which is atomic within the same volume. \
         If the source and destination are on different volumes, an error is \
         reported instead of partially copying the file."
    ));
}

#[test]
fn readme_configure_windows_instructions_are_translated() {
    let readme = readme();
    assert!(readme.contains(
        "On Windows, copy the file to `%APPDATA%\\\\organiza\\\\config.toml`. Edit the folders and run:"
    ));
}

/// The install/compile/configure/run/current-scope commands themselves were
/// not part of the translation and must remain byte-for-byte identical.
#[test]
fn readme_code_samples_are_unchanged() {
    let readme = readme();
    for snippet in [
        "cargo install organiza",
        "npm install -g @dallay/organiza",
        "docker pull yacosta738/organiza",
        "docker pull ghcr.io/dallay/organiza",
        "cargo test\ncargo build --release",
        "mkdir -p \"$HOME/.config/organiza\"\ncp config.example.toml \"$HOME/.config/organiza/config.toml\"",
        "organiza --config ~/.config/organiza/config.toml validate-config",
        "organiza run --dry-run\norganiza run\norganiza run --verbose ~/Downloads\norganiza run --config ./config.toml --log /dev/null",
        "mkdir -p \"$HOME/.local/bin\" \"$HOME/.config/systemd/user\"",
        "systemctl --user daemon-reload\nsystemctl --user enable --now organiza.timer",
    ] {
        assert!(
            readme.contains(snippet),
            "expected README.md to still contain the unmodified code sample {snippet:?}"
        );
    }
}

/// Links referenced by the README must survive translation untouched.
#[test]
fn readme_links_are_unchanged() {
    let readme = readme();
    assert!(readme.contains("[GitHub Release](https://github.com/dallay/file-organizer/releases)"));
    assert!(readme.contains("[the repository on GitHub](https://github.com/dallay/file-organizer)"));
}

/// Both platform-specific setup snippets now use the English placeholder.
#[test]
fn readme_uses_your_username_placeholder() {
    let readme = readme();
    assert!(readme
        .contains(r#"sed "s#YOUR_USERNAME#$(whoami)#" platform/macos/com.organiza.plist.example"#));
    assert!(readme.contains(r"C:\Users\YOUR_USERNAME\.local\bin\organiza.exe run"));
    assert!(!readme.contains("TU_USUARIO"));
}

/// Regression test: the translation renamed the `sed` placeholder documented
/// for the macOS `launchd` setup from `TU_USUARIO` to `YOUR_USERNAME`, but
/// did not update `platform/macos/com.organiza.plist.example`, which still
/// contains the literal `TU_USUARIO` token. If this test fails, the
/// documented command is broken: `sed "s#YOUR_USERNAME#$(whoami)#"` will not
/// match anything in the template, so the generated LaunchAgent plist will
/// keep the placeholder verbatim instead of the real username.
#[test]
fn readme_launchd_sed_placeholder_matches_plist_template() {
    let readme = readme();
    let plist_path = repo_root().join("platform/macos/com.organiza.plist.example");
    let plist = fs::read_to_string(&plist_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", plist_path.display()));

    let documented_placeholder = "YOUR_USERNAME";
    assert!(
        readme.contains(documented_placeholder),
        "README.md no longer documents the {documented_placeholder:?} placeholder \
         for the macOS launchd `sed` command"
    );
    assert!(
        plist.contains(documented_placeholder),
        "README.md documents `sed \"s#{documented_placeholder}#$(whoami)#\"` for \
         platform/macos/com.organiza.plist.example, but that file does not contain \
         the placeholder {documented_placeholder:?} (it still has the old \
         `TU_USUARIO` token). The documented command will silently fail to \
         substitute the username."
    );
}

/// Guards against accidentally reordering sections while translating them.
#[test]
fn readme_top_level_headings_preserve_original_order() {
    let readme = readme();
    let expected_order = [
        "## Install",
        "## Compile",
        "## Configure",
        "## Run",
        "## Default categories",
        "## Customizing categories",
        "## First-run behavior",
        "## Platform automation",
        "## Current scope",
        "## Development tooling",
    ];

    let mut last_index: isize = -1;
    for heading in expected_order {
        let index = readme
            .find(heading)
            .unwrap_or_else(|| panic!("expected README.md to contain heading {heading:?}"))
            as isize;
        assert!(
            index > last_index,
            "heading {heading:?} appears out of order in README.md"
        );
        last_index = index;
    }
}

/// Broad guard against reintroducing Spanish text in any heading line.
#[test]
fn readme_headings_contain_no_spanish_diacritics() {
    let readme = readme();
    let spanish_chars = [
        'á', 'é', 'í', 'ó', 'ú', 'ñ', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ñ', '¿', '¡',
    ];

    for line in readme.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            assert!(
                !trimmed.chars().any(|c| spanish_chars.contains(&c)),
                "heading line {trimmed:?} appears to still contain Spanish text"
            );
        }
    }
}
