use std::fmt::{self, Display};
use std::ops::Not;

use iced::{application, Color};
use iced::widget::{button, checkbox, container, pick_list, progress_bar, scrollable, slider, text, text_input};
use iced::widget::scrollable::{Scrollbar, Scroller};
use iced_aw::style::tab_bar;
use iced_style::{menu, rule};
use iced_style::rule::FillMode;
use iced_style::slider::{Handle, HandleShape, Rail};

use crate::utils::ColorExt;

pub mod types {
    use crate::Message;

    use super::Theme;

    type Renderer = iced::Renderer<Theme>;

    pub type Element<'a> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a> = iced::widget::Container<'a, Message, Renderer>;
    pub type Text<'a> = iced::widget::Text<'a, Renderer>;
    pub type Row<'a> = iced::widget::Row<'a, Message, Renderer>;
    pub type Column<'a> = iced::widget::Column<'a, Message, Renderer>;
    pub type Button<'a> = iced::widget::Button<'a, Message, Renderer>;
    pub type ClickButton<'a> = crate::widgets::click_button::ClickButton<'a, Message, Renderer>;
    pub type Tooltip<'a> = iced::widget::Tooltip<'a, Message, Renderer>;
    pub type Scrollable<'a> = iced::widget::Scrollable<'a, Message, Renderer>;
    pub type TextInput<'a> = iced::widget::TextInput<'a, Message, Renderer>;
    pub type CheckBox<'a> = iced::widget::Checkbox<'a, Message, Renderer>;
    pub type PickList<'a, T> = iced::widget::PickList<'a, T, Message, Renderer>;
    pub type Slider<'a, T> = iced::widget::Slider<'a, T, Message, Renderer>;
    pub type Rule = iced::widget::Rule<Renderer>;
    pub type ProgressBar = iced::widget::ProgressBar<Renderer>;
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
    fn palette(self, style: &Location) -> Palette {
        match self {
            Self::Dark => dark::palette(style),
            Self::Light => light::palette(style),
        }
    }

    fn palette2(self, style: Location) -> Palette2 {
        match self {
            Self::Dark => dark::palette2(style),
            Self::Light => light::palette2(style),
        }
    }

    fn disable_by(self, color: Color, amount: f32) -> Color {
        match self {
            Theme::Dark => color.darken(amount),
            Theme::Light => color.lighten(amount),
        }
    }

    fn disable(self, color: Color) -> Color {
        self.disable_by(color, 0.2)
    }

    fn hover_by(self, color: Color, amount: f32) -> Color {
        match self {
            Theme::Dark => color.lighten(amount),
            Theme::Light => color.darken(amount),
        }
    }

    fn hover(self, color: Color) -> Color {
        self.hover_by(color, 0.1)
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

pub struct Palette2 {
    text: Color,
    background: Color,
    button: Color,
    // todo is this how I want to do?
    outline: Color,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Location {
    #[default]
    Default,
    Transparent,
    SettingsBar,
    Alternating { idx: usize, highlight: bool },
    AdvancedSearch { enabled: bool },
    Tooltip,
}

impl text::StyleSheet for Theme {
    type Style = Option<Color>;

    fn appearance(&self, color: Self::Style) -> text::Appearance {
        text::Appearance { color }
    }
}

impl application::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        let palette = self.palette2(*style);
        application::Appearance {
            background_color: palette.background,
            text_color: palette.text,
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let palette = self.palette2(*style);
        container::Appearance {
            text_color: palette.text.into(),
            background: palette.background.into(),
            border_color: Color::TRANSPARENT,
            border_radius: if *style == Location::Tooltip { 8.0 } else { 0.0 },
            ..Default::default()
        }
    }
}

pub enum RuleStyle {
    Location(Location),
    FillMode(FillMode),
    Both(Location, FillMode),
}

impl Default for RuleStyle {
    fn default() -> Self {
        Self::Both(Location::Default, FillMode::Full)
    }
}

impl From<Location> for RuleStyle {
    fn from(value: Location) -> Self {
        Self::Location(value)
    }
}

impl From<FillMode> for RuleStyle {
    fn from(value: FillMode) -> Self {
        Self::FillMode(value)
    }
}

impl rule::StyleSheet for Theme {
    type Style = RuleStyle;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        let (style, fill_mode) = match *style {
            RuleStyle::Location(style) => (style, FillMode::Full),
            RuleStyle::FillMode(mode) => (Location::Default, mode),
            RuleStyle::Both(style, mode) => (style, mode),
        };
        let palette = self.palette2(style);
        rule::Appearance {
            color: palette.text.a(0.3),
            width: 1,
            radius: 0.0,
            fill_mode,
        }
    }
}

impl button::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.palette2(*style);
        button::Appearance {
            background: palette.button.into(),
            border_color: Color::TRANSPARENT,
            text_color: palette.text,
            border_radius: 4.0,
            ..Default::default()
        }
    }

    // todo this is the
    //  the what??
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.palette2(*style);
        button::Appearance {
            background: self.hover(palette.button).into(),
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.palette2(*style);
        button::Appearance {
            border_width: 1.0,
            border_color: palette.outline,
            ..self.hovered(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.palette2(*style);
        button::Appearance {
            background: self.disable(palette.button).into(),
            ..self.active(style)
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.palette2(*style);
        text_input::Appearance {
            background: self.hover_by(palette.background, 0.3).into(),
            // background: palette.surface.into(),
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            icon_color: self.value_color(style),
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.palette2(*style);
        text_input::Appearance {
            border_width: 1.0,
            border_color: palette.button,
            ..self.active(style)
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        self.disable(self.palette2(*style).text)
        // match style {
        //     // todo is this always good
        //     Location::Transparent => Color::TRANSPARENT,
        //     _ => Color::from_rgb(0.4, 0.4, 0.4),
        // }
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        self.palette2(*style).text
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.palette(style).disabled
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        self.palette(style).active
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_width: 1.0,
            border_color: self.palette(style).accent.a(0.3),
            ..self.focused(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            ..self.active(style)
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: &Self::Style) -> Scrollbar {
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

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> Scrollbar {
        let palette = self.palette(style);
        let active = self.active(style);
        if is_mouse_over_scrollbar {
            Scrollbar {
                background: palette.surface.darken(0.4).into(),
                scroller: Scroller {
                    color: palette.surface.darken(0.7),
                    ..active.scroller
                },
                ..active
            }
        } else {
            active
        }
    }
}

impl menu::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: &Self::Style) -> menu::Appearance {
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

    fn active(&self, style: &Self::Style) -> pick_list::Appearance {
        let palette = self.palette(style);
        pick_list::Appearance {
            text_color: palette.text,
            placeholder_color: palette.text,
            // todo what does this do
            handle_color: Color::TRANSPARENT,
            background: palette.surface.into(),
            border_color: Color::TRANSPARENT,
            border_radius: 3.0,
            border_width: 0.0,
        }
    }

    fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            background: self.palette(style).hovered.into(),
            ..self.active(style)
        }
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.palette(style);
        checkbox::Appearance {
            background: if is_checked {
                palette.active
            } else {
                palette.surface
            }.into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.active,
            // todo
            text_color: palette.text.into(),
            icon_color: palette.text,
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
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

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        let palette = self.palette(style);
        let color = palette.text.a(0.5);
        slider::Appearance {
            // todo this has to be transparent for TRANSPARENT
            rail: Rail {
                colors: (color, color),
                width: 2.0,
            },
            handle: Handle {
                shape: HandleShape::Circle { radius: 7.0 },
                color: palette.background,
                border_width: 1.0,
                // todo this has to be transparent for TRANSPARENT
                border_color: color,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        let mut appearance = self.active(style);
        appearance.handle.border_width = 1.5;
        appearance
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        let mut appearance = self.hovered(style);
        appearance.handle.border_color = self.palette(style).active;
        appearance.handle.border_width += 0.5;
        appearance
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = Location;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        let palette = self.palette(style);
        progress_bar::Appearance {
            background: palette.active.into(),
            bar: palette.active.into(),
            border_radius: 5.0,
        }
    }
}

impl tab_bar::StyleSheet for Theme {
    type Style = Location;

    fn active(&self, _style: Self::Style, is_active: bool) -> tab_bar::Appearance {
        let palette = self.palette(&Location::Default);
        tab_bar::Appearance {
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

    fn hovered(&self, _style: Self::Style, is_active: bool) -> tab_bar::Appearance {
        let palette = self.palette(&Location::Default);
        tab_bar::Appearance {
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

macro_rules! color {
    ($c:literal) => {
        Color::from_rgb(
            (($c >> 16) & 0xff) as f32 / 255.0,
            (($c >> 8)  & 0xff) as f32 / 255.0,
            ($c         & 0xff) as f32 / 255.0,
        )
    };
    ($r:literal $g:literal $b:literal) => {
        Color::from_rgb(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
        )
    };
}

mod dark {
    use iced::Color;

    use crate::theme::{Location, Palette, Palette2};
    use crate::utils::ColorExt;

    pub fn palette2(style: Location) -> Palette2 {
        match style {
            Location::Default => DEFAULT2,
            Location::Transparent => TRANSPARENT2,
            Location::SettingsBar => SETTINGS_BAR2,
            Location::Tooltip => TOOLTIP2,
            // todo
            Location::AdvancedSearch { enabled } => Palette2 {
                text: DEFAULT2.text.a(if enabled { 1.0 } else { 0.5 }),
                ..TRANSPARENT2
            },
            Location::Alternating { idx, highlight } => alternating2(idx, highlight),
        }
    }

    const DEFAULT2: Palette2 = Palette2 {
        text: Color::WHITE,
        background: color!(0x36393f),
        button: color!(0x6279ca),
        outline: Color::WHITE,
    };

    const TRANSPARENT2: Palette2 = Palette2 {
        text: Color::WHITE,
        background: Color::TRANSPARENT,
        button: Color::TRANSPARENT,
        outline: Color::TRANSPARENT,
    };

    const SETTINGS_BAR2: Palette2 = Palette2 {
        text: Color::WHITE,
        background: color!(0x2e2f37),
        button: Color::TRANSPARENT,
        outline: Color::TRANSPARENT,
    };

    const TOOLTIP2: Palette2 = Palette2 {
        background: Color {
            a: 0.8,
            ..DEFAULT2.background
        },
        ..DEFAULT2
    };

    fn alternating2(idx: usize, highlight: bool) -> Palette2 {
        let idx = idx % 2;
        let background = [
            DEFAULT2.background,
            color!(0x303335)
        ][idx];
        Palette2 {
            text: Color::WHITE,
            background,
            button: if highlight { background } else { Color::TRANSPARENT },
            outline: Color::TRANSPARENT,
        }
    }

    pub fn palette(style: &Location) -> Palette {
        match style {
            Location::Default => DEFAULT,
            Location::Transparent => Palette {
                text: DEFAULT.text,
                ..Palette::TRANSPARENT
            },
            Location::SettingsBar => SETTINGS_BAR,
            &Location::Alternating { idx, highlight } => alternating(idx, highlight),
            &Location::AdvancedSearch { enabled } => Palette {
                text: DEFAULT.text.a(if enabled { 1.0 } else { 0.5 }),
                ..Palette::TRANSPARENT
            },
            Location::Tooltip => Palette {
                background: DEFAULT.background.a(0.8),
                ..DEFAULT
            }
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

    const SETTINGS_BAR: Palette = Palette {
        text: Color::WHITE,
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
        ..Palette::TRANSPARENT
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
}

mod light {
    use iced::Color;

    use crate::theme::{Location, Palette, Palette2};
    use crate::utils::ColorExt;

    pub fn palette2(style: Location) -> Palette2 {
        match style {
            Location::Default => DEFAULT2,
            Location::Transparent => TRANSPARENT2,
            Location::SettingsBar => SETTINGS_BAR2,
            Location::Tooltip => TOOLTIP2,
            // todo
            Location::AdvancedSearch { enabled } => Palette2 {
                text: DEFAULT2.text.a(if enabled { 1.0 } else { 0.5 }),
                ..TRANSPARENT2
            },
            Location::Alternating { idx, highlight } => alternating2(idx, highlight),
        }
    }

    const DEFAULT2: Palette2 = Palette2 {
        text: Color::BLACK,
        background: color!(0xefefef),
        button: color!(0x728be5),
        outline: Color::BLACK,
    };

    const TRANSPARENT2: Palette2 = Palette2 {
        text: Color::BLACK,
        background: Color::TRANSPARENT,
        button: Color::TRANSPARENT,
        outline: Color::TRANSPARENT,
    };

    const SETTINGS_BAR2: Palette2 = Palette2 {
        text: Color::BLACK,
        background: color!(0xa5b0b0),
        button: Color::TRANSPARENT,
        outline: Color::TRANSPARENT,
    };

    const TOOLTIP2: Palette2 = Palette2 {
        background: Color {
            a: 0.8,
            ..DEFAULT2.background
        },
        ..DEFAULT2
    };

    fn alternating2(idx: usize, highlight: bool) -> Palette2 {
        let idx = idx % 2;
        let background = [
            DEFAULT2.background,
            color!(0xa5b0b0)
        ][idx];
        Palette2 {
            text: Color::BLACK,
            background,
            button: if highlight { background } else { Color::TRANSPARENT },
            outline: Color::TRANSPARENT,
        }
    }


    pub fn palette(style: &Location) -> Palette {
        match style {
            Location::Default => DEFAULT,
            Location::Transparent => Palette {
                text: DEFAULT.text,
                ..Palette::TRANSPARENT
            },
            Location::SettingsBar => SETTINGS_BAR,
            &Location::Alternating { idx, highlight } => alternating(idx, highlight),
            &Location::AdvancedSearch { enabled } => Palette {
                text: DEFAULT.text.a(if enabled { 1.0 } else { 0.5 }),
                ..Palette::TRANSPARENT
            },
            Location::Tooltip => Palette {
                background: DEFAULT.background.a(0.8),
                ..DEFAULT
            }
        }
    }

    const DEFAULT: Palette = Palette {
        text: Color::BLACK,
        background: color!(0xEF 0xEF 0xEF),
        surface: color!(0x99 0xa3 0xb5),
        accent: color!(0x0b 0x15 0x17),
        active: color!(0x72 0x8b 0xe5),
        hovered: color!(0x62 0x6f 0xaf),
        disabled: color!(0x52 0x59 0xa9),
    };

    const SETTINGS_BAR: Palette = Palette {
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
}