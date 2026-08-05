//! Classification rules: built-in defaults, user `[[categories]]` resolution,
//! and the `is_generated_category` set used by the recursive-scan guard and
//! the depth-1 directory movement.
//!
//! Public surface lives behind `pub(crate)` so `lib.rs` stays the only module
//! `main.rs` talks to.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Config;

/// Fallback for unknown extensions AND no-extension files per
/// `specs/classification/spec.md` Requirement "Default Category Set".
pub(crate) const DEFAULT_CATEGORY: &str = "Other";

/// User-supplied category declaration. Order = TOML declaration order.
/// `replace = false` (default) supplements; `replace = true` substitutes the
/// built-in list for that category name.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub struct CategoryRule {
    pub name: String,
    pub extensions: Vec<String>,
    #[serde(default)]
    pub replace: bool,
}

/// Built-in flat 7-name table: extension (lowercase, ASCII) → category name.
pub(crate) fn default_categories() -> HashMap<&'static str, &'static str> {
    let entries: &[(&'static str, &'static str)] = &[
        // Image
        ("jpg", "Image"),
        ("jpeg", "Image"),
        ("png", "Image"),
        ("gif", "Image"),
        ("webp", "Image"),
        ("heic", "Image"),
        ("svg", "Image"),
        ("tiff", "Image"),
        ("tif", "Image"),
        // Video
        ("mp4", "Video"),
        ("mov", "Video"),
        ("mkv", "Video"),
        ("avi", "Video"),
        ("webm", "Video"),
        ("m4v", "Video"),
        // Audio
        ("mp3", "Audio"),
        ("m4a", "Audio"),
        ("wav", "Audio"),
        ("flac", "Audio"),
        ("ogg", "Audio"),
        ("aac", "Audio"),
        // Text (includes xls and ppt per intent.md lock)
        ("txt", "Text"),
        ("md", "Text"),
        ("rtf", "Text"),
        ("doc", "Text"),
        ("docx", "Text"),
        ("pages", "Text"),
        ("pdf", "Text"),
        ("xlsx", "Text"),
        ("xls", "Text"),
        ("pptx", "Text"),
        ("ppt", "Text"),
        ("key", "Text"),
        ("numbers", "Text"),
        ("csv", "Text"),
        ("epub", "Text"),
        ("odt", "Text"),
        ("ods", "Text"),
        ("odp", "Text"),
        ("log", "Text"),
        ("tex", "Text"),
        // Executable
        ("dmg", "Executable"),
        ("pkg", "Executable"),
        ("msi", "Executable"),
        ("exe", "Executable"),
        ("deb", "Executable"),
        ("rpm", "Executable"),
        // Compressed
        ("zip", "Compressed"),
        ("rar", "Compressed"),
        ("7z", "Compressed"),
        ("tar", "Compressed"),
        ("gz", "Compressed"),
        ("bz2", "Compressed"),
        ("xz", "Compressed"),
    ];
    entries.iter().copied().collect()
}

/// Classify `path` against an already-composed map. Returns
/// `DEFAULT_CATEGORY` for unknown extension AND no-extension files.
///
/// Prefer this in hot loops: compose the map once via [`apply_categories`]
/// and reuse it, instead of calling [`category_for`] per path (which
/// recomposes the map on every call).
pub(crate) fn classify(path: &Path, composed: &HashMap<String, String>) -> String {
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return DEFAULT_CATEGORY.to_string();
    };
    let Some(extension) = filename.rsplit_once('.').map(|(_, extension)| extension) else {
        return DEFAULT_CATEGORY.to_string();
    };
    if extension.is_empty() {
        return DEFAULT_CATEGORY.to_string();
    }
    composed
        .get(&extension.to_ascii_lowercase())
        .cloned()
        .unwrap_or_else(|| DEFAULT_CATEGORY.to_string())
}

/// Classify `path`, composing the resolution map on the fly. Convenience
/// helper used by tests; production hot paths compose the map once via
/// [`apply_categories`] and call [`classify`] directly.
#[cfg(test)]
pub(crate) fn category_for(path: &Path, config: &Config) -> String {
    classify(path, &apply_categories(config))
}

/// One-shot composition of the resolution map. Resolution order:
/// built-ins → `[[categories]]` (with `replace` consumed) → `[extensions]`
/// last-write-wins.
pub(crate) fn apply_categories(config: &Config) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = default_categories()
        .into_iter()
        .map(|(ext, name)| (ext.to_string(), name.to_string()))
        .collect();

    for rule in &config.categories {
        if rule.replace {
            map.retain(|_, value| value != &rule.name);
        }
        for extension in &rule.extensions {
            map.insert(extension.to_ascii_lowercase(), rule.name.clone());
        }
    }

    for (extension, name) in &config.extensions {
        map.insert(extension.to_ascii_lowercase(), name.clone());
    }

    map
}

/// Membership for `is_generated_category`. Always contains the seven flat
/// English names; merged with `config.categories[*].name` so users can mark
/// their own directories as "do not re-enter".
pub(crate) fn is_generated_category_set(config: &Config) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    names.insert(DEFAULT_CATEGORY.to_string());
    for (_, name) in default_categories() {
        names.insert(name.to_string());
    }
    for rule in &config.categories {
        names.insert(rule.name.clone());
    }
    names
}

/// True if `path` equals `<root>/<name>` or starts with it, for any name in
/// `is_generated_category_set`. Used by `should_visit` and the in-place
/// pre-classify inside `collect_top_level_directories`.
pub(crate) fn is_generated_category(path: &Path, root: &Path, config: &Config) -> bool {
    let names = is_generated_category_set(config);
    names
        .into_iter()
        .map(|category| root.join(category))
        .any(|category_path| path == category_path || path.starts_with(category_path))
}

/// A category name is unsafe when it resolves outside the destination folder
/// on any platform: absolute paths and parent traversal are rejected.
/// Windows additionally treats drive-relative roots (`C:\...`, `\...`) and
/// backslash-separated traversal as unsafe.
pub(crate) fn is_unsafe_category_name(name: &str) -> bool {
    let path = Path::new(name);
    // Native absolute detection; `starts_with('/')` also catches POSIX-style
    // roots on Windows, where `Path::is_absolute` requires a drive letter.
    if path.is_absolute() || name.starts_with('/') {
        return true;
    }
    #[cfg(windows)]
    {
        // A leading backslash is rooted to the current drive on Windows.
        if name.starts_with('\\') {
            return true;
        }
        // Windows accepts both separators for parent traversal.
        name.split(|c| c == '/' || c == '\\')
            .any(|part| part == "..")
    }
    #[cfg(not(windows))]
    {
        name.split('/').any(|part| part == "..")
    }
}

/// Per-rule validation. Called from `lib::validate_config`.
pub(crate) fn validate_categories(rules: &[CategoryRule]) -> anyhow::Result<()> {
    use std::collections::HashSet;
    let mut seen_names: HashSet<String> = HashSet::new();
    for rule in rules {
        if rule.name.trim().is_empty() {
            anyhow::bail!("category name cannot be empty");
        }
        if is_unsafe_category_name(&rule.name) {
            anyhow::bail!(
                "category name '{}' is not a safe folder name (absolute path or '..' segment)",
                rule.name
            );
        }
        if rule.extensions.is_empty() {
            anyhow::bail!("category '{}' has empty extensions list", rule.name);
        }
        for extension in &rule.extensions {
            if extension.trim().is_empty() {
                anyhow::bail!("category '{}' contains an empty extension", rule.name);
            }
        }
        if !seen_names.insert(rule.name.clone()) {
            anyhow::bail!(
                "duplicate category name '{}' across [[categories]] rules",
                rule.name
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_set_has_seven_flat_english_names() {
        let config = Config::default();
        let names = is_generated_category_set(&config);
        for required in [
            "Text",
            "Other",
            "Executable",
            "Compressed",
            "Audio",
            "Video",
            "Image",
        ] {
            assert!(
                names.contains(required),
                "missing generated name: {}",
                required
            );
        }
    }

    #[test]
    fn apply_categories_supplement_adds_without_removing() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["foo".to_string(), "bar".to_string()],
            replace: false,
        });
        let map = apply_categories(&config);
        assert_eq!(map.get("foo"), Some(&"Text".to_string()));
        assert_eq!(map.get("bar"), Some(&"Text".to_string()));
        // Built-ins remain.
        assert_eq!(map.get("txt"), Some(&"Text".to_string()));
        assert_eq!(map.get("md"), Some(&"Text".to_string()));
        assert_eq!(map.get("pdf"), Some(&"Text".to_string()));
    }

    #[test]
    fn apply_categories_replace_substitutes_builtin_list() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["onlytxt".to_string()],
            replace: true,
        });
        let map = apply_categories(&config);
        assert_eq!(map.get("onlytxt"), Some(&"Text".to_string()));
        assert_eq!(map.get("txt"), None);
        assert_eq!(map.get("pdf"), None);
        assert_eq!(map.get("md"), None);
    }

    #[test]
    fn apply_categories_non_colliding_adds_untouched() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Design".to_string(),
            extensions: vec!["psd".to_string(), "ai".to_string()],
            replace: false,
        });
        let map = apply_categories(&config);
        assert_eq!(map.get("psd"), Some(&"Design".to_string()));
        assert_eq!(map.get("ai"), Some(&"Design".to_string()));
        assert_eq!(map.get("jpg"), Some(&"Image".to_string()));
        assert_eq!(map.get("txt"), Some(&"Text".to_string()));
    }

    #[test]
    fn apply_categories_extension_override_wins_after_rules() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["md".to_string()],
            replace: false,
        });
        config
            .extensions
            .insert("md".to_string(), "Docs".to_string());
        let map = apply_categories(&config);
        assert_eq!(map.get("md"), Some(&"Docs".to_string()));
    }

    #[test]
    fn validate_categories_rejects_absolute_or_parent_traversal() {
        let bad = vec![
            CategoryRule {
                name: "/etc/passwd".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
            CategoryRule {
                name: "../escape".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
            CategoryRule {
                name: "Sub/../Other".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
        ];
        for rule in bad {
            assert!(validate_categories(&[rule]).is_err());
        }
    }

    #[cfg(windows)]
    #[test]
    fn validate_categories_rejects_windows_absolute_and_backslash_traversal() {
        let bad = vec![
            CategoryRule {
                name: "C:\\Windows".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
            CategoryRule {
                name: "\\Windows".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
            CategoryRule {
                name: "..\\..\\etc".to_string(),
                extensions: vec!["x".to_string()],
                replace: false,
            },
        ];
        for rule in bad {
            assert!(validate_categories(&[rule]).is_err());
        }
    }

    #[test]
    fn validate_categories_rejects_empty_extensions() {
        let rule = CategoryRule {
            name: "EmptyCat".to_string(),
            extensions: vec![],
            replace: false,
        };
        assert!(validate_categories(&[rule]).is_err());
    }

    #[test]
    fn validate_categories_rejects_duplicate_names() {
        let rules = vec![
            CategoryRule {
                name: "Text".to_string(),
                extensions: vec!["a".to_string()],
                replace: false,
            },
            CategoryRule {
                name: "Text".to_string(),
                extensions: vec!["b".to_string()],
                replace: true,
            },
        ];
        assert!(validate_categories(&rules).is_err());
    }

    #[test]
    fn is_generated_category_recognizes_flat_names() {
        let config = Config::default();
        let root = Path::new("/tmp/root");
        assert!(is_generated_category(&root.join("Image"), root, &config));
        assert!(is_generated_category(
            &root.join("Image/nested.jpg"),
            root,
            &config
        ));
        assert!(is_generated_category(&root.join("Other"), root, &config));
        assert!(!is_generated_category(
            &root.join("Projects"),
            root,
            &config
        ));
        assert!(!is_generated_category(
            &root.join("Imágenes"),
            root,
            &config
        ));
    }

    #[test]
    fn classify_with_precomposed_map_matches_category_for() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Design".to_string(),
            extensions: vec!["psd".to_string(), "ai".to_string()],
            replace: false,
        });
        config
            .extensions
            .insert("md".to_string(), "Docs".to_string());

        let map = apply_categories(&config);
        assert_eq!(classify(Path::new("PHOTO.JPG"), &map), "Image");
        assert_eq!(classify(Path::new("photo.jpg"), &map), "Image");
        assert_eq!(classify(Path::new("resume.PDF"), &map), "Text");
        assert_eq!(classify(Path::new("design.psd"), &map), "Design");
        assert_eq!(classify(Path::new("notes.md"), &map), "Docs");
        assert_eq!(classify(Path::new("README"), &map), "Other");
        assert_eq!(classify(Path::new("data.custom"), &map), "Other");
        // Same observable results as the one-shot convenience wrapper.
        assert_eq!(
            classify(Path::new("design.psd"), &map),
            category_for(Path::new("design.psd"), &config)
        );
    }
}
