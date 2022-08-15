use std::fmt::Display;

use iced::{Color, Length, widget::{horizontal_space, tooltip::Position, vertical_space}};
use iced::widget::{Column, Row};
use iced_aw::Icon;
use iced_native::widget::text;
use palette::{FromColor, Hsl, Srgb};

use crate::{Element, ICON_FONT, Text, Tooltip};

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer> SpacingExt for Column<'a, Message, Renderer> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(vertical_space(length.into()))
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer> SpacingExt for Row<'a, Message, Renderer> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(horizontal_space(length.into()))
    }
}

fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Srgb::from(color))
}

fn from_hsl(hsl: Hsl) -> Color {
    Srgb::from_color(hsl).into()
}

pub trait ColorExt {
    fn r(self, r: f32) -> Self;
    fn g(self, g: f32) -> Self;
    fn b(self, b: f32) -> Self;
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
    fn tooltip_at<S: ToString>(self, tooltip: S, position: Position) -> Tooltip<'a> {
        iced::widget::tooltip(self, tooltip, position)
    }

    fn tooltip<S: ToString>(self, tooltip: S) -> Tooltip<'a> {
        self.tooltip_at(tooltip, Position::FollowCursor)
    }
}

impl<'a, E: Into<Element<'a>>> TooltipExt<'a> for E {}

pub fn text_icon(icon: Icon) -> Text {
    text(icon).font(ICON_FONT)
}