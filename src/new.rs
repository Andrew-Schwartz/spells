use std::mem;
use std::sync::Arc;

use iced::{button, Button, Column, Container, Length, Space, Text, text_input, TextInput};

use crate::style::Style;

#[derive(Debug, Clone)]
pub enum Message {
    Name(String),
    Submit,
}

pub struct NewPage {
    name: String,
    pub state: text_input::State,
    create: button::State,
}

impl Default for NewPage {
    fn default() -> Self {
        Self {
            name: Default::default(),
            state: text_input::State::focused(),
            create: Default::default(),
        }
    }
}

impl NewPage {
    pub fn update(&mut self, message: Message, characters: &[Arc<str>]) -> Option<Arc<str>> {
        match message {
            Message::Name(name) => {
                self.name = name;
                None
            }
            Message::Submit => {
                self.state.focus();
                if !self.name.is_empty() && !characters.iter().any(|n| **n == self.name) {
                    let name = Arc::from(mem::take(&mut self.name));
                    Some(name)
                } else {
                    // todo notify in gui somehow
                    println!("{} is already a character", self.name);
                    None
                }
            }
        }
    }

    pub fn view(&mut self, style: Style) -> Container<crate::Message> {
        let name = TextInput::new(
            &mut self.state,
            "Character Name",
            &self.name,
            |n| crate::Message::New(Message::Name(n)),
        ).style(style)
            .on_submit(crate::Message::New(Message::Submit));
        let button = Button::new(
            &mut self.create,
            Text::new("Create"),
        ).style(style)
            .on_press(crate::Message::New(Message::Submit));

        let col = Column::new()
            // todo probably make this not take the entire page
            .push(name)
            .push(Space::with_height(Length::Units(5)))
            .push(button);

        Container::new(col)
            .center_x()
            .center_y()
    }
}