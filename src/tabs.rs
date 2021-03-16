use std::sync::Arc;

use iced::{Button, Row, Text};
use iced::widget::button;

use crate::Message;
use crate::new::NewPage;
use crate::search::SearchPage;
use crate::style::Style;

pub struct Tabs {
    pub state: Tab,
    pub characters: Vec<(Arc<str>, button::State)>,
    search: button::State,
    new_character: button::State,
}

impl Tabs {
    pub fn new<I: Iterator<Item=Arc<str>>>(characters: I) -> Self {
        Self {
            state: Tab::Search,
            characters: characters.map(|name| (name, Default::default())).collect(),
            search: Default::default(),
            new_character: Default::default(),
        }
    }

    pub fn update(&mut self, active: Tab, search: &mut SearchPage, new: &mut NewPage) {
        match &active {
            Tab::Search => search.state.focus(),
            Tab::New => new.state.focus(),
            Tab::Character(_) => {}
        }
        self.state = active;
    }

    pub fn view(&mut self, style: Style) -> Row<Message> {
        fn button(state: &mut button::State, tab: Tab, style: Style, selected: bool) -> Button<Message> {
            let mut button = Button::new(state, Text::new(&tab))
                .style(style);
            if !selected {
                button = button.on_press(Message::SwitchTab(tab));
            }
            button
        }

        let mut row = Row::new()
            .spacing(2)
            // .push(Space::with_width(Length::FillPortion(6)))
            .push(button(&mut self.search, Tab::Search, style, self.state == Tab::Search));

        for (name, state) in &mut self.characters {
            let tab = Tab::Character(Arc::clone(name));
            let selected = self.state == tab;
            row = row.push(button(state, tab, style, selected));
        }

        let new_button = button(&mut self.new_character, Tab::New, style, self.state == Tab::New);
        row.push(new_button)
        // .push(Space::with_width(Length::FillPortion(6)))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tab {
    Search,
    Character(Arc<str>),
    New,
}

impl<'a> From<&'a Tab> for String {
    fn from(tab: &'a Tab) -> Self {
        match tab {
            Tab::Search => "Search".into(),
            Tab::Character(name) => name.to_string(),
            Tab::New => "+".into()
        }
    }
}