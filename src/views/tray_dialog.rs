//! System tray dialog

use crate::app::Message;
use crate::localization::{keys, Localization};
use crate::views::styles::{self, colors};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill};

/// Build the "minimize to tray" confirmation dialog
pub fn view<'a>(loc: &'a Localization) -> Element<'a, Message> {
    let title = text(loc.get(keys::DIALOG_TRAY_TITLE))
        .size(20)
        .color(colors::TEXT);

    let message = text(loc.get(keys::DIALOG_TRAY_MESSAGE))
        .size(14)
        .color(colors::TEXT_DIM);

    let yes_btn = button(text(loc.get(keys::BTN_YES)).size(14))
        .style(styles::primary_button)
        .padding([10, 24])
        .on_press(Message::TrayDialogResponse(true));

    let no_btn = button(text(loc.get(keys::BTN_NO)).size(14))
        .style(styles::secondary_button)
        .padding([10, 24])
        .on_press(Message::TrayDialogResponse(false));

    let buttons = row![no_btn, yes_btn].spacing(12);

    container(
        container(
            column![
                title,
                iced::widget::Space::new().height(12),
                message,
                iced::widget::Space::new().height(24),
                buttons,
            ]
            .align_x(Alignment::Center)
            .width(Fill),
        )
        .style(styles::card_container)
        .padding(32)
        .max_width(420),
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
