//! Global hotkey support using global-hotkey

use crate::types::{HotkeyAction, HotkeyBinding, HotkeySettings};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;

/// Manages global hotkeys
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, HotkeyAction>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Result<Self, global_hotkey::Error> {
        let manager = GlobalHotKeyManager::new()?;
        Ok(Self {
            manager,
            registered: HashMap::new(),
        })
    }

    /// Register all enabled hotkeys from settings
    pub fn register_from_settings(&mut self, settings: &HotkeySettings) {
        // Unregister all existing
        self.unregister_all();

        // Register lasso window hotkey
        if settings.lasso_window.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.lasso_window) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered.insert(hotkey.id(), HotkeyAction::LassoWindow);
                }
            }
        }

        // Register refresh windows hotkey
        if settings.refresh_windows.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.refresh_windows) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered
                        .insert(hotkey.id(), HotkeyAction::RefreshWindows);
                }
            }
        }

        // Register move to primary hotkey
        if settings.move_to_primary.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.move_to_primary) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered
                        .insert(hotkey.id(), HotkeyAction::MoveToPrimary);
                }
            }
        }

        // Register move all to primary hotkey
        if settings.move_all_to_primary.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.move_all_to_primary) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered
                        .insert(hotkey.id(), HotkeyAction::MoveAllToPrimary);
                }
            }
        }

        // Register center window hotkey
        if settings.center_window.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.center_window) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered
                        .insert(hotkey.id(), HotkeyAction::CenterWindow);
                }
            }
        }

        // Register next monitor hotkey
        if settings.next_monitor.enabled {
            if let Some(hotkey) = binding_to_hotkey(&settings.next_monitor) {
                if self.manager.register(hotkey).is_ok() {
                    self.registered
                        .insert(hotkey.id(), HotkeyAction::NextMonitor);
                }
            }
        }
    }

    /// Unregister all hotkeys
    pub fn unregister_all(&mut self) {
        // Note: The global-hotkey crate doesn't expose individual unregister by id,
        // so we just clear our tracking map. Hotkeys will be re-registered when needed.
        self.registered.clear();
    }

    /// Get the action for a hotkey id
    pub fn get_action(&self, id: u32) -> Option<HotkeyAction> {
        self.registered.get(&id).copied()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister_all();
    }
}

/// Convert a HotkeyBinding to a global-hotkey HotKey
fn binding_to_hotkey(binding: &HotkeyBinding) -> Option<HotKey> {
    let code = key_to_code(&binding.key)?;
    let modifiers = modifiers_to_flags(&binding.modifiers);

    Some(HotKey::new(modifiers, code))
}

fn modifiers_to_flags(modifiers: &[String]) -> Option<Modifiers> {
    let mut flags = Modifiers::empty();

    for m in modifiers {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => flags |= Modifiers::CONTROL,
            "alt" => flags |= Modifiers::ALT,
            "shift" => flags |= Modifiers::SHIFT,
            "win" | "super" | "meta" => flags |= Modifiers::META,
            _ => {}
        }
    }

    if flags.is_empty() {
        None
    } else {
        Some(flags)
    }
}

fn key_to_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        "SPACE" => Some(Code::Space),
        "ENTER" | "RETURN" => Some(Code::Enter),
        "TAB" => Some(Code::Tab),
        "ESCAPE" | "ESC" => Some(Code::Escape),
        "HOME" => Some(Code::Home),
        "END" => Some(Code::End),
        "PAGEUP" => Some(Code::PageUp),
        "PAGEDOWN" => Some(Code::PageDown),
        "INSERT" => Some(Code::Insert),
        "DELETE" => Some(Code::Delete),
        "UP" => Some(Code::ArrowUp),
        "DOWN" => Some(Code::ArrowDown),
        "LEFT" => Some(Code::ArrowLeft),
        "RIGHT" => Some(Code::ArrowRight),
        _ => None,
    }
}

/// Poll for hotkey events (returns the hotkey ID if pressed)
pub fn poll_hotkey_event() -> Option<u32> {
    if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.state == HotKeyState::Pressed {
            return Some(event.id);
        }
    }
    None
}
