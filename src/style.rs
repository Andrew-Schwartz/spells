use std::fmt::{self, Display};
use std::ops::Not;

use iced::{application, Color};
use iced::widget::{button, checkbox, container, pick_list, progress_bar, scrollable, slider, text, text_input};
use iced::widget::scrollable::{Scrollbar, Scroller};
use iced_aw::tabs;
use iced_style::{menu, rule};
use iced_style::rule::FillMode;
use iced_style::slider::{Handle, HandleShape};

use crate::utils::ColorExt;

pub mod types {
    use crate::Message;

    use super::Theme;

    type Renderer = iced::Renderer<Theme>;

    pub type Element<'a> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a> = iced::widget::Container<'a, Message, Renderer>;
    pub type Text = iced::widget::Text<Renderer>;
    pub type Row<'a> = iced::widget::Row<'a, Message, Renderer>;
    pub type Column<'a> = iced::widget::Column<'a, Message, Renderer>;
    pub type Button<'a> = iced::widget::Button<'a, Message, Renderer>;
    pub type Tooltip<'a> = iced::widget::Tooltip<'a, Message, Renderer>;
    pub type Scrollable<'a> = iced::widget::Scrollable<'a, Message, Renderer>;
}

impl Not for Theme {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        })
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    fn palette(self, style: Location) -> Palette {
        match self {
            Self::Dark => dark::palette(style),
            Self::Light => todo!(),
        }
    }
}

// todo clean this up - background vs surface, accent vs active?
#[derive(Copy, Clone)]
pub struct Palette {
    text: Color,
    background: Color,
    surface: Color,
    accent: Color,
    active: Color,
    hovered: Color,
    disabled: Color,
}

impl Palette {
    const TRANSPARENT: Self = Palette {
        text: Color::TRANSPARENT,
        background: Color::TRANSPARENT,
        surface: Color::TRANSPARENT,
        accent: Color::TRANSPARENT,
        active: Color::TRANSPARENT,
        hovered: Color::TRANSPARENT,
        disabled: Color::TRANSPARENT,
    };
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum Location {
    #[default]
    Default,
    Transparent,
    SettingsBar,
    Alternating { idx: usize, highlight: bool },
}

impl text::StyleSheet for Theme {
    type Style = Option<Color>;

    fn appearance(&self, color: Self::Style) -> text::Appearance {
        text::Appearance { color }
    }
}

impl application::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: Self::Style) -> application::Appearance {
        let palette = self.palette(style);
        application::Appearance {
            background_color: palette.background,
            text_color: palette.text,
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: Self::Style) -> container::Appearance {
        let palette = self.palette(style);
        container::Appearance {
            text_color: palette.text.into(),
            background: palette.background.into(),
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
}

impl rule::StyleSheet for Theme {
    type Style = Location;

    fn style(&self, style: Self::Style) -> rule::Appearance {
        let palette = self.palette(style);
        rule::Appearance {
            color: palette.text.a(0.3),
            width: 1,
            radius: 0.0,
            fill_mode: FillMode::Full,
        }
    }
}

impl button::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: Self::Style) -> button::Appearance {
        let palette = self.palette(style);
        button::Appearance {
            background: palette.active.into(),
            border_color: Color::TRANSPARENT,
            text_color: palette.text,
            border_radius: 4.0,
            ..Default::default()
        }
    }

    // todo this is the
    fn hovered(&self, style: Self::Style) -> button::Appearance {
        let palette = self.palette(style);
        button::Appearance {
            background: palette.hovered.into(),
            ..self.active(style)
        }
    }

    fn pressed(&self, style: Self::Style) -> button::Appearance {
        let palette = self.palette(style);
        button::Appearance {
            border_width: 1.0,
            // todo
            border_color: if style == Location::Transparent {
                Color::TRANSPARENT
            } else {
                palette.text
            },
            ..self.hovered(style)
        }
    }

    fn disabled(&self, style: Self::Style) -> button::Appearance {
        let palette = self.palette(style);
        button::Appearance {
            background: palette.disabled.into(),
            ..self.active(style)
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: Self::Style) -> text_input::Appearance {
        let palette = self.palette(style);
        text_input::Appearance {
            background: palette.surface.into(),
            border_radius: 2.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self, style: Self::Style) -> text_input::Appearance {
        let palette = self.palette(style);
        text_input::Appearance {
            border_width: 1.0,
            border_color: palette.active,
            ..self.active(style)
        }
    }

    fn placeholder_color(&self, style: Self::Style) -> Color {
        match style {
            // todo is this always good
            Location::Transparent => Color::TRANSPARENT,
            _ => Color::from_rgb(0.4, 0.4, 0.4),
        }
    }

    fn value_color(&self, style: Self::Style) -> Color {
        self.palette(style).text
    }

    fn selection_color(&self, style: Self::Style) -> Color {
        self.palette(style).active
    }

    fn hovered(&self, style: Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_width: 1.0,
            border_color: self.palette(style).accent.a(0.3),
            ..self.focused(style)
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: Self::Style) -> Scrollbar {
        let palette = self.palette(style);
        Scrollbar {
            background: None,
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: Scroller {
                color: palette.surface.darken(0.6),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> Scrollbar {
        let palette = self.palette(style);
        let active = self.active(style);
        Scrollbar {
            background: palette.surface.darken(0.4).into(),
            scroller: Scroller {
                color: palette.surface.darken(0.7),
                ..active.scroller
            },
            ..active
        }
    }
}

impl menu::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: Self::Style) -> menu::Appearance {
        let palette = self.palette(style);
        menu::Appearance {
            text_color: palette.text,
            background: palette.surface.into(),
            // todo
            border_color: [0.3, 0.3, 0.3].into(),
            selected_text_color: palette.text,
            selected_background: palette.active.into(),
            border_width: 1.0,
            border_radius: 2.0,
        }
    }
}

impl pick_list::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: <Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        let palette = self.palette(style);
        pick_list::Appearance {
            text_color: palette.text,
            placeholder_color: palette.text,
            background: palette.surface.into(),
            border_color: Color::TRANSPARENT,
            border_radius: 3.0,
            border_width: 0.0,
            icon_size: 0.0,
        }
    }

    fn hovered(&self, style: <Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            background: self.palette(style).hovered.into(),
            ..self.active(style)
        }
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.palette(style);
        checkbox::Appearance {
            background: if is_checked {
                palette.active
            } else {
                palette.surface
            }.into(),
            checkmark_color: palette.text,
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.active,
            // todo
            text_color: palette.text.into(),
        }
    }

    fn hovered(&self, style: Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.palette(style);
        let active = self.active(style, is_checked);
        checkbox::Appearance {
            background: Color {
                a: 0.8,
                ..if is_checked { palette.active } else { palette.surface }
            }.into(),
            ..active
        }
    }
}

impl slider::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: Self::Style) -> slider::Appearance {
        let palette = self.palette(style);
        slider::Appearance {
            // todo this has to be transparent for TRANSPARENT
            rail_colors: (palette.text, Color::TRANSPARENT),
            handle: Handle {
                shape: HandleShape::Circle { radius: 7.0 },
                color: palette.surface,
                border_width: 1.0,
                // todo this has to be transparent for TRANSPARENT
                border_color: palette.text,
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> slider::Appearance {
        let mut appearance = self.active(style);
        appearance.handle.border_width = 1.5;
        appearance
    }

    fn dragging(&self, style: Self::Style) -> slider::Appearance {
        let mut appearance = self.hovered(style);
        appearance.handle.border_color = self.palette(style).active;
        appearance.handle.border_width += 0.5;
        appearance
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: Self::Style) -> progress_bar::Appearance {
        let palette = self.palette(style);
        progress_bar::Appearance {
            background: palette.active.into(),
            bar: palette.active.into(),
            border_radius: 5.0,
        }
    }
}

impl tabs::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, _style: Self::Style, is_active: bool) -> tabs::Appearance {
        let palette = self.palette(Location::Default);
        tabs::Appearance {
            background: None,
            border_color: None,
            border_width: 0.0,
            tab_label_background: if is_active {
                palette.background
            } else {
                palette.surface
            }.into(),
            tab_label_border_color: Default::default(),
            tab_label_border_width: 0.0,
            icon_color: palette.text,
            text_color: palette.text,
        }
    }

    fn hovered(&self, _style: Self::Style, is_active: bool) -> tabs::Appearance {
        let palette = self.palette(Location::Default);
        tabs::Appearance {
            background: None,
            border_color: None,
            border_width: 0.0,
            tab_label_background: if is_active {
                palette.background.lighten(0.065)
            } else {
                palette.surface.lighten(0.065)
            }.into(),
            tab_label_border_color: Default::default(),
            tab_label_border_width: 0.0,
            icon_color: palette.text,
            text_color: palette.text,
        }
    }
}

#[allow(clippy::cast_precision_loss)]
mod dark {
    use iced::Color;

    // use iced::widget::{
//         button,
//         checkbox,
//         container,
//         pick_list,
//         progress_bar,
//         scrollable,
//         slider::{self, Handle, HandleShape},
//         text_input,
//     };
//     use iced_aw::tabs;
    use crate::style::{Location, Palette};

    pub fn palette(style: Location) -> Palette {
        match style {
            Location::Default => DEFAULT,
            Location::Transparent => Palette {
                text: DEFAULT.text,
                ..Palette::TRANSPARENT
            },
            Location::SettingsBar => SETTINGS_BAR,
            Location::Alternating { idx, highlight } => alternating(idx, highlight),
        }
    }

    const DEFAULT: Palette = Palette {
        text: Color::WHITE,
        background: Color::from_rgb(
            0x36 as f32 / 255.0,
            0x39 as f32 / 255.0,
            0x3F as f32 / 255.0,
        ),
        surface: Color::from_rgb(
            0x40 as f32 / 255.0,
            0x44 as f32 / 255.0,
            0x4B as f32 / 255.0,
        ),
        accent: Color::from_rgb(
            0x6F as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xE9 as f32 / 255.0,
        ),
        active: Color::from_rgb(
            0x62 as f32 / 255.0,
            0x79 as f32 / 255.0,
            0xCA as f32 / 255.0,
        ),
        hovered: Color::from_rgb(
            0x77 as f32 / 255.0,
            0x87 as f32 / 255.0,
            0xD7 as f32 / 255.0,
        ),
        disabled: Color::from_rgb(
            0x52 as f32 / 255.0,
            0x59 as f32 / 255.0,
            0x9A as f32 / 255.0,
        ),
    };

    // todo
    const SETTINGS_BAR: Palette = Palette {
        background: Color::from_rgb(
            0x2E as f32 / 255.0,
            0x2F as f32 / 255.0,
            0x37 as f32 / 255.0,
        ),
        accent: Color::from_rgb(
            0x3E as f32 / 255.0,
            0x3F as f32 / 255.0,
            0x47 as f32 / 255.0,
        ),
        ..DEFAULT
    };

    const fn alternating(idx: usize, highlight: bool) -> Palette {
        const BACKGROUNDS: [Color; 2] = [
            DEFAULT.background,
            Color::from_rgb(
                0x30 as f32 / 255.0,
                0x33 as f32 / 255.0,
                0x35 as f32 / 255.0,
            )];
        const HOVERED: [Color; 2] = [Color::from_rgb(
            0x41 as f32 / 255.0,
            0x3E as f32 / 255.0,
            0x44 as f32 / 255.0,
        ), Color::from_rgb(
            0x34 as f32 / 255.0,
            0x37 as f32 / 255.0,
            0x39 as f32 / 255.0,
        )];

        let background = BACKGROUNDS[idx % 2];
        Palette {
            active: background,
            background,
            hovered: if highlight { HOVERED[idx % 2] } else { background },
            ..DEFAULT
        }
    }

    mod color {
        // use iced::Color;

        /*pub mod tab_bar {
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
                }*/

        // pub mod alt {
        //     use iced::Color;
        //
        //     pub const BACKGROUNDS: [Color; 2] = [
        //         super::BACKGROUND,
        //         Color::from_rgb(
        //             0x30 as f32 / 255.0,
        //             0x33 as f32 / 255.0,
        //             0x35 as f32 / 255.0,
        //         )];
        //
        //     pub const HOVERED: [Color; 2] = [Color::from_rgb(
        //         0x41 as f32 / 255.0,
        //         0x3E as f32 / 255.0,
        //         0x44 as f32 / 255.0,
        //     ), Color::from_rgb(
        //         0x34 as f32 / 255.0,
        //         0x37 as f32 / 255.0,
        //         0x39 as f32 / 255.0,
        //     )];
        // }
    }

    // impl button::StyleSheet for SettingsBarStyle {
    //     fn active(&self) -> button::Style {
    //         button::Style {
    //             background: color::tab_bar::BACKGROUND.into(),
    //             text_color: Color::WHITE,
    //             ..button::Style::default()
    //         }
    //     }
    // }
    //
    // impl container::StyleSheet for SettingsBarStyle {
    //     fn style(&self) -> container::Style {
    //         container::Style {
    //             background: Some(Background::Color(color::tab_bar::BACKGROUND)),
    //             ..Container.style()
    //         }
    //     }
    // }
    //
    // impl progress_bar::StyleSheet for SettingsBarStyle {
    //     fn style(&self) -> progress_bar::Style {
    //         progress_bar::Style {
    //             background: color::settings_bar::PROGRESS_BAR.into(),
    //             bar: color::ACTIVE.into(),
    //             border_radius: 5.0,
    //         }
    //     }
    // }
    //
    // pub struct TabButton;
    //
    // impl button::StyleSheet for TabButton {
    //     fn active(&self) -> button::Style {
    //         button::Style {
    //             background: color::BACKGROUND.into(),
    //             text_color: Color::WHITE,
    //             ..button::Style::default()
    //         }
    //     }
    //
    //     fn hovered(&self) -> button::Style {
    //         button::Style {
    //             background: Color::from_rgb8(
    //                 0x40,
    //                 0x40,
    //                 0x48,
    //             ).into(),
    //             ..self.active()
    //         }
    //     }
    //
    //     fn disabled(&self) -> button::Style {
    //         button::Style {
    //             background: Color::from_rgb8(
    //                 0x46,
    //                 0x46,
    //                 0x57,
    //             ).into(),
    //             ..self.active()
    //         }
    //     }
    // }

    // pub mod alt {
    //     use crate::utils::ColorExt;
    //
    //     use super::*;
    //
    //     pub struct Container<const N: usize>;
    //
    //     impl<const N: usize> container::StyleSheet for Container<N> {
    //         fn style(&self) -> container::Style {
    //             container::Style {
    //                 background: Some(Background::Color(color::alt::BACKGROUNDS[N])),
    //                 ..super::Container.style()
    //             }
    //         }
    //     }
    //
    //     pub struct Button<const N: usize>(pub bool);
    //
    //     impl<const N: usize> button::StyleSheet for Button<N> {
    //         fn active(&self) -> button::Style {
    //             button::Style {
    //                 background: Color::TRANSPARENT.into(),
    //                 text_color: Color::WHITE,
    //                 border_width: 0.0,
    //                 border_color: Color::TRANSPARENT,
    //                 border_radius: 5.0,
    //                 ..button::Style::default()
    //             }
    //         }
    //
    //         fn hovered(&self) -> button::Style {
    //             let mut style = self.active();
    //             if self.0 {
    //                 style.background = color::alt::HOVERED[N].into();
    //             }
    //             style
    //         }
    //
    //         fn pressed(&self) -> button::Style {
    //             if self.0 {
    //                 button::Style {
    //                     border_width: 1.0,
    //                     border_radius: 3.0,
    //                     border_color: Color::WHITE.a(0.3),
    //                     ..self.active()
    //                 }
    //             } else {
    //                 self.active()
    //             }
    //         }
    //     }
    //
    //     pub struct TextInput<const N: usize>(pub bool);
    //
    //     impl<const N: usize> text_input::StyleSheet for TextInput<N> {
    //         fn active(&self) -> text_input::Style {
    //             text_input::Style {
    //                 background: Color::TRANSPARENT.into(),
    //                 border_radius: 2.0,
    //                 border_width: 0.0,
    //                 border_color: Color::TRANSPARENT,
    //             }
    //         }
    //
    //         fn focused(&self) -> text_input::Style {
    //             text_input::Style {
    //                 border_width: 1.0,
    //                 border_color: color::ACTIVE.clearer(0.8),
    //                 ..self.active()
    //             }
    //         }
    //
    //         fn placeholder_color(&self) -> Color {
    //             Color::WHITE.clearer(0.7)
    //         }
    //
    //         fn value_color(&self) -> Color {
    //             Color::WHITE
    //         }
    //
    //         fn selection_color(&self) -> Color {
    //             color::ACTIVE
    //         }
    //
    //         fn hovered(&self) -> text_input::Style {
    //             text_input::Style {
    //                 border_width: 1.0,
    //                 border_color: color::ACCENT.a(0.3),
    //                 ..self.focused()
    //             }
    //         }
    //     }
    // }
}