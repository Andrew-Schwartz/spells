use std::cmp::min;
use std::sync::Arc;

use iced::{button, Button, Column, Container, Length, Row, Scrollable, scrollable, Space, Text, text_input, TextInput};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{SpellButtonTrait, SpellId, SPELLS};
use crate::style::Style;

#[derive(Debug, Copy, Clone)]
pub enum MoveSpell {
    Up,
    Down,
    Left,
    Right,
}

impl MoveSpell {
    pub fn is_negative(self) -> bool {
        matches!(self, Self::Up | Self::Left)
    }

    pub fn delta(self, num_cols: usize) -> usize {
        match self {
            Self::Up | Self::Down => num_cols,
            Self::Left | Self::Right => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SpellTab(usize),
    AddSpell(SpellId),
    RemoveSpell(SpellId),
    /// delta to move the spell
    MoveSpell(SpellId, MoveSpell),
    Search(String),
}

pub const TABS: usize = 11;

pub struct CharacterPage {
    pub name: Arc<str>,
    /// the spells this character knows, by level
    pub spells: [Vec<Spell>; 10],
    move_left: button::State,
    move_right: button::State,
    delete: button::State,
    pub tab: usize,
    tabs: [button::State; TABS],
    scroll: scrollable::State,
    pub search_state: text_input::State,
    search: String,
}

#[derive(Debug)]
pub struct Spell {
    pub spell: &'static crate::Spell,
    remove: button::State,
    left: button::State,
    right: button::State,
    up: button::State,
    down: button::State,
}

impl PartialEq for Spell {
    fn eq(&self, other: &Self) -> bool {
        self.spell == other.spell
    }
}

impl From<&'static crate::Spell> for Spell {
    fn from(spell: &'static crate::Spell) -> Self {
        Self {
            spell,
            remove: Default::default(),
            left: Default::default(),
            right: Default::default(),
            up: Default::default(),
            down: Default::default(),
        }
    }
}

impl CharacterPage {
    pub fn new(name: Arc<str>) -> Self {
        Self::with_spells(name, Default::default())
    }

    fn with_spells(name: Arc<str>, spells: [Vec<Spell>; 10]) -> Self {
        Self {
            name,
            spells,
            move_left: Default::default(),
            move_right: Default::default(),
            delete: Default::default(),
            tab: 0,
            tabs: Default::default(),
            scroll: Default::default(),
            search_state: text_input::State::focused(),
            search: Default::default(),
        }
    }

    pub fn add_spell(&mut self, spell: SpellId) {
        let spell = SPELLS.iter().find(|s| **s == spell).unwrap();
        let level = spell.level;
        let spell = spell.into();
        if !self.spells[level].contains(&spell) {
            self.spells[level].push(spell);
        }
    }

    /// returns true if the character should be saved now
    pub fn update(&mut self, message: Message, num_cols: usize) -> bool {
        match message {
            Message::SpellTab(level) => {
                self.tab = level;
                false
            }
            Message::AddSpell(id) => {
                self.add_spell(id);
                true
            }
            Message::RemoveSpell(id) => {
                let spells = &mut self.spells[id.level];
                let idx = spells.iter()
                    .position(|spell| spell.spell.name == id.name);
                idx.map_or(false, |idx| {
                    spells.remove(idx);
                    true
                })
            }
            Message::MoveSpell(id, move_spell) => {
                let spells = &mut self.spells[id.level];
                let idx = spells.iter()
                    .position(|spell| spell.spell.name == id.name);
                idx.map_or(false, |idx| {
                    let new_idx = if move_spell.is_negative() {
                        idx.saturating_sub(move_spell.delta(num_cols))
                    } else {
                        min(idx + move_spell.delta(num_cols), spells.len() - 1)
                    };
                    // let new_idx = max(0, min(new_idx, spells.len() - 1));
                    spells.swap(idx, new_idx);
                    true
                })
            }
            Message::Search(search) => {
                self.search = search;
                false
            }
        }
    }

    pub fn view(&mut self, num_cols: usize, style: Style) -> Container<crate::Message> {
        let Self {
            name,
            spells,
            move_left,
            move_right,
            delete,
            tab,
            tabs,
            scroll,
            search_state,
            search
        } = self;
        let selected_level = *tab;
        let character_name = Arc::clone(name);

        // row with details: delete, move tab, etc
        let delete_width = Length::Units(23);
        let details_row = Row::new()
            .spacing(5)
            .push(Space::with_width(Length::Fill))
            .push(Space::with_width(delete_width))
            .push(Button::new(move_left, Text::new("<"))
                .style(style)
                .on_press(crate::Message::MoveCharacter(Arc::clone(&character_name), -1)))
            .push(Text::new(name.to_string())
                .size(30))
            .push(Button::new(move_right, Text::new(">"))
                .style(style)
                .on_press(crate::Message::MoveCharacter(Arc::clone(&character_name), 1)))
            .push(Button::new(delete, Text::new("X"))
                .style(style)
                .on_press(crate::Message::DeleteCharacter(Arc::clone(&character_name)))
                .width(delete_width))
            .push(Space::with_width(Length::Fill));

        // spell tabs
        let make_button = |state, name, level| {
            let mut button = Button::new(state, Text::new(name))
                .style(style);
            if level != selected_level {
                button = button.on_press(crate::Message::Character(Arc::clone(&character_name), Message::SpellTab(level)));
            }
            button
        };
        let mut tabs_row = Row::new()
            .spacing(2)
            .push(Space::with_width(Length::Fill));

        // iterate through tabs, allowing for specific handling for "all" and "cantrip" tabs
        let mut iter = tabs.iter_mut();
        // todo show search params on the ALL page
        // all spells tab
        let all = iter.next().unwrap();
        tabs_row = tabs_row.push(make_button(all, "All".into(), 0));

        // attach spell levels
        let mut iter = iter.enumerate();
        // name cantrip tab
        let (_, cantrip) = iter.next().unwrap();
        tabs_row = tabs_row.push(make_button(cantrip, "Cantrip".to_string(), 1));

        // generic spell tab with some `level`
        for (level, state) in iter {
            tabs_row = tabs_row.push(make_button(state, level.to_string(), level + 1));
        }
        let tabs_row = tabs_row.push(Space::with_width(Length::Fill));

        // slightly cursed way to flatten spells if we're in the `all` tab
        let mut mut_spells = Vec::new();
        let search_row = if selected_level == 0 {
            let needle = search.to_lowercase();
            mut_spells.extend(
                spells.iter_mut()
                    .flatten()
                    .filter(|spell| spell.spell.name.to_lowercase().contains(&needle))
            );
            // only thing to focus on
            search_state.focus();
            Row::new()
                .push(Space::with_width(Length::Fill))
                .push(TextInput::new(
                    search_state,
                    "search for a spell",
                    &search,
                    {
                        let character_name = Arc::clone(&character_name);
                        move |s| crate::Message::Character(Arc::clone(&character_name), Message::Search(s))
                    },
                ).style(style).width(Length::FillPortion(4)))
                .push(Space::with_width(Length::Fill))
        } else {
            mut_spells.extend(&mut spells[selected_level - 1]);
            Row::new()
        };
        // let spells = &mut spells[selected_level];
        let spells = mut_spells;

        let len = spells.len();
        // attach relevant buttons to a spell
        fn spell_buttons(name: Arc<str>, spell: &mut Spell, idx: usize, len: usize, num_cols: usize, all_tab: bool) -> MoveButtons {
            MoveButtons {
                name,
                remove: &mut spell.remove,
                left: if all_tab || idx == 0 { None } else { Some(&mut spell.left) },
                right: if all_tab || idx == len - 1 { None } else { Some(&mut spell.right) },
                up: if all_tab || idx < num_cols { None } else { Some(&mut spell.up) },
                down: if all_tab || len - idx - 1 <= {
                    let a = len % num_cols;
                    let bottom_start_idx = if a == 0 { num_cols } else { a };
                    bottom_start_idx - 1
                } { None } else { Some(&mut spell.down) },
            }
        }

        let spells_col = (&spells.into_iter().enumerate().chunks(num_cols))
            .into_iter()
            .fold(Column::new().spacing(18), |spells_col, mut chunk| {
                let row = (0..num_cols).fold(Row::new(), |row, _| {
                    if let Some((idx, spell)) = chunk.next() {
                        row.push(spell.spell.view(spell_buttons(
                            Arc::clone(&character_name), spell, idx, len, num_cols, selected_level == 0,
                        ), style).width(Length::Fill))
                    } else {
                        row.push(Space::with_width(Length::Fill))
                    }
                });
                spells_col.push(row)
            });

        Container::new(Column::new()
            .spacing(12)
            .padding(20)
            .push(details_row)
            .push(tabs_row)
            .push(search_row)
            .push(Scrollable::new(scroll).push(spells_col))
        )
    }

    pub fn from_serialized(serialized: &SerializeCharacter) -> Self {
        let mut spells: [Vec<Spell>; 10] = Default::default();
        serialized.spells.iter()
            .filter_map(|name| SPELLS.iter().find(|spell| spell.name == *name))
            .map(Spell::from)
            .for_each(|spell| spells[spell.spell.level].push(spell));
        Self::with_spells(Arc::clone(&serialized.name), spells)
    }

    pub fn serialize(&self) -> SerializeCharacter<'static> {
        SerializeCharacter {
            name: Arc::clone(&self.name),
            spells: self.spells.iter()
                .flatten()
                .map(|spell| spell.spell.name)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializeCharacter<'a> {
    // safe to Deserialize Arc because we only ever do so once, when the program starts
    name: Arc<str>,
    #[serde(borrow)]
    spells: Vec<&'a str>,
}

struct MoveButtons<'a> {
    name: Arc<str>,
    remove: &'a mut button::State,
    left: Option<&'a mut button::State>,
    right: Option<&'a mut button::State>,
    up: Option<&'a mut button::State>,
    down: Option<&'a mut button::State>,
}

impl<'a> SpellButtonTrait<'a> for MoveButtons<'a> {
    fn view(self, id: SpellId, style: Style) -> Row<'a, crate::Message> {
        let mut row = Row::new().spacing(2);
        if let Some(left) = self.left {
            row = row.push(
                Button::new(left, Text::new("<").size(12))
                    .style(style)
                    .on_press(crate::Message::Character(Arc::clone(&self.name), Message::MoveSpell(id, MoveSpell::Left)))
            );
        }
        if let Some(up) = self.up {
            row = row.push(
                Button::new(up, Text::new("^").size(12))
                    .style(style)
                    .on_press(crate::Message::Character(Arc::clone(&self.name), Message::MoveSpell(id, MoveSpell::Up)))
            );
        }
        row = row.push(
            Button::new(self.remove, Text::new("Remove").size(12))
                .style(style)
                .on_press(crate::Message::Character(Arc::clone(&self.name), Message::RemoveSpell(id)))
        );
        if let Some(down) = self.down {
            row = row.push(
                Button::new(down, Text::new("v").size(12))
                    .style(style)
                    .on_press(crate::Message::Character(Arc::clone(&self.name), Message::MoveSpell(id, MoveSpell::Down)))
            );
        }
        if let Some(right) = self.right {
            row = row.push(
                Button::new(right, Text::new(">").size(12))
                    .style(style)
                    .on_press(crate::Message::Character(Arc::clone(&self.name), Message::MoveSpell(id, MoveSpell::Right)))
            );
        }
        row
    }
}