rust_i18n::i18n!("locales");

/// Initialize the locale based on CLI flag, config, or system detection.
pub fn init_locale(cli_lang: Option<&str>, config_lang: Option<&str>) {
    let locale = cli_lang
        .or(config_lang)
        .or_else(|| detect_system_locale())
        .unwrap_or("en");

    rust_i18n::set_locale(locale);
}

/// Detect system locale from environment variables.
fn detect_system_locale() -> Option<&'static str> {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANGUAGE"))
        .ok()
        .and_then(|locale| {
            // Extract language code (es_ES.UTF-8 -> es, en:es:fr -> en)
            let lang = locale
                .split(':') // Handle LANGUAGE format (en:es:fr)
                .next()?
                .split('_') // Handle locale format (es_ES.UTF-8)
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

    #[test]
    fn detects_spanish_from_lang_env() {
        std::env::set_var("LANG", "es_ES.UTF-8");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LANGUAGE");
        assert_eq!(detect_system_locale(), Some("es"));
    }

    #[test]
    fn detects_english_from_lang_env() {
        std::env::set_var("LANG", "en_US.UTF-8");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LANGUAGE");
        assert_eq!(detect_system_locale(), Some("en"));
    }

    #[test]
    fn returns_none_for_unsupported_locale() {
        std::env::set_var("LANG", "fr_FR.UTF-8");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LANGUAGE");
        assert_eq!(detect_system_locale(), None);
    }

    #[test]
    fn detects_from_lc_all_when_lang_missing() {
        std::env::remove_var("LANG");
        std::env::set_var("LC_ALL", "es_MX.UTF-8");
        std::env::remove_var("LANGUAGE");
        assert_eq!(detect_system_locale(), Some("es"));
    }

    #[test]
    fn detects_from_language_as_fallback() {
        std::env::remove_var("LANG");
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANGUAGE", "en:es:fr");
        assert_eq!(detect_system_locale(), Some("en"));
    }
}
