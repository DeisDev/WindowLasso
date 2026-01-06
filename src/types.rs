//! Shared type definitions for WindowLasso

use serde::{Deserialize, Serialize};

/// Application version (read from Cargo.toml at compile time)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository URL
pub const GITHUB_URL: &str = "https://github.com/DeisDev/WindowLasso";

/// GitHub issues URL
pub const ISSUES_URL: &str = "https://github.com/DeisDev/WindowLasso/issues";

/// Information about a window
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub rect: WindowRect,
    pub is_visible: bool,
    pub is_offscreen: bool,
    pub is_minimized: bool,
    pub monitor_name: Option<String>,
    pub icon_rgba: Option<Vec<u8>>,
    pub icon_size: u32,
}

/// Window rectangle/bounds
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindowRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl WindowRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn center(&self) -> (i32, i32) {
        (self.left + self.width() / 2, self.top + self.height() / 2)
    }

    pub fn intersects(&self, other: &WindowRect) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }
}

/// Information about a monitor/display
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub handle: isize,
    pub name: String,
    pub device_name: String,
    pub bounds: WindowRect,
    pub work_area: WindowRect,
    pub is_primary: bool,
    pub display_index: usize,
}

impl MonitorInfo {
    pub fn center(&self) -> (i32, i32) {
        self.work_area.center()
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub minimize_to_tray: Option<bool>,
    pub auto_focus_after_lasso: bool,
    #[serde(default)]
    pub close_after_recovery: bool,
    pub hotkeys: HotkeySettings,
    pub theme: ThemeSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            minimize_to_tray: None, // None = not yet asked
            auto_focus_after_lasso: true,
            close_after_recovery: false,
            hotkeys: HotkeySettings::default(),
            theme: ThemeSettings::default(),
        }
    }
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub lasso_window: HotkeyBinding,
    pub refresh_windows: HotkeyBinding,
    pub move_to_primary: HotkeyBinding,
    #[serde(default = "default_move_all_to_primary")]
    pub move_all_to_primary: HotkeyBinding,
    #[serde(default = "default_center_window")]
    pub center_window: HotkeyBinding,
    #[serde(default = "default_next_monitor")]
    pub next_monitor: HotkeyBinding,
}

fn default_move_all_to_primary() -> HotkeyBinding {
    HotkeyBinding {
        modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
        key: "A".to_string(),
        enabled: true,
    }
}

fn default_center_window() -> HotkeyBinding {
    HotkeyBinding {
        modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
        key: "C".to_string(),
        enabled: true,
    }
}

fn default_next_monitor() -> HotkeyBinding {
    HotkeyBinding {
        modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
        key: "N".to_string(),
        enabled: true,
    }
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            lasso_window: HotkeyBinding {
                modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
                key: "L".to_string(),
                enabled: true,
            },
            refresh_windows: HotkeyBinding {
                modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
                key: "R".to_string(),
                enabled: true,
            },
            move_to_primary: HotkeyBinding {
                modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
                key: "P".to_string(),
                enabled: true,
            },
            move_all_to_primary: default_move_all_to_primary(),
            center_window: default_center_window(),
            next_monitor: default_next_monitor(),
        }
    }
}

/// A single hotkey binding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HotkeyBinding {
    pub modifiers: Vec<String>,
    pub key: String,
    pub enabled: bool,
}

impl HotkeyBinding {
    pub fn display_string(&self) -> String {
        let mut parts = self.modifiers.clone();
        parts.push(self.key.clone());
        parts.join(" + ")
    }
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub dark_mode: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

/// Application screen/view state
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Screen {
    #[default]
    Main,
    MonitorPicker {
        selected_window: WindowInfo,
    },
    Settings,
}

/// Actions that can have hotkeys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    LassoWindow,
    RefreshWindows,
    MoveToPrimary,
    MoveAllToPrimary,
    CenterWindow,
    NextMonitor,
}

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Japanese,
    Chinese,
}

impl Language {
    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Japanese,
            Language::Chinese,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Japanese => "ja",
            Language::Chinese => "zh",
        }
    }

    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Japanese => "日本語",
            Language::Chinese => "中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            "de" => Some(Language::German),
            "ja" => Some(Language::Japanese),
            "zh" => Some(Language::Chinese),
            _ => None,
        }
    }
}
