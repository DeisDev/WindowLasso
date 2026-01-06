//! Main application state and message handling

use crate::hotkeys::{self, HotkeyManager};
use crate::localization::Localization;
use crate::settings::{load_settings, save_settings};
use crate::tray::{self, SystemTray, TrayMenuAction};
use crate::types::{AppSettings, HotkeyAction, HotkeyBinding, MonitorInfo, Screen, WindowInfo};
use crate::views::{main_view, monitor_picker, settings_view, tray_dialog};
use crate::windows_api;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::{self, Duration};
use iced::{event, Element, Event, Subscription, Task, Theme};

/// Application state
pub struct App {
    /// List of open windows
    windows: Vec<WindowInfo>,
    /// List of connected monitors
    monitors: Vec<MonitorInfo>,
    /// Current screen/view
    screen: Screen,
    /// Application settings
    settings: AppSettings,
    /// Localization
    loc: Localization,
    /// Status message to display
    status_message: Option<String>,
    /// Whether we're showing the tray dialog
    show_tray_dialog: bool,
    /// The window ID that requested close (for tray dialog)
    pending_close_window: Option<iced::window::Id>,
    /// Whether we're editing a hotkey
    editing_hotkey: Option<HotkeyAction>,
    /// System tray (kept alive)
    #[allow(dead_code)]
    tray: Option<SystemTray>,
    /// Hotkey manager
    hotkey_manager: Option<HotkeyManager>,
    /// Whether to check for close-after-recovery on next WindowsLoaded
    pending_recovery_check: bool,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Window operations
    RefreshWindows,
    WindowsLoaded(Vec<WindowInfo>, Vec<MonitorInfo>),
    SelectWindow(WindowInfo),
    MoveToMonitor(MonitorInfo),
    CancelSelection,
    WindowMoved(Result<(), String>),

    // Settings
    OpenSettings,
    CloseSettings,
    ChangeLanguage(String),
    SetMinimizeToTray(Option<bool>),
    SetAutoFocusAfterLasso(bool),
    SetCloseAfterRecovery(bool),
    EditHotkey(HotkeyAction),
    CancelHotkeyEdit,
    UpdateHotkey(HotkeyAction, HotkeyBinding),
    ToggleHotkey(HotkeyAction, bool),

    // External links
    OpenUrl(String),

    // Keyboard input (for hotkey recording)
    KeyPressed(Key, Modifiers),

    // Tray dialog
    TrayDialogResponse(bool),
    RequestClose(iced::window::Id),

    // Window focus
    BringToFront,

    // Hotkey triggers (from global hotkeys)
    HotkeyLasso,
    HotkeyRefresh,
    HotkeyMoveToPrimary,
    HotkeyMoveAllToPrimary,
    HotkeyCenterWindow,
    HotkeyNextMonitor,

    // Tray events
    TrayMenuEvent(TrayMenuAction),
    TrayDoubleClick,

    // Timer/polling
    Tick,
    PollEvents,

    // Status
    ClearStatus,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = load_settings();
        let loc = Localization::new(&settings.language);

        // Initialize system tray
        let tray = SystemTray::new("WindowLasso").ok();

        // Initialize hotkey manager
        let mut hotkey_manager = HotkeyManager::new().ok();
        if let Some(ref mut manager) = hotkey_manager {
            manager.register_from_settings(&settings.hotkeys);
        }

        let app = Self {
            windows: Vec::new(),
            monitors: Vec::new(),
            screen: Screen::Main,
            settings,
            loc,
            status_message: None,
            show_tray_dialog: false,
            pending_close_window: None,
            editing_hotkey: None,
            tray,
            hotkey_manager,
            pending_recovery_check: false,
        };

        // Load windows on startup
        (
            app,
            Task::perform(load_windows_and_monitors(), |(w, m)| {
                Message::WindowsLoaded(w, m)
            }),
        )
    }

    pub fn title(&self) -> String {
        self.loc.get(crate::localization::keys::APP_TITLE)
    }

    pub fn theme(&self) -> Theme {
        if self.settings.theme.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RefreshWindows => {
                self.status_message =
                    Some(self.loc.get(crate::localization::keys::STATUS_REFRESHED));
                Task::batch([
                    Task::perform(load_windows_and_monitors(), |(w, m)| {
                        Message::WindowsLoaded(w, m)
                    }),
                    Task::perform(
                        async { tokio::time::sleep(tokio::time::Duration::from_secs(2)).await },
                        |_| Message::ClearStatus,
                    ),
                ])
            }

            Message::WindowsLoaded(windows, monitors) => {
                let had_offscreen_before = self.windows.iter().any(|w| w.is_offscreen);
                self.windows = windows;
                self.monitors = monitors;
                
                // Check if we should close after recovery
                if self.pending_recovery_check {
                    self.pending_recovery_check = false;
                    let has_offscreen_now = self.windows.iter().any(|w| w.is_offscreen);
                    
                    // Close if close_after_recovery is enabled and no more off-screen windows
                    if self.settings.close_after_recovery && had_offscreen_before && !has_offscreen_now {
                        return iced::exit();
                    }
                }
                
                Task::none()
            }

            Message::SelectWindow(window) => {
                self.screen = Screen::MonitorPicker {
                    selected_window: window,
                };
                Task::none()
            }

            Message::MoveToMonitor(monitor) => {
                if let Screen::MonitorPicker { selected_window } = &self.screen {
                    let hwnd = selected_window.hwnd;
                    let monitor_clone = monitor.clone();
                    let auto_focus = self.settings.auto_focus_after_lasso;
                    self.screen = Screen::Main;

                    Task::perform(
                        async move {
                            windows_api::move_window_to_monitor_with_options(
                                hwnd,
                                &monitor_clone,
                                None,
                                true,
                                auto_focus,
                            )
                        },
                        Message::WindowMoved,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CancelSelection => {
                self.screen = Screen::Main;
                Task::none()
            }

            Message::WindowMoved(result) => {
                match result {
                    Ok(()) => {
                        self.status_message =
                            Some(self.loc.get(crate::localization::keys::STATUS_MOVED));
                        // Set flag to check for close-after-recovery after windows reload
                        self.pending_recovery_check = true;
                    }
                    Err(e) => {
                        self.status_message = Some(self.loc.get_with_arg(
                            crate::localization::keys::STATUS_ERROR,
                            "message",
                            &e,
                        ));
                    }
                }

                // Refresh windows after move and clear status after delay
                Task::batch([
                    Task::perform(load_windows_and_monitors(), |(w, m)| {
                        Message::WindowsLoaded(w, m)
                    }),
                    Task::perform(
                        async { tokio::time::sleep(tokio::time::Duration::from_secs(3)).await },
                        |_| Message::ClearStatus,
                    ),
                ])
            }

            Message::OpenSettings => {
                self.screen = Screen::Settings;
                Task::none()
            }

            Message::CloseSettings => {
                self.screen = Screen::Main;
                // Save settings
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::ChangeLanguage(code) => {
                self.settings.language = code.clone();
                self.loc.set_language(&code);
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::SetMinimizeToTray(value) => {
                self.settings.minimize_to_tray = value;
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::SetAutoFocusAfterLasso(value) => {
                self.settings.auto_focus_after_lasso = value;
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::SetCloseAfterRecovery(value) => {
                self.settings.close_after_recovery = value;
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::OpenUrl(url) => {
                let _ = open::that(&url);
                Task::none()
            }

            Message::EditHotkey(action) => {
                self.editing_hotkey = Some(action);
                Task::none()
            }

            Message::CancelHotkeyEdit => {
                self.editing_hotkey = None;
                Task::none()
            }

            Message::UpdateHotkey(action, binding) => {
                match action {
                    HotkeyAction::LassoWindow => {
                        self.settings.hotkeys.lasso_window = binding;
                    }
                    HotkeyAction::RefreshWindows => {
                        self.settings.hotkeys.refresh_windows = binding;
                    }
                    HotkeyAction::MoveToPrimary => {
                        self.settings.hotkeys.move_to_primary = binding;
                    }
                    HotkeyAction::MoveAllToPrimary => {
                        self.settings.hotkeys.move_all_to_primary = binding;
                    }
                    HotkeyAction::CenterWindow => {
                        self.settings.hotkeys.center_window = binding;
                    }
                    HotkeyAction::NextMonitor => {
                        self.settings.hotkeys.next_monitor = binding;
                    }
                }
                self.editing_hotkey = None;
                // Re-register hotkeys with updated settings
                if let Some(ref mut manager) = self.hotkey_manager {
                    manager.register_from_settings(&self.settings.hotkeys);
                }
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::ToggleHotkey(action, enabled) => {
                match action {
                    HotkeyAction::LassoWindow => {
                        self.settings.hotkeys.lasso_window.enabled = enabled;
                    }
                    HotkeyAction::RefreshWindows => {
                        self.settings.hotkeys.refresh_windows.enabled = enabled;
                    }
                    HotkeyAction::MoveToPrimary => {
                        self.settings.hotkeys.move_to_primary.enabled = enabled;
                    }
                    HotkeyAction::MoveAllToPrimary => {
                        self.settings.hotkeys.move_all_to_primary.enabled = enabled;
                    }
                    HotkeyAction::CenterWindow => {
                        self.settings.hotkeys.center_window.enabled = enabled;
                    }
                    HotkeyAction::NextMonitor => {
                        self.settings.hotkeys.next_monitor.enabled = enabled;
                    }
                }
                // Re-register hotkeys with updated settings
                if let Some(ref mut manager) = self.hotkey_manager {
                    manager.register_from_settings(&self.settings.hotkeys);
                }
                let _ = save_settings(&self.settings);
                Task::none()
            }

            Message::KeyPressed(key, modifiers) => {
                // Only process if we're in hotkey editing mode
                if let Some(action) = self.editing_hotkey {
                    // Check for Escape to cancel
                    if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
                        return self.update(Message::CancelHotkeyEdit);
                    }
                    // Convert key to string (skip modifier-only presses)
                    if let Some(key_str) = key_to_string(&key) {
                        // Require at least one modifier for safety
                        let mods = modifiers_to_strings(&modifiers);
                        if !mods.is_empty() {
                            let binding = HotkeyBinding {
                                modifiers: mods,
                                key: key_str,
                                enabled: true,
                            };
                            return self.update(Message::UpdateHotkey(action, binding));
                        }
                    }
                }
                Task::none()
            }

            Message::BringToFront => {
                // Restore window from hidden mode and bring to foreground
                Task::batch([
                    iced::window::oldest().and_then(|id| {
                        iced::window::set_mode(id, iced::window::Mode::Windowed)
                    }),
                    iced::window::oldest().and_then(iced::window::gain_focus),
                ])
            }

            Message::TrayDialogResponse(minimize_to_tray) => {
                self.settings.minimize_to_tray = Some(minimize_to_tray);
                self.show_tray_dialog = false;
                let _ = save_settings(&self.settings);

                if minimize_to_tray {
                    // Hide window to tray
                    if let Some(id) = self.pending_close_window.take() {
                        iced::window::set_mode(id, iced::window::Mode::Hidden)
                    } else {
                        Task::none()
                    }
                } else {
                    // Exit application
                    self.pending_close_window = None;
                    iced::exit()
                }
            }

            Message::RequestClose(id) => {
                // Check if we should show the tray dialog
                if self.settings.minimize_to_tray.is_none() {
                    self.show_tray_dialog = true;
                    self.pending_close_window = Some(id);
                    Task::none()
                } else if self.settings.minimize_to_tray == Some(true) {
                    // Hide window to tray
                    iced::window::set_mode(id, iced::window::Mode::Hidden)
                } else {
                    // Exit
                    iced::exit()
                }
            }

            Message::HotkeyLasso => {
                // If there's an off-screen window, auto-select it
                if let Some(window) = self.windows.iter().find(|w| w.is_offscreen).cloned() {
                    self.screen = Screen::MonitorPicker {
                        selected_window: window,
                    };
                }
                // Also bring the app to front
                windows_api::focus_self();
                Task::none()
            }

            Message::HotkeyRefresh => Task::perform(load_windows_and_monitors(), |(w, m)| {
                Message::WindowsLoaded(w, m)
            }),

            Message::HotkeyMoveToPrimary => {
                // Move first off-screen window to primary monitor
                if let Some(window) = self.windows.iter().find(|w| w.is_offscreen) {
                    if let Some(primary) = self.monitors.iter().find(|m| m.is_primary) {
                        let hwnd = window.hwnd;
                        let monitor = primary.clone();
                        return Task::perform(
                            async move { windows_api::move_window_to_monitor(hwnd, &monitor) },
                            Message::WindowMoved,
                        );
                    }
                }
                Task::none()
            }

            Message::HotkeyMoveAllToPrimary => {
                // Move ALL off-screen windows to primary monitor
                let offscreen_windows: Vec<_> = self.windows.iter()
                    .filter(|w| w.is_offscreen)
                    .map(|w| w.hwnd)
                    .collect();
                
                if offscreen_windows.is_empty() {
                    return Task::none();
                }
                
                if let Some(primary) = self.monitors.iter().find(|m| m.is_primary).cloned() {
                    return Task::perform(
                        async move {
                            let mut last_result = Ok(());
                            for hwnd in offscreen_windows {
                                last_result = windows_api::move_window_to_monitor(hwnd, &primary);
                            }
                            last_result
                        },
                        Message::WindowMoved,
                    );
                }
                Task::none()
            }

            Message::HotkeyCenterWindow => {
                // Center the currently focused window
                if let Some(hwnd) = windows_api::get_foreground_window() {
                    let monitors = self.monitors.clone();
                    return Task::perform(
                        async move { windows_api::center_window(hwnd, &monitors) },
                        Message::WindowMoved,
                    );
                }
                Task::none()
            }

            Message::HotkeyNextMonitor => {
                // Move the focused window to the next monitor
                if let Some(hwnd) = windows_api::get_foreground_window() {
                    let monitors = self.monitors.clone();
                    return Task::perform(
                        async move { windows_api::move_to_next_monitor(hwnd, &monitors) },
                        Message::WindowMoved,
                    );
                }
                Task::none()
            }

            Message::ClearStatus => {
                self.status_message = None;
                Task::none()
            }

            Message::Tick => {
                // Auto-refresh window list
                Task::perform(load_windows_and_monitors(), |(w, m)| {
                    Message::WindowsLoaded(w, m)
                })
            }

            Message::PollEvents => {
                // Poll for hotkey events
                if let Some(ref manager) = self.hotkey_manager {
                    if let Some(id) = hotkeys::poll_hotkey_event() {
                        if let Some(action) = manager.get_action(id) {
                            return match action {
                                HotkeyAction::LassoWindow => {
                                    self.update(Message::HotkeyLasso)
                                }
                                HotkeyAction::RefreshWindows => {
                                    self.update(Message::HotkeyRefresh)
                                }
                                HotkeyAction::MoveToPrimary => {
                                    self.update(Message::HotkeyMoveToPrimary)
                                }
                                HotkeyAction::MoveAllToPrimary => {
                                    self.update(Message::HotkeyMoveAllToPrimary)
                                }
                                HotkeyAction::CenterWindow => {
                                    self.update(Message::HotkeyCenterWindow)
                                }
                                HotkeyAction::NextMonitor => {
                                    self.update(Message::HotkeyNextMonitor)
                                }
                            };
                        }
                    }
                }

                // Poll for tray menu events
                if let Some(action) = tray::poll_menu_event() {
                    return self.update(Message::TrayMenuEvent(action));
                }

                // Poll for tray double-click
                if let Some(true) = tray::poll_tray_click() {
                    return self.update(Message::TrayDoubleClick);
                }

                Task::none()
            }

            Message::TrayMenuEvent(action) => match action {
                TrayMenuAction::Show => {
                    self.update(Message::BringToFront)
                }
                TrayMenuAction::Refresh => {
                    self.update(Message::RefreshWindows)
                }
                TrayMenuAction::Settings => {
                    windows_api::focus_self();
                    self.update(Message::OpenSettings)
                }
                TrayMenuAction::Exit => {
                    iced::exit()
                }
            },

            Message::TrayDoubleClick => {
                self.update(Message::BringToFront)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Main content based on screen
        let content: Element<Message> = match &self.screen {
            Screen::Main => {
                main_view::view(&self.windows, &self.loc, self.status_message.as_deref())
            }
            Screen::MonitorPicker { selected_window } => {
                monitor_picker::view(selected_window, &self.monitors, &self.loc)
            }
            Screen::Settings => settings_view::view(&self.settings, &self.loc),
        };

        // Show tray dialog overlay if needed
        if self.show_tray_dialog {
            let overlay = tray_dialog::view(&self.loc);
            iced::widget::stack![content, overlay].into()
        } else if let Some(action) = self.editing_hotkey {
            let overlay = settings_view::hotkey_edit_view(action, &self.loc);
            iced::widget::stack![content, overlay].into()
        } else {
            content
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Combine subscriptions
        let poll_events = time::every(Duration::from_millis(50)).map(|_| Message::PollEvents);

        // Auto-refresh every 1 second when on main screen
        let auto_refresh = match self.screen {
            Screen::Main => time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            _ => Subscription::none(),
        };

        // Keyboard events for hotkey recording
        let keyboard = if self.editing_hotkey.is_some() {
            event::listen_with(|event, _status, _id| {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                    Some(Message::KeyPressed(key, modifiers))
                } else {
                    None
                }
            })
        } else {
            Subscription::none()
        };

        // Subscribe to window close requests
        let close_requests = iced::window::close_requests().map(Message::RequestClose);

        Subscription::batch([poll_events, auto_refresh, keyboard, close_requests])
    }
}

/// Convert iced Key to a string representation
fn key_to_string(key: &Key) -> Option<String> {
    match key {
        Key::Character(c) => {
            let s = c.to_string().to_uppercase();
            // Only allow single characters (letters, digits)
            if s.len() == 1 {
                Some(s)
            } else {
                None
            }
        }
        Key::Named(named) => {
            use iced::keyboard::key::Named;
            match named {
                Named::F1 => Some("F1".to_string()),
                Named::F2 => Some("F2".to_string()),
                Named::F3 => Some("F3".to_string()),
                Named::F4 => Some("F4".to_string()),
                Named::F5 => Some("F5".to_string()),
                Named::F6 => Some("F6".to_string()),
                Named::F7 => Some("F7".to_string()),
                Named::F8 => Some("F8".to_string()),
                Named::F9 => Some("F9".to_string()),
                Named::F10 => Some("F10".to_string()),
                Named::F11 => Some("F11".to_string()),
                Named::F12 => Some("F12".to_string()),
                Named::Space => Some("Space".to_string()),
                Named::Enter => Some("Enter".to_string()),
                Named::Tab => Some("Tab".to_string()),
                Named::Escape => None, // Escape cancels recording
                Named::Home => Some("Home".to_string()),
                Named::End => Some("End".to_string()),
                Named::PageUp => Some("PageUp".to_string()),
                Named::PageDown => Some("PageDown".to_string()),
                Named::Insert => Some("Insert".to_string()),
                Named::Delete => Some("Delete".to_string()),
                Named::ArrowUp => Some("Up".to_string()),
                Named::ArrowDown => Some("Down".to_string()),
                Named::ArrowLeft => Some("Left".to_string()),
                Named::ArrowRight => Some("Right".to_string()),
                // Modifier keys alone don't count as valid hotkeys
                Named::Control | Named::Shift | Named::Alt | Named::Super => None,
                _ => None,
            }
        }
        Key::Unidentified => None,
    }
}

/// Convert iced Modifiers to a list of modifier strings
fn modifiers_to_strings(modifiers: &Modifiers) -> Vec<String> {
    let mut result = Vec::new();
    if modifiers.control() {
        result.push("Ctrl".to_string());
    }
    if modifiers.alt() {
        result.push("Alt".to_string());
    }
    if modifiers.shift() {
        result.push("Shift".to_string());
    }
    if modifiers.logo() {
        result.push("Win".to_string());
    }
    result
}

/// Load windows and monitors asynchronously
async fn load_windows_and_monitors() -> (Vec<WindowInfo>, Vec<MonitorInfo>) {
    tokio::task::spawn_blocking(|| {
        let monitors = windows_api::enumerate_monitors();
        let windows = windows_api::enumerate_windows(&monitors);
        (windows, monitors)
    })
    .await
    .unwrap_or_default()
}
