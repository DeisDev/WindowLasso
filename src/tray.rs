//! System tray integration using tray-icon

use std::sync::OnceLock;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

static TRAY_MENU_SHOW_ID: OnceLock<String> = OnceLock::new();
static TRAY_MENU_REFRESH_ID: OnceLock<String> = OnceLock::new();
static TRAY_MENU_SETTINGS_ID: OnceLock<String> = OnceLock::new();
static TRAY_MENU_EXIT_ID: OnceLock<String> = OnceLock::new();

/// Menu action from the tray
#[derive(Debug, Clone, PartialEq)]
pub enum TrayMenuAction {
    Show,
    Refresh,
    Settings,
    Exit,
}

/// Holds the tray icon (must be kept alive)
pub struct SystemTray {
    _icon: TrayIcon,
}

impl SystemTray {
    /// Create and show the system tray icon
    pub fn new(tooltip: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let icon = load_tray_icon()?;
        let menu = build_menu()?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(tooltip)
            .with_icon(icon)
            .build()?;

        Ok(Self { _icon: tray })
    }
}

fn load_tray_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../icons/app/windowlasso.ico");

    // Decode the ICO file
    let img = image::load_from_memory(icon_bytes)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    Icon::from_rgba(rgba.into_raw(), w, h).map_err(|e| e.into())
}

fn build_menu() -> Result<Menu, Box<dyn std::error::Error>> {
    let menu = Menu::new();

    let show_item = MenuItem::new("Show WindowLasso", true, None);
    let refresh_item = MenuItem::new("Refresh Windows", true, None);
    let settings_item = MenuItem::new("Settings", true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    // Store the IDs
    let _ = TRAY_MENU_SHOW_ID.set(show_item.id().0.clone());
    let _ = TRAY_MENU_REFRESH_ID.set(refresh_item.id().0.clone());
    let _ = TRAY_MENU_SETTINGS_ID.set(settings_item.id().0.clone());
    let _ = TRAY_MENU_EXIT_ID.set(exit_item.id().0.clone());

    menu.append(&show_item)?;
    menu.append(&refresh_item)?;
    menu.append(&settings_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&exit_item)?;

    Ok(menu)
}

/// Poll for tray icon click events (returns true if double-clicked)
pub fn poll_tray_click() -> Option<bool> {
    if let Ok(event) = TrayIconEvent::receiver().try_recv() {
        match event {
            TrayIconEvent::DoubleClick { .. } => Some(true),
            TrayIconEvent::Click { .. } => Some(false),
            _ => None,
        }
    } else {
        None
    }
}

/// Poll for menu events
pub fn poll_menu_event() -> Option<TrayMenuAction> {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        let id = event.id.0;

        if TRAY_MENU_SHOW_ID.get().is_some_and(|s| s == &id) {
            return Some(TrayMenuAction::Show);
        }
        if TRAY_MENU_REFRESH_ID.get().is_some_and(|s| s == &id) {
            return Some(TrayMenuAction::Refresh);
        }
        if TRAY_MENU_SETTINGS_ID.get().is_some_and(|s| s == &id) {
            return Some(TrayMenuAction::Settings);
        }
        if TRAY_MENU_EXIT_ID.get().is_some_and(|s| s == &id) {
            return Some(TrayMenuAction::Exit);
        }
    }
    None
}
