rust_i18n::i18n!("locales");

/// Initialize the locale based on CLI flag, config, or system detection.
///
/// Only `en` and `es` are supported. Values outside that contract fall back
/// to the next source in precedence, ending with English.
pub fn init_locale(cli_lang: Option<&str>, config_lang: Option<&str>) {
    let locale = cli_lang
        .filter(|lang| is_supported_language(lang))
        .or_else(|| config_lang.filter(|lang| is_supported_language(lang)))
        .or_else(|| detect_system_locale())
        .unwrap_or("en");

    rust_i18n::set_locale(locale);
}

/// True for the two supported interface languages.
pub fn is_supported_language(lang: &str) -> bool {
    matches!(lang, "en" | "es")
}

/// Extract the `language` field from a config file without validating the
/// rest of it. Used at startup so a configured language like "es" survives
/// even when other fields in the TOML are invalid.
pub fn preflight_config_language(path: &std::path::Path) -> Option<String> {
    let source = std::fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&source).ok()?;
    value
        .get("language")
        .and_then(|field| field.as_str())
        .map(str::to_string)
}

/// Detect system locale from environment variables.
fn detect_system_locale() -> Option<&'static str> {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANGUAGE"))
        .ok()
        .and_then(|locale| {
            // Extract language code: es_ES.UTF-8 -> es, es-ES.UTF-8 -> es,
            // en:es:fr -> en. Locale prefixes use '_' (POSIX) or '-' (BCP 47).
            let lang = locale
                .split(':') // Handle LANGUAGE format (en:es:fr)
                .next()?
                .split('_') // Handle locale format (es_ES.UTF-8)
                .next()?
                .split('-') // Handle BCP 47 format (es-ES.UTF-8)
                .next()?
                .split('.') // Remove encoding (.UTF-8)
                .next()?;
            match lang {
                "en" => Some("en"),
                "es" => Some("es"),
                _ => None,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captures LANG/LC_ALL/LANGUAGE, mutates them inside the test, and
    /// restores the originals on drop — including during assertion panics.
    struct EnvGuard {
        lang: Option<std::ffi::OsString>,
        lc_all: Option<std::ffi::OsString>,
        language: Option<std::ffi::OsString>,
        _env_lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn with(mutations: impl FnOnce(&mut EnvVars)) -> Self {
            let _env_lock = crate::ENV_LOCK
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            let guard = Self {
                lang: std::env::var_os("LANG"),
                lc_all: std::env::var_os("LC_ALL"),
                language: std::env::var_os("LANGUAGE"),
                _env_lock,
            };
            let mut vars = EnvVars;
            mutations(&mut vars);
            guard
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            restore_var("LANG", self.lang.as_deref());
            restore_var("LC_ALL", self.lc_all.as_deref());
            restore_var("LANGUAGE", self.language.as_deref());
        }
    }

    struct EnvVars;

    impl EnvVars {
        fn set(&self, key: &str, value: &str) {
            std::env::set_var(key, value);
        }

        fn remove(&self, key: &str) {
            std::env::remove_var(key);
        }
    }

    fn restore_var(key: &str, value: Option<&std::ffi::OsStr>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn detects_spanish_from_lang_env() {
        let _guard = EnvGuard::with(|vars| {
            vars.set("LANG", "es_ES.UTF-8");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), Some("es"));
    }

    #[test]
    fn detects_english_from_lang_env() {
        let _guard = EnvGuard::with(|vars| {
            vars.set("LANG", "en_US.UTF-8");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), Some("en"));
    }

    #[test]
    fn detects_spanish_from_hyphenated_lang_env() {
        let _guard = EnvGuard::with(|vars| {
            vars.set("LANG", "es-ES.UTF-8");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), Some("es"));
    }

    #[test]
    fn detects_english_from_hyphenated_lang_env() {
        let _guard = EnvGuard::with(|vars| {
            vars.set("LANG", "en-US.UTF-8");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), Some("en"));
    }

    #[test]
    fn returns_none_for_unsupported_locale() {
        let _guard = EnvGuard::with(|vars| {
            vars.set("LANG", "fr_FR.UTF-8");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), None);
    }

    #[test]
    fn detects_from_lc_all_when_lang_missing() {
        let _guard = EnvGuard::with(|vars| {
            vars.remove("LANG");
            vars.set("LC_ALL", "es_MX.UTF-8");
            vars.remove("LANGUAGE");
        });
        assert_eq!(detect_system_locale(), Some("es"));
    }

    #[test]
    fn detects_from_language_as_fallback() {
        let _guard = EnvGuard::with(|vars| {
            vars.remove("LANG");
            vars.remove("LC_ALL");
            vars.set("LANGUAGE", "en:es:fr");
        });
        assert_eq!(detect_system_locale(), Some("en"));
    }

    #[test]
    fn init_locale_accepts_supported_languages() {
        let _guard = EnvGuard::with(|vars| {
            vars.remove("LANG");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        init_locale(Some("es"), None);
        assert_eq!(&*rust_i18n::locale(), "es");
        init_locale(Some("en"), None);
        assert_eq!(&*rust_i18n::locale(), "en");
    }

    #[test]
    fn init_locale_ignores_unsupported_cli_language() {
        let _guard = EnvGuard::with(|vars| {
            vars.remove("LANG");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        // "fr" is outside the en|es contract: falls through to config, then
        // system (empty here), then the English default.
        init_locale(Some("fr"), Some("es"));
        assert_eq!(&*rust_i18n::locale(), "es");

        init_locale(Some("fr"), None);
        assert_eq!(&*rust_i18n::locale(), "en");
    }

    #[test]
    fn init_locale_ignores_unsupported_config_language() {
        let _guard = EnvGuard::with(|vars| {
            vars.remove("LANG");
            vars.remove("LC_ALL");
            vars.remove("LANGUAGE");
        });
        init_locale(None, Some("de"));
        assert_eq!(&*rust_i18n::locale(), "en");
    }

    #[test]
    fn preflight_extracts_language_from_valid_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        std::fs::write(&config_path, "language = \"es\"\nmin_age_seconds = 5\n").unwrap();
        assert_eq!(
            preflight_config_language(&config_path).as_deref(),
            Some("es")
        );
    }

    #[test]
    fn preflight_extracts_language_even_when_other_fields_are_invalid() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        // Invalid: extensions empty category. load_config would fail, but the
        // preflight parser must still surface the configured language.
        std::fs::write(&config_path, "language = \"es\"\n[extensions]\nmd = \"\"\n").unwrap();
        assert_eq!(
            preflight_config_language(&config_path).as_deref(),
            Some("es")
        );
    }

    #[test]
    fn preflight_returns_none_for_missing_or_invalid_config() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing.toml");
        assert_eq!(preflight_config_language(&missing), None);

        let invalid = temp.path().join("invalid.toml");
        std::fs::write(&invalid, "not toml {{{\n").unwrap();
        assert_eq!(preflight_config_language(&invalid), None);
    }
}
