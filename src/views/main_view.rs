//! Main window list view

use crate::app::Message;
use crate::localization::{keys, Localization};
use crate::types::{WindowInfo, GITHUB_URL, ISSUES_URL, VERSION};
use crate::views::styles::{self, colors};
use iced::widget::{button, column, container, image, row, scrollable, svg, text, tooltip};
use iced::{Alignment, Element, Fill};

/// Build the main view showing the window list
pub fn view<'a>(
    windows: &'a [WindowInfo],
    loc: &'a Localization,
    status_message: Option<&'a str>,
) -> Element<'a, Message> {
    let header = build_header(loc, windows.len());
    let window_list = build_window_list(windows, loc);
    let footer = build_footer(status_message);

    container(
        column![header, window_list, footer]
            .spacing(0)
            .width(Fill)
            .height(Fill),
    )
    .style(styles::main_container)
    .width(Fill)
    .height(Fill)
    .into()
}

fn build_header<'a>(loc: &'a Localization, window_count: usize) -> Element<'a, Message> {
    let title = text(loc.get(keys::APP_TITLE)).size(24).color(colors::TEXT);

    let count_text = text(loc.get_with_count(keys::WINDOWS_COUNT, window_count as i64))
        .size(14)
        .color(colors::TEXT_DIM);

    let refresh_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/refresh-ccw.svg"
    )))
    .width(18)
    .height(18)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT),
    });

    let refresh_btn = tooltip(
        button(refresh_icon)
            .style(styles::secondary_button)
            .padding([8, 12])
            .on_press(Message::RefreshWindows),
        text(loc.get(keys::TOOLTIP_REFRESH)).size(13),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .style(styles::tooltip_container);

    let settings_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/settings-2.svg"
    )))
    .width(18)
    .height(18)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT),
    });

    let settings_btn = tooltip(
        button(settings_icon)
            .style(styles::secondary_button)
            .padding([8, 12])
            .on_press(Message::OpenSettings),
        text(loc.get(keys::TOOLTIP_SETTINGS)).size(13),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .style(styles::tooltip_container);

    container(
        row![
            column![title, count_text].spacing(4),
            iced::widget::Space::new().width(Fill),
            refresh_btn,
            settings_btn,
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(16),
    )
    .style(styles::header_container)
    .width(Fill)
    .into()
}

fn build_window_list<'a>(windows: &'a [WindowInfo], loc: &'a Localization) -> Element<'a, Message> {
    if windows.is_empty() {
        return container(
            text(loc.get(keys::WINDOWS_EMPTY))
                .size(16)
                .color(colors::TEXT_DIM),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into();
    }

    let items: Vec<Element<Message>> = windows.iter().map(|w| build_window_item(w, loc)).collect();

    scrollable(column(items).spacing(8).padding(16).width(Fill))
        .style(styles::list_scrollable)
        .width(Fill)
        .height(Fill)
        .into()
}

fn build_window_item<'a>(window: &'a WindowInfo, loc: &'a Localization) -> Element<'a, Message> {
    let style = if window.is_offscreen {
        styles::window_item_offscreen
    } else {
        styles::window_item
    };

    // Title row with minimized indicator
    let title_str = if window.is_minimized {
        format!("{} [{}]", window.title, loc.get(keys::WINDOWS_MINIMIZED))
    } else {
        window.title.clone()
    };

    let title = text(title_str).size(15).color(colors::TEXT);

    // Subtitle with process name and monitor info
    let monitor_info = if window.is_offscreen {
        loc.get(keys::WINDOWS_OFFSCREEN)
    } else {
        window.monitor_name.clone().unwrap_or_default()
    };

    let subtitle = text(format!("{} • {}", window.process_name, monitor_info))
        .size(12)
        .color(colors::TEXT_DIM);

    // Process icon or status indicator
    let icon_element: Element<'a, Message> = if let Some(ref rgba) = window.icon_rgba {
        let handle = image::Handle::from_rgba(window.icon_size, window.icon_size, rgba.clone());
        container(image(handle).width(24).height(24))
            .width(24)
            .height(24)
            .into()
    } else if window.is_offscreen {
        text("⚠").size(20).color(colors::WARNING).into()
    } else if window.is_minimized {
        text("▽").size(20).color(colors::TEXT_DIM).into()
    } else {
        text("◻").size(20).color(colors::TEXT_DIM).into()
    };

    // Lasso button with icon
    let lasso_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/lasso.svg"
    )))
    .width(16)
    .height(16)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT),
    });

    let lasso_btn = tooltip(
        button(lasso_icon)
            .style(if window.is_offscreen {
                styles::primary_button
            } else {
                styles::secondary_button
            })
            .padding([6, 10])
            .on_press(Message::SelectWindow(window.clone())),
        text(loc.get(keys::TOOLTIP_LASSO)).size(13),
        tooltip::Position::Left,
    )
    .gap(4)
    .style(styles::tooltip_container);

    let content = row![
        icon_element,
        column![title, subtitle].spacing(2).width(Fill),
        lasso_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(12);

    container(content).style(style).width(Fill).into()
}

fn build_footer<'a>(status_message: Option<&'a str>) -> Element<'a, Message> {
    let left_content: Element<'a, Message> = if let Some(msg) = status_message {
        text(msg).size(11).color(colors::TEXT_DIM).into()
    } else {
        text(format!("v{}", VERSION))
            .size(11)
            .color(colors::TEXT_DIM)
            .into()
    };

    let github_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/github.svg"
    )))
    .width(14)
    .height(14)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT_DIM),
    });

    let github_btn = tooltip(
        button(github_icon)
            .style(styles::icon_button)
            .padding(4)
            .on_press(Message::OpenUrl(GITHUB_URL.to_string())),
        text("GitHub").size(12),
        tooltip::Position::Top,
    )
    .gap(4)
    .style(styles::tooltip_container);

    let bug_icon = svg(svg::Handle::from_memory(include_bytes!(
        "../../icons/interface/bug.svg"
    )))
    .width(14)
    .height(14)
    .style(|_theme, _status| svg::Style {
        color: Some(colors::TEXT_DIM),
    });

    let bug_btn = tooltip(
        button(bug_icon)
            .style(styles::icon_button)
            .padding(4)
            .on_press(Message::OpenUrl(ISSUES_URL.to_string())),
        text("Report Issue").size(12),
        tooltip::Position::Top,
    )
    .gap(4)
    .style(styles::tooltip_container);

    container(
        row![
            left_content,
            iced::widget::Space::new().width(Fill),
            github_btn,
            bug_btn,
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([6, 12]),
    )
    .style(styles::footer_container)
    .width(Fill)
    .into()
}
