//! Localization module using Fluent

use fluent::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;
use tracing::{error, warn};
use unic_langid::LanguageIdentifier;

/// Localization manager
pub struct Localization {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    current_language: String,
}

impl Default for Localization {
    fn default() -> Self {
        Self::new("en")
    }
}

impl Localization {
    pub fn new(language: &str) -> Self {
        let mut loc = Self {
            bundles: HashMap::new(),
            current_language: language.to_string(),
        };

        // Load all language bundles
        loc.load_language("en", include_str!("locales/en.ftl"));
        loc.load_language("es", include_str!("locales/es.ftl"));
        loc.load_language("fr", include_str!("locales/fr.ftl"));
        loc.load_language("de", include_str!("locales/de.ftl"));
        loc.load_language("ja", include_str!("locales/ja.ftl"));
        loc.load_language("zh", include_str!("locales/zh.ftl"));

        // Ensure current language exists, otherwise fall back to English
        if !loc.bundles.contains_key(&loc.current_language) {
            warn!(
                "Requested language '{}' not available, falling back to English",
                loc.current_language
            );
            loc.current_language = "en".to_string();
        }

        loc
    }

    fn load_language(&mut self, code: &str, ftl_content: &str) {
        let lang_id: LanguageIdentifier = match code.parse() {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid language code '{}': {}", code, e);
                return;
            }
        };

        let resource = match FluentResource::try_new(ftl_content.to_string()) {
            Ok(res) => res,
            Err((res, errors)) => {
                // Fluent returns partial resource even on error
                for err in &errors {
                    warn!("FTL parse error in '{}': {:?}", code, err);
                }
                res
            }
        };

        let mut bundle = FluentBundle::new(vec![lang_id]);
        if let Err(errors) = bundle.add_resource(resource) {
            for err in errors {
                error!("Failed to add resource for '{}': {:?}", code, err);
            }
        }

        self.bundles.insert(code.to_string(), bundle);
    }

    pub fn set_language(&mut self, language: &str) {
        if self.bundles.contains_key(language) {
            self.current_language = language.to_string();
        } else {
            warn!(
                "Language '{}' not available, keeping current language '{}'",
                language, self.current_language
            );
        }
    }

    /// Get a translated string
    pub fn get(&self, key: &str) -> String {
        self.get_with_args(key, None)
    }

    /// Get a translated string with arguments
    pub fn get_with_args(&self, key: &str, args: Option<&FluentArgs>) -> String {
        // Try current language first
        if let Some(result) = self.try_get_from_bundle(&self.current_language, key, args) {
            return result;
        }

        // Fallback to English if not current language
        if self.current_language != "en" {
            if let Some(result) = self.try_get_from_bundle("en", key, args) {
                warn!(
                    "Key '{}' not found in '{}', using English fallback",
                    key, self.current_language
                );
                return result;
            }
        }

        // Return key as final fallback
        warn!("Translation key '{}' not found in any language", key);
        key.to_string()
    }

    fn try_get_from_bundle(
        &self,
        lang: &str,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Option<String> {
        let bundle = self.bundles.get(lang)?;
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;

        let mut errors = vec![];
        let result = bundle.format_pattern(pattern, args, &mut errors);

        if !errors.is_empty() {
            for err in &errors {
                warn!("Format error for key '{}' in '{}': {:?}", key, lang, err);
            }
        }

        Some(result.to_string())
    }

    /// Convenience method for getting string with a single string argument
    pub fn get_with_arg(&self, key: &str, arg_name: &str, arg_value: &str) -> String {
        let mut args = FluentArgs::new();
        args.set(arg_name, FluentValue::from(arg_value));
        self.get_with_args(key, Some(&args))
    }

    /// Convenience method for getting string with a single number argument
    pub fn get_with_count(&self, key: &str, count: i64) -> String {
        let mut args = FluentArgs::new();
        args.set("count", FluentValue::from(count));
        self.get_with_args(key, Some(&args))
    }
}

// Translation keys constants for type safety
pub mod keys {
    // App
    pub const APP_TITLE: &str = "app-title";

    // Buttons
    pub const BTN_CANCEL: &str = "btn-cancel";
    pub const BTN_MOVE: &str = "btn-move";
    pub const BTN_YES: &str = "btn-yes";
    pub const BTN_NO: &str = "btn-no";

    // Tooltips
    pub const TOOLTIP_LASSO: &str = "tooltip-lasso";
    pub const TOOLTIP_REFRESH: &str = "tooltip-refresh";
    pub const TOOLTIP_SETTINGS: &str = "tooltip-settings";
    pub const TOOLTIP_BACK: &str = "tooltip-back";

    // Window list
    pub const WINDOWS_EMPTY: &str = "windows-empty";
    pub const WINDOWS_OFFSCREEN: &str = "windows-offscreen";
    pub const WINDOWS_MINIMIZED: &str = "windows-minimized";
    pub const WINDOWS_COUNT: &str = "windows-count";

    // Monitor picker
    pub const MONITOR_TITLE: &str = "monitor-title";
    pub const MONITOR_SELECT: &str = "monitor-select";
    pub const MONITOR_PRIMARY: &str = "monitor-primary";
    pub const MONITOR_RESOLUTION: &str = "monitor-resolution";

    // Settings
    pub const SETTINGS_TITLE: &str = "settings-title";
    pub const SETTINGS_LANGUAGE: &str = "settings-language";
    pub const SETTINGS_BEHAVIOR: &str = "settings-behavior";
    pub const SETTINGS_AUTO_FOCUS: &str = "settings-auto-focus";
    pub const SETTINGS_CLOSE_AFTER_RECOVERY: &str = "settings-close-after-recovery";
    pub const SETTINGS_HOTKEYS: &str = "settings-hotkeys";
    pub const SETTINGS_TRAY: &str = "settings-tray";

    // Hotkeys
    pub const HOTKEY_LASSO: &str = "hotkey-lasso";
    pub const HOTKEY_REFRESH: &str = "hotkey-refresh";
    pub const HOTKEY_PRIMARY: &str = "hotkey-primary";
    pub const HOTKEY_ALL_PRIMARY: &str = "hotkey-all-primary";
    pub const HOTKEY_CENTER: &str = "hotkey-center";
    pub const HOTKEY_NEXT_MONITOR: &str = "hotkey-next-monitor";
    pub const HOTKEY_EDIT: &str = "hotkey-edit";
    pub const HOTKEY_PRESS: &str = "hotkey-press";

    // Dialogs
    pub const DIALOG_TRAY_TITLE: &str = "dialog-tray-title";
    pub const DIALOG_TRAY_MESSAGE: &str = "dialog-tray-message";

    // Status
    pub const STATUS_MOVED: &str = "status-moved";
    pub const STATUS_ERROR: &str = "status-error";
    pub const STATUS_REFRESHED: &str = "status-refreshed";
}
