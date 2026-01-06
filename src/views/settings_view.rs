//! Settings view

use crate::app::Message;
use crate::localization::{keys, Localization};
use crate::types::{AppSettings, HotkeyAction, HotkeyBinding, Language};
use crate::views::styles::{self, colors};
use iced::widget::{button, column, container, pick_list, row, scrollable, svg, text, toggler, tooltip};
use iced::{Alignment, Element, Fill};

/// Build the settings view
pub fn view<'a>(settings: &'a AppSettings, loc: &'a Localization) -> Element<'a, Message> {
    let header = build_header(loc);
    let content = build_settings_content(settings, loc);

    container(column![header, content].spacing(0).width(Fill).height(Fill))
        .style(styles::main_container)
        .width(Fill)
        .height(Fill)
        .into()
}

fn build_header<'a>(loc: &'a Localization) -> Element<'a, Message> {
    let back_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/chevron-left.svg"
    )))
    .width(18)
    .height(18)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT),
    });

    let back_btn = tooltip(
        button(back_icon)
            .style(styles::secondary_button)
            .padding([8, 12])
            .on_press(Message::CloseSettings),
        text(loc.get(keys::TOOLTIP_BACK)).size(13),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .style(styles::tooltip_container);

    let title = text(loc.get(keys::SETTINGS_TITLE))
        .size(24)
        .color(colors::TEXT);

    container(
        column![
            row![back_btn, iced::widget::Space::new().width(Fill)].width(Fill),
            iced::widget::Space::new().height(8),
            title,
        ]
        .spacing(4)
        .padding(16),
    )
    .style(styles::header_container)
    .width(Fill)
    .into()
}

fn build_settings_content<'a>(
    settings: &'a AppSettings,
    loc: &'a Localization,
) -> Element<'a, Message> {
    // Language section
    let language_row = build_setting_row(
        loc.get(keys::SETTINGS_LANGUAGE),
        build_language_picker(settings),
    );

    // Behavior section header
    let behavior_header = text(loc.get(keys::SETTINGS_BEHAVIOR))
        .size(13)
        .color(colors::TEXT_DIM);

    let auto_focus_row = build_toggle_row(
        loc.get(keys::SETTINGS_AUTO_FOCUS),
        settings.auto_focus_after_lasso,
        Message::SetAutoFocusAfterLasso,
    );

    let close_after_recovery_row = build_toggle_row(
        loc.get(keys::SETTINGS_CLOSE_AFTER_RECOVERY),
        settings.close_after_recovery,
        Message::SetCloseAfterRecovery,
    );

    let tray_row = build_toggle_row(
        loc.get(keys::SETTINGS_TRAY),
        settings.minimize_to_tray.unwrap_or(false),
        |enabled| Message::SetMinimizeToTray(Some(enabled)),
    );

    // Hotkeys section header
    let hotkeys_header = text(loc.get(keys::SETTINGS_HOTKEYS))
        .size(13)
        .color(colors::TEXT_DIM);

    let hotkey_rows = column![
        build_hotkey_row(
            loc.get(keys::HOTKEY_LASSO),
            &settings.hotkeys.lasso_window,
            HotkeyAction::LassoWindow,
            loc,
        ),
        build_hotkey_row(
            loc.get(keys::HOTKEY_REFRESH),
            &settings.hotkeys.refresh_windows,
            HotkeyAction::RefreshWindows,
            loc,
        ),
        build_hotkey_row(
            loc.get(keys::HOTKEY_PRIMARY),
            &settings.hotkeys.move_to_primary,
            HotkeyAction::MoveToPrimary,
            loc,
        ),
        build_hotkey_row(
            loc.get(keys::HOTKEY_ALL_PRIMARY),
            &settings.hotkeys.move_all_to_primary,
            HotkeyAction::MoveAllToPrimary,
            loc,
        ),
        build_hotkey_row(
            loc.get(keys::HOTKEY_CENTER),
            &settings.hotkeys.center_window,
            HotkeyAction::CenterWindow,
            loc,
        ),
        build_hotkey_row(
            loc.get(keys::HOTKEY_NEXT_MONITOR),
            &settings.hotkeys.next_monitor,
            HotkeyAction::NextMonitor,
            loc,
        ),
    ]
    .spacing(0);

    let content = column![
        language_row,
        divider(),
        behavior_header,
        auto_focus_row,
        close_after_recovery_row,
        tray_row,
        divider(),
        hotkeys_header,
        hotkey_rows,
    ]
    .spacing(12)
    .padding(20)
    .width(Fill);

    scrollable(container(content).width(Fill))
        .style(styles::list_scrollable)
        .width(Fill)
        .height(Fill)
        .into()
}

fn divider<'a>() -> Element<'a, Message> {
    container(iced::widget::Space::new().height(1))
        .style(|_: &_| container::Style {
            background: Some(iced::Background::Color(colors::BORDER)),
            ..Default::default()
        })
        .height(1)
        .width(Fill)
        .into()
}

fn build_setting_row<'a>(label: String, control: Element<'a, Message>) -> Element<'a, Message> {
    row![
        text(label).size(14).color(colors::TEXT),
        iced::widget::Space::new().width(Fill),
        control
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}

fn build_toggle_row<'a, F>(label: String, value: bool, on_toggle: F) -> Element<'a, Message>
where
    F: 'a + Fn(bool) -> Message,
{
    row![
        text(label).size(14).color(colors::TEXT),
        iced::widget::Space::new().width(Fill),
        toggler(value).on_toggle(on_toggle).size(20)
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .padding([8, 0])
    .width(Fill)
    .into()
}

fn build_language_picker<'a>(settings: &'a AppSettings) -> Element<'a, Message> {
    let languages: Vec<String> = Language::all()
        .iter()
        .map(|l| l.native_name().to_string())
        .collect();

    let current_language = Language::from_code(&settings.language)
        .map(|l| l.native_name().to_string())
        .unwrap_or_else(|| "English".to_string());

    pick_list(languages, Some(current_language), |selected| {
        let code = Language::all()
            .iter()
            .find(|l| l.native_name() == selected)
            .map(|l| l.code().to_string())
            .unwrap_or_else(|| "en".to_string());
        Message::ChangeLanguage(code)
    })
    .padding([6, 12])
    .into()
}

fn build_hotkey_row<'a>(
    label_text: String,
    binding: &'a HotkeyBinding,
    action: HotkeyAction,
    loc: &'a Localization,
) -> Element<'a, Message> {
    let name = text(label_text).size(14).color(colors::TEXT);

    let shortcut_display = container(
        text(binding.display_string())
            .size(12)
            .color(colors::TEXT_DIM),
    )
    .style(|_: &_| container::Style {
        background: Some(iced::Background::Color(colors::SURFACE_HOVER)),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding([4, 8]);

    let edit_btn = button(text(loc.get(keys::HOTKEY_EDIT)).size(11))
        .style(styles::secondary_button)
        .padding([4, 8])
        .on_press(Message::EditHotkey(action));

    let enabled_toggle = toggler(binding.enabled)
        .on_toggle(move |enabled| Message::ToggleHotkey(action, enabled))
        .size(18);

    row![
        name,
        iced::widget::Space::new().width(Fill),
        shortcut_display,
        edit_btn,
        enabled_toggle,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .padding([10, 0])
    .width(Fill)
    .into()
}

/// Build the hotkey editing dialog
pub fn hotkey_edit_view<'a>(action: HotkeyAction, loc: &'a Localization) -> Element<'a, Message> {
    let action_name = match action {
        HotkeyAction::LassoWindow => loc.get(keys::HOTKEY_LASSO),
        HotkeyAction::RefreshWindows => loc.get(keys::HOTKEY_REFRESH),
        HotkeyAction::MoveToPrimary => loc.get(keys::HOTKEY_PRIMARY),
        HotkeyAction::MoveAllToPrimary => loc.get(keys::HOTKEY_ALL_PRIMARY),
        HotkeyAction::CenterWindow => loc.get(keys::HOTKEY_CENTER),
        HotkeyAction::NextMonitor => loc.get(keys::HOTKEY_NEXT_MONITOR),
    };

    let title = text(format!("{}: {}", loc.get(keys::HOTKEY_EDIT), action_name))
        .size(20)
        .color(colors::TEXT);

    let instruction = text(loc.get(keys::HOTKEY_PRESS))
        .size(14)
        .color(colors::TEXT_DIM);

    let cancel_btn = button(text(loc.get(keys::BTN_CANCEL)).size(14))
        .style(styles::secondary_button)
        .padding([10, 20])
        .on_press(Message::CancelHotkeyEdit);

    container(
        container(
            column![
                title,
                iced::widget::Space::new().height(16),
                instruction,
                iced::widget::Space::new().height(24),
                cancel_btn
            ]
            .align_x(Alignment::Center)
            .width(Fill),
        )
        .style(styles::card_container)
        .padding(32)
        .max_width(400),
    )
    .style(|_: &_| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.0, 0.0, 0.0, 0.7,
        ))),
        ..Default::default()
    })
    .width(Fill)
    .height(Fill)
    .center_x(Fill)
    .center_y(Fill)
    .into()
}
