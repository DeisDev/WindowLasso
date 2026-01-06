//! Monitor picker view

use crate::app::Message;
use crate::localization::{keys, Localization};
use crate::types::{MonitorInfo, WindowInfo};
use crate::views::styles::{self, colors};
use iced::widget::{button, canvas, column, container, row, svg, text, tooltip};
use iced::{Alignment, Element, Fill, Length, Point, Size};

/// Build the monitor picker view
pub fn view<'a>(
    selected_window: &'a WindowInfo,
    monitors: &'a [MonitorInfo],
    loc: &'a Localization,
) -> Element<'a, Message> {
    let header = build_header(selected_window, loc);
    let monitor_grid = build_monitor_grid(monitors, loc);

    container(
        column![header, monitor_grid]
            .spacing(0)
            .width(Fill)
            .height(Fill),
    )
    .style(styles::main_container)
    .width(Fill)
    .height(Fill)
    .into()
}

fn build_header<'a>(window: &'a WindowInfo, loc: &'a Localization) -> Element<'a, Message> {
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
            .on_press(Message::CancelSelection),
        text(loc.get(keys::TOOLTIP_BACK)).size(13),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .style(styles::tooltip_container);

    let title = text(loc.get(keys::MONITOR_TITLE))
        .size(24)
        .color(colors::TEXT);

    let window_info = text(format!("\"{}\"", truncate_string(&window.title, 40)))
        .size(14)
        .color(colors::TEXT_DIM);

    container(
        column![
            row![back_btn, iced::widget::Space::new().width(Fill)].width(Fill),
            iced::widget::Space::new().height(8),
            title,
            window_info,
            iced::widget::Space::new().height(4),
            text(loc.get(keys::MONITOR_SELECT))
                .size(13)
                .color(colors::TEXT_DIM),
        ]
        .spacing(4)
        .padding(16),
    )
    .style(styles::header_container)
    .width(Fill)
    .into()
}

fn build_monitor_grid<'a>(
    monitors: &'a [MonitorInfo],
    loc: &'a Localization,
) -> Element<'a, Message> {
    // Sort monitors: primary first, then by display index
    let mut sorted_monitors: Vec<&MonitorInfo> = monitors.iter().collect();
    sorted_monitors.sort_by(|a, b| match (a.is_primary, b.is_primary) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.display_index.cmp(&b.display_index),
    });

    // Calculate the maximum resolution for relative scaling
    let max_pixels = monitors
        .iter()
        .map(|m| (m.bounds.width() as u64) * (m.bounds.height() as u64))
        .max()
        .unwrap_or(1) as f64;

    let monitor_cards: Vec<Element<Message>> = sorted_monitors
        .iter()
        .map(|m| build_monitor_card(m, loc, max_pixels))
        .collect();

    // Layout monitors in a column for simplicity
    let content = column(monitor_cards).spacing(16).width(Fill);

    container(content)
        .padding(16)
        .width(Fill)
        .height(Fill)
        .into()
}

fn build_monitor_card<'a>(
    monitor: &'a MonitorInfo,
    loc: &'a Localization,
    max_pixels: f64,
) -> Element<'a, Message> {
    let style = if monitor.is_primary {
        styles::monitor_card_primary
    } else {
        styles::monitor_card
    };

    // Monitor name
    let name = text(&monitor.name).size(18).color(colors::TEXT);

    // Resolution
    let width = monitor.bounds.width();
    let height = monitor.bounds.height();
    let mut args = fluent::FluentArgs::new();
    args.set("width", fluent::FluentValue::from(width as i64));
    args.set("height", fluent::FluentValue::from(height as i64));
    let resolution = text(loc.get_with_args(keys::MONITOR_RESOLUTION, Some(&args)))
        .size(13)
        .color(colors::TEXT_DIM);

    // Primary badge
    let primary_badge: Element<Message> = if monitor.is_primary {
        container(
            text(loc.get(keys::MONITOR_PRIMARY))
                .size(11)
                .color(colors::PRIMARY),
        )
        .padding([2, 8])
        .style(|_: &_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.36, 0.56, 0.96, 0.2,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        iced::widget::Space::new().height(0).into()
    };

    // Move button
    let move_btn = button(text(loc.get(keys::BTN_MOVE)).size(14))
        .style(styles::primary_button)
        .padding([10, 20])
        .on_press(Message::MoveToMonitor(monitor.clone()));

    // Calculate visual size based on relative resolution
    // Base size for the largest monitor
    let base_width: f32 = 140.0;

    // Scale based on pixel count relative to max
    let this_pixels = (width as u64 * height as u64) as f64;
    let scale = (this_pixels / max_pixels).sqrt() as f32;

    // Maintain aspect ratio
    let aspect_ratio = width as f32 / height.max(1) as f32;
    let box_width = base_width * scale;
    let box_height = box_width / aspect_ratio;

    // Create the monitor preview using canvas
    let monitor_visual = canvas(MonitorPreview {
        width: box_width,
        height: box_height,
        is_primary: monitor.is_primary,
    })
    .width(Length::Fixed(box_width))
    .height(Length::Fixed(box_height));

    let content = row![
        monitor_visual,
        iced::widget::Space::new().width(16),
        column![name, resolution, primary_badge,]
            .spacing(4)
            .width(Fill),
        move_btn,
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .padding(16)
    .width(Fill);

    container(content).style(style).width(Fill).into()
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Canvas-based monitor preview that renders a desktop-like visualization
struct MonitorPreview {
    width: f32,
    height: f32,
    is_primary: bool,
}

impl<Message> canvas::Program<Message> for MonitorPreview {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw monitor bezel
        let bezel_color = if self.is_primary {
            iced::Color::from_rgb(0.36, 0.56, 0.96)
        } else {
            colors::BORDER
        };

        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(self.width, self.height),
            bezel_color,
        );

        // Draw screen area (inset by bezel)
        let bezel_width = 3.0;
        let screen_color = iced::Color::from_rgb(0.12, 0.14, 0.18);

        frame.fill_rectangle(
            Point::new(bezel_width, bezel_width),
            Size::new(
                self.width - bezel_width * 2.0,
                self.height - bezel_width * 2.0,
            ),
            screen_color,
        );

        // Draw taskbar at bottom
        let taskbar_height = (self.height * 0.08).max(4.0);
        let taskbar_color = iced::Color::from_rgb(0.08, 0.09, 0.12);

        frame.fill_rectangle(
            Point::new(bezel_width, self.height - bezel_width - taskbar_height),
            Size::new(self.width - bezel_width * 2.0, taskbar_height),
            taskbar_color,
        );

        // Draw some fake windows to give it a "desktop" look
        let window_color = iced::Color::from_rgb(0.2, 0.22, 0.28);

        // Window 1 (top-left area)
        let w1_x = bezel_width + (self.width * 0.08);
        let w1_y = bezel_width + (self.height * 0.1);
        let w1_w = self.width * 0.35;
        let w1_h = self.height * 0.4;
        frame.fill_rectangle(Point::new(w1_x, w1_y), Size::new(w1_w, w1_h), window_color);

        // Window 2 (overlapping, center-right)
        let w2_x = bezel_width + (self.width * 0.3);
        let w2_y = bezel_width + (self.height * 0.25);
        let w2_w = self.width * 0.45;
        let w2_h = self.height * 0.45;
        let window_color_2 = iced::Color::from_rgb(0.25, 0.27, 0.32);
        frame.fill_rectangle(
            Point::new(w2_x, w2_y),
            Size::new(w2_w, w2_h),
            window_color_2,
        );

        vec![frame.into_geometry()]
    }
}
