use iced::{button, checkbox, container, pick_list, scrollable, text_input};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    Light,
    Dark,
}

impl Default for Style {
    fn default() -> Self {
        Self::Dark
    }
}

impl From<Style> for String {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => "Light",
            Style::Dark => "Dark",
        }.into()
    }
}

impl From<Style> for Box<dyn container::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => Default::default(),
            Style::Dark => dark::Container.into(),
        }
    }
}

impl From<Style> for Box<dyn text_input::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => Default::default(),
            Style::Dark => dark::TextInput.into(),
        }
    }
}

impl From<Style> for Box<dyn scrollable::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => Default::default(),
            Style::Dark => dark::Scrollable.into(),
        }
    }
}

impl From<Style> for Box<dyn button::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => light::Button.into(),
            Style::Dark => dark::Button.into(),
        }
    }
}

impl From<Style> for Box<dyn pick_list::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => Default::default(),
            Style::Dark => dark::PickList.into(),
        }
    }
}

impl From<Style> for Box<dyn checkbox::StyleSheet> {
    fn from(theme: Style) -> Self {
        match theme {
            Style::Light => Default::default(),
            Style::Dark => dark::Checkbox.into(),
        }
    }
}

mod light {
    use iced::{button, Color};

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                // background: Color::from_rgb8(0xAD, 0xAD, 0xCD).into(),
                // border_radius: 4.0,
                // text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..Default::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                // text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                // border_width: 1.0,
                // border_color: [0.2, 0.2, 0.2].into(),
                ..self.hovered()
            }
        }

        fn disabled(&self) -> button::Style {
            let mut active = self.active();
            active.background = Color::from_rgb8(0xAE, 0xAE, 0xAE).into();
            active
            // button::Style {
            //     background: Color::from_rgb8(0x7D, 0x7D, 0x9D).into(),
            //     ..self.active()
            // }
        }
    }
}

#[allow(clippy::cast_precision_loss)]
mod dark {
    use iced::{Background, button, checkbox, Color, container, pick_list, scrollable, text_input};

    const SURFACE: Color = Color::from_rgb(
        0x40 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x4B as f32 / 255.0,
    );

    const ACCENT: Color = Color::from_rgb(
        0x6F as f32 / 255.0,
        0xFF as f32 / 255.0,
        0xE9 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x62 as f32 / 255.0,
        0x79 as f32 / 255.0,
        0xCA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    pub struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
                ..Default::default()
            }
        }
    }

    pub struct TextInput;

    impl text_input::StyleSheet for TextInput {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(SURFACE),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: ACCENT,
                ..self.active()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb(0.4, 0.4, 0.4)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            ACTIVE
        }

        fn hovered(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: Color { a: 0.3, ..ACCENT },
                ..self.focused()
            }
        }
    }

    pub struct Scrollable;

    impl scrollable::StyleSheet for Scrollable {
        fn active(&self) -> scrollable::Scrollbar {
            scrollable::Scrollbar {
                background: Some(Background::Color(SURFACE)),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: ACTIVE,
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> scrollable::Scrollbar {
            let active = self.active();
            scrollable::Scrollbar {
                background: Some(Background::Color(Color { a: 0.5, ..SURFACE })),
                scroller: scrollable::Scroller {
                    color: HOVERED,
                    ..active.scroller
                },
                ..active
            }
        }

        fn dragging(&self) -> scrollable::Scrollbar {
            let hovered = self.hovered();

            scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.85, 0.85, 0.85),
                    ..hovered.scroller
                },
                ..hovered
            }
        }
    }

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: ACTIVE.into(),
                border_radius: 4.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: HOVERED.into(),
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1.0,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }

        fn disabled(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb8(0x52, 0x59, 0x9A).into(),
                ..self.active()
            }
        }
    }

    pub struct PickList;

    impl pick_list::StyleSheet for PickList {
        fn menu(&self) -> pick_list::Menu {
            pick_list::Menu {
                text_color: Color::WHITE,
                background: Background::Color(SURFACE),
                border_width: 1.0,
                border_color: [0.3, 0.3, 0.3].into(),
                selected_text_color: Color::WHITE,
                selected_background: Background::Color(ACTIVE),
            }
        }

        fn active(&self) -> pick_list::Style {
            pick_list::Style {
                text_color: Color::WHITE,
                background: Background::Color(SURFACE),
                border_radius: 3.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                icon_size: 0.0,
            }
        }

        fn hovered(&self) -> pick_list::Style {
            pick_list::Style {
                background: Background::Color(HOVERED),
                ..self.active()
            }
        }
    }

    pub struct Checkbox;

    impl checkbox::StyleSheet for Checkbox {
        fn active(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(if is_checked {
                    ACTIVE
                } else {
                    SURFACE
                }),
                checkmark_color: Color::WHITE,
                border_radius: 2.0,
                border_width: 1.0,
                border_color: ACTIVE,
            }
        }

        fn hovered(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(Color {
                    a: 0.8,
                    ..if is_checked { ACTIVE } else { SURFACE }
                }),
                ..self.active(is_checked)
            }
        }
    }
}