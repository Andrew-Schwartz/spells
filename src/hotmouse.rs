use std::ops::Sub;
use std::time::Instant;

use iced::mouse::{self, Button, Event};

#[derive(Default, Debug, Copy, Clone)]
pub struct State {
    pub pt: Pt,
    pub press: ButtonPress,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Pt(f32, f32);

impl Sub for Pt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pt(self.0 - rhs.0, self.1 - rhs.1)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ButtonPress {
    Left(Instant, Pt),
    Right(Instant, Pt),
    Middle(Instant, Pt),
    None,
}

impl Default for ButtonPress {
    fn default() -> Self {
        Self::None
    }
}

impl PartialEq<Button> for ButtonPress {
    fn eq(&self, other: &Button) -> bool {
        match self {
            ButtonPress::Left(_, _) => matches!(other, Button::Left),
            ButtonPress::Right(_, _) => matches!(other, Button::Right),
            ButtonPress::Middle(_, _) => matches!(other, Button::Middle),
            ButtonPress::None => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StateMessage {
    MoveTo(Pt),
    ButtonPress(fn(Instant, Pt) -> ButtonPress),
    ButtonRelease(iced::mouse::Button),
}

pub fn handle(event: mouse::Event) -> Option<crate::Message> {
    // println!("event = {:?}", event);
    match event {
        Event::CursorEntered | Event::CursorLeft => None,
        Event::CursorMoved { x, y } => Some(crate::Message::MouseState(StateMessage::MoveTo(Pt(x, y)))),
        Event::ButtonPressed(button) => match button {
            Button::Left => Some(ButtonPress::Left as fn(Instant, Pt) -> ButtonPress),
            Button::Right => Some(ButtonPress::Right as fn(Instant, Pt) -> ButtonPress),
            Button::Middle => Some(ButtonPress::Middle as fn(Instant, Pt) -> ButtonPress),
            Button::Other(_) => None,
        }.map(|ctor| crate::Message::MouseState(StateMessage::ButtonPress(ctor))),
        Event::ButtonReleased(button) => Some(crate::Message::MouseState(StateMessage::ButtonRelease(button))),
        Event::WheelScrolled { .. } => None,
    }
}

pub fn gesture(delta: Pt) -> Option<crate::Message> {
    // pixels/second
    let Pt(x, y) = delta;
    if (y / x).abs() < 0.3 {
        use crate::hotkey::{Message, Move};
        let delta = if x.is_sign_positive() { Move::Right } else { Move::Left };
        Some(crate::Message::Hotkey(Message::Move(delta, false)))
    } else {
        None
    }
}