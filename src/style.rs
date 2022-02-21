use std::fmt::{self, Display};
use std::ops::Not;

use iced::{button, checkbox, container, pick_list, progress_bar, scrollable, slider, text_input};
use iced_aw::tabs;

macro_rules! from {
    (
        @priv $style:ident => $module:ident: dark = $dark:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv $style:ident => $module:ident: light = $light:ident, dark = $dark:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv $style:ident => $module:ident: dark = $dark:ident,light = $light:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv-final $style:ident => $module:ident: light = $light:expr, dark = $dark:expr
    ) => {
        impl From<$style> for Box<dyn $module::StyleSheet> {
            fn from(style: $style) -> Self {
                match style {
                    $style::Light => $light,
                    $style::Dark => $dark,
                }
            }
        }
    };
    (
        $style:ident =>
        $($module:ident: $($light_dark_token:tt = $light_dark:ident),*);* $(;)?
    ) => {
        $(
            from! { @priv $style => $module: $($light_dark_token = $light_dark),* }
        )*
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SettingsBarStyle {
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TabButtonStyle {
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AlternatingStyle {
    style: Style,
    alt: bool,
    highlight: bool,
}

impl Style {
    pub fn settings_bar(self) -> SettingsBarStyle {
        match self {
            Self::Light => SettingsBarStyle::Light,
            Self::Dark => SettingsBarStyle::Dark,
        }
    }

    pub fn tab_button(self) -> TabButtonStyle {
        match self {
            Self::Light => TabButtonStyle::Light,
            Self::Dark => TabButtonStyle::Dark,
        }
    }

    pub fn alternating(self, n: usize) -> AlternatingStyle {
        AlternatingStyle {
            style: self,
            alt: n % 2 == 0,
            highlight: true,
        }
    }

    pub fn background(self) -> AlternatingStyle {
        AlternatingStyle {
            style: self,
            alt: true,
            highlight: false,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::Dark
    }
}

impl Not for Style {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Style::Light => "Light",
            Style::Dark => "Dark",
        })
    }
}

impl AlternatingStyle {
    pub fn no_highlight(self) -> Self {
        Self { highlight: false, ..self }
    }
}

from! { Style =>
    container: dark = Container;
    text_input: dark = TextInput;
    scrollable: dark = Scrollable;
    button: light = Button, dark = Button;
    pick_list: dark = PickList;
    checkbox: dark = Checkbox;
    slider: dark = Slider;
    tabs: dark = Tabs;
}

from! { SettingsBarStyle =>
    button: light = Button, dark = SettingsBarStyle;
    container: dark = SettingsBarStyle;
    progress_bar: dark = SettingsBarStyle;
}

from! { TabButtonStyle =>
    button: light = Button, dark = TabButton;
}

// todo epic macro for this too :)
impl From<AlternatingStyle> for Box<dyn container::StyleSheet> {
    fn from(AlternatingStyle { style, alt, .. }: AlternatingStyle) -> Self {
        match style {
            Style::Light => Default::default(),
            Style::Dark => if alt {
                dark::alt::Container::<0>.into()
            } else {
                dark::alt::Container::<1>.into()
            }
        }
    }
}

impl From<AlternatingStyle> for Box<dyn button::StyleSheet> {
    fn from(AlternatingStyle { style, alt, highlight }: AlternatingStyle) -> Self {
        match style {
            Style::Light => Default::default(),
            Style::Dark => if alt {
                dark::alt::Button::<0>(highlight).into()
            } else {
                dark::alt::Button::<1>(highlight).into()
            }
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
    use iced::{Background, button, checkbox, Color, container, pick_list, progress_bar, scrollable, slider, text_input};
    use iced::button::Style;
    use iced::slider::{Handle, HandleShape};
    use iced_aw::tabs;

    mod color {
        use iced::Color;

        pub const SURFACE: Color = Color::from_rgb(
            0x40 as f32 / 255.0,
            0x44 as f32 / 255.0,
            0x4B as f32 / 255.0,
        );

        pub const ACCENT: Color = Color::from_rgb(
            0x6F as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xE9 as f32 / 255.0,
        );

        pub const ACTIVE: Color = Color::from_rgb(
            0x62 as f32 / 255.0,
            0x79 as f32 / 255.0,
            0xCA as f32 / 255.0,
        );

        pub const HOVERED: Color = Color::from_rgb(
            0x77 as f32 / 255.0,
            0x87 as f32 / 255.0,
            0xD7 as f32 / 255.0,
        );

        pub const BACKGROUND: Color = Color::from_rgb(
            0x36 as f32 / 255.0,
            0x39 as f32 / 255.0,
            0x3F as f32 / 255.0,
        );

        pub const BRIGHTER_THAN_BACKGROUND: Color = Color::from_rgb(
            0x3A as f32 / 255.0,
            0x3C as f32 / 255.0,
            0x43 as f32 / 255.0,
        );

        pub const BRIGHTER_THAN_SURFACE: Color = Color::from_rgb(
            0x46 as f32 / 255.0,
            0x4A as f32 / 255.0,
            0x51 as f32 / 255.0,
        );

        pub mod tab_bar {
            use iced::Color;

            pub const BACKGROUND: Color = Color::from_rgb(
                0x2E as f32 / 255.0,
                0x2F as f32 / 255.0,
                0x37 as f32 / 255.0,
            );
        }

        pub mod settings_bar {
            use iced::Color;

            pub const PROGRESS_BAR: Color = Color::from_rgb(
                0x3E as f32 / 255.0,
                0x3F as f32 / 255.0,
                0x47 as f32 / 255.0,
            );
        }

        pub mod alt {
            use iced::Color;

            pub const BACKGROUNDS: [Color; 2] = [
                super::BACKGROUND,
                Color::from_rgb(
                    0x30 as f32 / 255.0,
                    0x33 as f32 / 255.0,
                    0x35 as f32 / 255.0,
                )];

            pub const HOVERED: [Color; 2] = [Color::from_rgb(
                0x41 as f32 / 255.0,
                0x3E as f32 / 255.0,
                0x44 as f32 / 255.0,
            ), Color::from_rgb(
                0x34 as f32 / 255.0,
                0x37 as f32 / 255.0,
                0x39 as f32 / 255.0,
            )];
        }
    }

    pub struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Background::Color(color::BACKGROUND)),
                ..Default::default()
            }
        }
    }

    pub struct TextInput;

    impl text_input::StyleSheet for TextInput {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(color::SURFACE),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: color::ACCENT,
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
            color::ACTIVE
        }

        fn hovered(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: Color { a: 0.3, ..color::ACCENT },
                ..self.focused()
            }
        }
    }

    pub struct Scrollable;

    impl scrollable::StyleSheet for Scrollable {
        fn active(&self) -> scrollable::Scrollbar {
            scrollable::Scrollbar {
                background: Some(Background::Color(color::SURFACE)),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: color::ACTIVE,
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> scrollable::Scrollbar {
            let active = self.active();
            scrollable::Scrollbar {
                background: Some(Background::Color(Color { a: 0.5, ..color::SURFACE })),
                scroller: scrollable::Scroller {
                    color: color::HOVERED,
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
                background: color::ACTIVE.into(),
                border_radius: 4.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: color::HOVERED.into(),
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
                background: Background::Color(color::SURFACE),
                border_width: 1.0,
                border_color: [0.3, 0.3, 0.3].into(),
                selected_text_color: Color::WHITE,
                selected_background: Background::Color(color::ACTIVE),
            }
        }

        fn active(&self) -> pick_list::Style {
            pick_list::Style {
                text_color: Color::WHITE,
                background: Background::Color(color::SURFACE),
                border_radius: 3.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                icon_size: 0.0,
            }
        }

        fn hovered(&self) -> pick_list::Style {
            pick_list::Style {
                background: Background::Color(color::HOVERED),
                ..self.active()
            }
        }
    }

    pub struct Checkbox;

    impl checkbox::StyleSheet for Checkbox {
        fn active(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(if is_checked {
                    color::ACTIVE
                } else {
                    color::SURFACE
                }),
                checkmark_color: Color::WHITE,
                border_radius: 2.0,
                border_width: 1.0,
                border_color: color::ACTIVE,
            }
        }

        fn hovered(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(Color {
                    a: 0.8,
                    ..if is_checked { color::ACTIVE } else { color::SURFACE }
                }),
                ..self.active(is_checked)
            }
        }
    }

    pub struct Slider;

    impl Slider {}

    impl slider::StyleSheet for Slider {
        fn active(&self) -> slider::Style {
            slider::Style {
                rail_colors: (Color::WHITE, Color::TRANSPARENT),
                handle: Handle {
                    shape: HandleShape::Circle { radius: 7.0 },
                    color: color::SURFACE,
                    border_width: 1.0,
                    border_color: Color::WHITE,
                },
            }
        }

        fn hovered(&self) -> slider::Style {
            let mut style = self.active();
            style.handle.border_width = 1.5;
            style
        }

        fn dragging(&self) -> slider::Style {
            let mut style = self.hovered();
            style.handle.border_color = color::ACTIVE;
            style.handle.border_width += 0.5;
            style
        }
    }

    pub struct Tabs;

    impl tabs::StyleSheet for Tabs {
        fn active(&self, is_active: bool) -> tabs::Style {
            tabs::Style {
                background: None,
                border_color: None,
                border_width: 0.0,
                tab_label_background: Background::Color(
                    if is_active { color::BACKGROUND } else { color::SURFACE }
                ),
                tab_label_border_color: Default::default(),
                tab_label_border_width: 0.0,
                icon_color: Color::WHITE,
                text_color: Color::WHITE,
            }
        }

        fn hovered(&self, is_active: bool) -> tabs::Style {
            tabs::Style {
                background: None,
                border_color: None,
                border_width: 0.0,
                tab_label_background: Background::Color(
                    if is_active {
                        color::BRIGHTER_THAN_BACKGROUND
                    } else {
                        color::BRIGHTER_THAN_SURFACE
                    }
                ),
                tab_label_border_color: Default::default(),
                tab_label_border_width: 0.0,
                icon_color: Color::WHITE,
                text_color: Color::WHITE,
            }
        }
    }

    pub struct SettingsBarStyle;

    impl button::StyleSheet for SettingsBarStyle {
        fn active(&self) -> button::Style {
            button::Style {
                background: color::tab_bar::BACKGROUND.into(),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }

    impl container::StyleSheet for SettingsBarStyle {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(color::tab_bar::BACKGROUND)),
                ..Container.style()
            }
        }
    }

    impl progress_bar::StyleSheet for SettingsBarStyle {
        fn style(&self) -> progress_bar::Style {
            progress_bar::Style {
                background: color::settings_bar::PROGRESS_BAR.into(),
                bar: color::ACTIVE.into(),
                border_radius: 5.0,
            }
        }
    }

    pub struct TabButton;

    impl button::StyleSheet for TabButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: color::BACKGROUND.into(),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb8(
                    0x40,
                    0x40,
                    0x48,
                ).into(),
                ..self.active()
            }
        }

        fn disabled(&self) -> Style {
            button::Style {
                background: Color::from_rgb8(
                    0x46,
                    0x46,
                    0x57,
                ).into(),
                ..self.active()
            }
        }
    }

    pub mod alt {
        use crate::utils::ColorExt;

        use super::*;

        pub struct Container<const N: usize>;

        impl<const N: usize> container::StyleSheet for Container<N> {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Some(Background::Color(color::alt::BACKGROUNDS[N])),
                    ..super::Container.style()
                }
            }
        }

        pub struct Button<const N: usize>(pub bool);

        impl<const N: usize> button::StyleSheet for Button<N> {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Color::TRANSPARENT.into(),
                    text_color: Color::WHITE,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    border_radius: 5.0,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                let mut style = self.active();
                if self.0 {
                    style.background = color::alt::HOVERED[N].into();
                }
                style
            }

            fn pressed(&self) -> button::Style {
                if self.0 {
                    button::Style {
                        border_width: 1.0,
                        border_radius: 3.0,
                        border_color: Color::WHITE.a(0.3),
                        ..self.active()
                    }
                } else {
                    self.active()
                }
            }
        }
    }
}