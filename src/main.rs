//! WindowLasso - A sophisticated window management tool
//!
//! Recover and manage windows, especially those stuck on inaccessible displays.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hotkeys;
mod localization;
mod settings;
mod tray;
mod types;
mod views;
mod windows_api;

use app::App;
use iced::window::icon;
use iced::Size;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load window icon
    let window_icon = load_window_icon();

    iced::application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: Size::new(500.0, 600.0),
            min_size: Some(Size::new(400.0, 400.0)),
            position: iced::window::Position::Centered,
            icon: window_icon,
            exit_on_close_request: false,
            ..Default::default()
        })
        .run()
}

fn load_window_icon() -> Option<icon::Icon> {
    let icon_bytes = include_bytes!("../icons/app/windowlasso.ico");

    // Use the image crate to decode the ICO file
    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            icon::from_rgba(rgba.into_raw(), w, h).ok()
        }
        Err(_) => None,
    }
}
