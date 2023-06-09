use std::fmt::Display;
use std::ops::Not;

use iced::{Length, widget::tooltip::Position};
use iced::widget::{Column, Row};
use iced_aw::Icon;
use iced_core::Color;
use iced_native::widget::{horizontal_space, Space, text, vertical_space};
use palette::{FromColor, Hsl, Srgb};

use crate::{Element, ICON_FONT, Text, Tooltip};
use crate::style::Location;

// versions that get the spacing easier
macro_rules! col {
    () => {
        iced::widget::Column::new()
    };
    ($($x:expr), + $(,)?) => {
        iced::widget::Column::with_children(vec![$($crate::utils::DirectionalElement::<$crate::utils::ColDir>::into_element($x)),+])
    }
}

macro_rules! row {
    () => {
        iced::widget::Row::new()
    };
    ($($x:expr),+ $(,)?) => {
        iced::widget::Row::with_children(vec![$($crate::utils::DirectionalElement::<$crate::utils::RowDir>::into_element($x)),+])
    }
}

trait Dir {
    fn space(length: Length) -> Space;
}

pub enum ColDir {}

impl Dir for ColDir {
    fn space(length: Length) -> Space {
        vertical_space(length)
    }
}

pub enum RowDir {}

impl Dir for RowDir {
    fn space(length: Length) -> Space {
        horizontal_space(length)
    }
}

pub trait DirectionalElement<'a, Dir> {
    fn into_element(self) -> Element<'a>;
}

macro_rules! impl_directional_element {
    ($(
        $ty:path/*, $(<$lt:lifetime>)?*/
    );+ $(;)?) => {
        $(
            impl<'a, Dir> DirectionalElement<'a, Dir> for $ty {
                fn into_element(self) -> $crate::Element<'a> {
                    $crate::Element::from(self)
                }
            }
        )+
    };
}

impl_directional_element! {
    crate::Element<'a>;
    crate::TextInput<'a>;
    crate::Container<'a>;
    crate::Text<'a>;
    crate::Button<'a>;
    crate::ClickButton<'a>;
    crate::Row<'a>;
    crate::Column<'a>;
    crate::Tooltip<'a>;
    crate::Scrollable<'a>;
    crate::CheckBox<'a>;
    crate::Rule;
    crate::ProgressBar;
    iced::widget::Space;
}

impl<'a, T, Dir> DirectionalElement<'a, Dir> for crate::Slider<'a, T>
    where T: Copy + num_traits::cast::FromPrimitive + 'a,
          f64: From<T>,
{
    fn into_element(self) -> crate::Element<'a> {
        crate::Element::from(self)
    }
}

impl<'a, T, Dir> DirectionalElement<'a, Dir> for crate::PickList<'a, T>
    where T: Clone + Eq + Display + 'static,
          [T]: ToOwned<Owned=Vec<T>>,
{
    fn into_element(self) -> crate::Element<'a> {
        crate::Element::from(self)
    }
}

impl<'a, D: Dir> DirectionalElement<'a, D> for Length {
    fn into_element(self) -> Element<'a> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self))
    }
}

impl<'a, D: Dir> DirectionalElement<'a, D> for u16 {
    fn into_element(self) -> Element<'a> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self.into()))
    }
}

impl<'a, D: Dir> DirectionalElement<'a, D> for &'a str {
    fn into_element(self) -> Element<'a> {
        Text::new(self).into()
    }
}

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer> SpacingExt for Column<'a, Message, Renderer> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(iced::widget::vertical_space(length.into()))
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer> SpacingExt for Row<'a, Message, Renderer> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(iced::widget::horizontal_space(length.into()))
    }
}

fn to_hsl(color: Color) -> Hsl {
    // Hsl::from_color(<Srgb as From<Color>>::from(color))
    let Color { r, g, b, a: _ } = color;
    Hsl::from_color(Srgb::new(r, g, b))
}

fn from_hsl(hsl: Hsl) -> Color {
    // Srgb::from_color(hsl).into()
    let Srgb { red, green, blue, standard: _ } = Srgb::from_color(hsl);
    Color::from_rgb(red, green, blue)
}

pub trait ColorExt {
    fn r(self, r: f32) -> Self;
    fn g(self, g: f32) -> Self;
    fn b(self, b: f32) -> Self;
    /// 1.0 is fully opaque
    fn a(self, a: f32) -> Self;

    fn darken(self, amount: f32) -> Self;
    fn lighten(self, amount: f32) -> Self;
}

impl ColorExt for Color {
    fn r(mut self, r: f32) -> Self {
        self.r = r;
        self
    }

    fn g(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    fn b(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    /// 1.0 is fully opaque
    fn a(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    /// amount from 0 to 1
    fn darken(self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let mut hsl = to_hsl(self);

        hsl.lightness -= hsl.lightness * amount;

        from_hsl(hsl)
    }

    /// amount from 0 to 1
    fn lighten(self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let mut hsl = to_hsl(self);

        hsl.lightness += hsl.lightness * amount;

        from_hsl(hsl)
    }
}

pub trait TryRemoveExt<T> {
    fn try_remove(&mut self, index: usize) -> Option<T>;
}

impl<T> TryRemoveExt<T> for Vec<T> {
    fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.len() > index {
            Some(self.remove(index))
        } else {
            None
        }
    }
}

pub trait ListGrammaticallyExt: ExactSizeIterator + Sized {
    fn list_grammatically(self) -> String where Self::Item: Display {
        if self.len() == 0 { return String::new(); }
        let last = self.len() - 1;
        self.enumerate()
            .fold(String::new(), |mut acc, (i, new)| {
                if i != 0 {
                    acc.push_str(if i == last {
                        if i == 1 {
                            " and "
                        } else {
                            ", and "
                        }
                    } else {
                        ", "
                    });
                }
                acc = format!("{}{}", acc, new);
                acc
            })
    }
}

impl<T: Display, I: ExactSizeIterator<Item=T>> ListGrammaticallyExt for I {}

pub trait Tap {
    fn tap<T, F: FnOnce(Self) -> T>(self, f: F) -> T where Self: Sized {
        f(self)
    }

    fn tap_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self where Self: Sized {
        if condition {
            f(self)
        } else {
            self
        }
    }

    fn tap_if_some<T, F: FnOnce(Self, T) -> Self>(self, option: Option<T>, f: F) -> Self where Self: Sized {
        if let Some(t) = option {
            f(self, t)
        } else {
            self
        }
    }

    fn tap_ref<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T {
        f(self)
    }
}

impl<T> Tap for T {}

pub trait IterExt: Iterator + Sized {
    fn none<P: FnMut(Self::Item) -> bool>(mut self, predicate: P) -> bool {
        !self.any(predicate)
    }
}

impl<I: Iterator + Sized> IterExt for I {}

pub trait TooltipExt<'a>: Into<Element<'a>> {
    fn tooltip_at<S: ToString>(self, position: Position, tooltip: S) -> Tooltip<'a> {
        iced::widget::tooltip(self, tooltip, position)
            .size(16)
            .style(Location::Tooltip)
    }

    fn tooltip<S: ToString>(self, tooltip: S) -> Tooltip<'a> {
        self.tooltip_at(Position::FollowCursor, tooltip)
    }
}

impl<'a, E: Into<Element<'a>>> TooltipExt<'a> for E {}

pub fn text_icon(icon: Icon) -> Text<'static> {
    text(icon).font(ICON_FONT)
}

pub trait Toggle: Not<Output=Self> + Copy {
    fn toggle(&mut self) {
        *self = !*self;
    }
}

impl<T: Not<Output=T> + Copy> Toggle for T {}