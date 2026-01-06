//! Custom styles for the application

use iced::widget::{button, container, scrollable};
use iced::{Background, Border, Color, Theme};

/// Colors for the dark theme
#[allow(dead_code)]
pub mod colors {
    use iced::Color;

    pub const BACKGROUND: Color = Color::from_rgb(0.11, 0.11, 0.13);
    pub const SURFACE: Color = Color::from_rgb(0.15, 0.15, 0.17);
    pub const SURFACE_HOVER: Color = Color::from_rgb(0.20, 0.20, 0.22);
    pub const SURFACE_SELECTED: Color = Color::from_rgb(0.25, 0.25, 0.28);
    pub const PRIMARY: Color = Color::from_rgb(0.36, 0.56, 0.96);
    pub const PRIMARY_HOVER: Color = Color::from_rgb(0.46, 0.66, 1.0);
    pub const DANGER: Color = Color::from_rgb(0.92, 0.35, 0.35);
    pub const WARNING: Color = Color::from_rgb(0.95, 0.65, 0.25);
    pub const SUCCESS: Color = Color::from_rgb(0.35, 0.78, 0.50);
    pub const TEXT: Color = Color::from_rgb(0.93, 0.93, 0.93);
    pub const TEXT_DIM: Color = Color::from_rgb(0.60, 0.60, 0.65);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.28);
}

/// Main container style
pub fn main_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND)),
        text_color: Some(colors::TEXT),
        ..Default::default()
    }
}

/// Header container style with subtle border
pub fn header_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND)),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Card/panel container style
pub fn card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Primary action button style
pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::PRIMARY)),
        text_color: Color::WHITE,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::PRIMARY_HOVER)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::PRIMARY)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.35))),
            text_color: colors::TEXT_DIM,
            ..base
        },
    }
}

/// Secondary/outline button style
pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: colors::TEXT,
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_HOVER)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SURFACE_SELECTED)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_DIM,
            ..base
        },
    }
}

/// Window list item style (normal)
pub fn window_item(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Window list item style (off-screen/warning)
pub fn window_item_offscreen(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.95, 0.65, 0.25, 0.15))),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::WARNING,
            width: 2.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Monitor card style
pub fn monitor_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Monitor card style (primary)
pub fn monitor_card_primary(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.36, 0.56, 0.96, 0.15))),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::PRIMARY,
            width: 2.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Scrollable style for lists
pub fn list_scrollable(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(colors::BACKGROUND)),
            ..Default::default()
        },
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(colors::BORDER),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(colors::BORDER),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(colors::SURFACE),
            border: Border::default(),
            shadow: iced::Shadow::default(),
            icon: colors::TEXT,
        },
    }
}

/// Tooltip container style
pub fn tooltip_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        snap: false,
    }
}

/// Footer container style
pub fn footer_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT_DIM),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Icon button style (minimal, no background)
pub fn icon_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_DIM,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_HOVER)),
            text_color: colors::TEXT,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SURFACE_SELECTED)),
            ..base
        },
        button::Status::Disabled => base,
    }
}
