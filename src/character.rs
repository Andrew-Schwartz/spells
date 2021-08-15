use std::cmp::min;
use std::sync::Arc;

use iced::{Align, button, Button, Column, Container, Element, Length, Row, Scrollable, scrollable, Text, text_input, TextInput, Tooltip};
use iced::tooltip::Position;
use iced_aw::{Icon, ICON_FONT};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{CustomSpell, find_spell, SpellButtons, SpellId, StArc, StaticCustomSpell};
use crate::style::Style;
use crate::utils::{ArrayIterTemp, SpacingExt};

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
    ToggleCollapse,
    ToggleCollapseAll,
    Prepare(SpellId),
    PrepareAll(bool),
    SpellTab(usize),
    AddSpell(SpellId),
    RemoveSpell(SpellId),
    /// delta to move the spell
    MoveSpell(SpellId, MoveSpell),
    Search(String),
}

pub const TABS: usize = 11;

pub struct Character {
    pub name: Arc<str>,
    /// the spells this character knows, by level, and if it's prepared
    pub spells: [Vec<(Spell, bool)>; 10],
}

impl Character {
    pub fn from_serialized(serialized: &SerializeCharacter, custom: &[CustomSpell]) -> Self {
        let mut spells: [Vec<(Spell, bool)>; 10] = Default::default();
        serialized.spells.iter()
            .filter_map(|(name, prepared)| {
                find_spell(name, custom)
                    .map(Spell::from)
                    .map(|spell| (spell, *prepared))
            })
            .for_each(|spell_prepared| spells[spell_prepared.0.spell.level()].push(spell_prepared));
        Self {
            name: Arc::clone(&serialized.name),
            spells,
        }
    }

    pub fn serialize(&self) -> SerializeCharacter {
        SerializeCharacter {
            name: Arc::clone(&self.name),
            spells: self.spells.iter()
                .flatten()
                .map(|(spell, prepared)| (spell.spell.name(), *prepared))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializeCharacter {
    // todo make sure this is true
    // fine to Deserialize Arc because we only ever do so once, when the program starts
    name: Arc<str>,
    spells: Vec<(StArc<str>, bool)>,
}

pub struct CharacterPage {
    pub character: Character,
    should_collapse_all: bool,
    collapse_all: button::State,
    should_collapse_unprepared: bool,
    collapse_unprepared: button::State,
    prepare_all: button::State,
    unprepare_all: button::State,
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
    pub spell: StaticCustomSpell,
    name: button::State,
    prepare: button::State,
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

impl From<StaticCustomSpell> for Spell {
    fn from(spell: StaticCustomSpell) -> Self {
        Self {
            spell,
            name: Default::default(),
            prepare: Default::default(),
            remove: Default::default(),
            left: Default::default(),
            right: Default::default(),
            up: Default::default(),
            down: Default::default(),
        }
    }
}

impl From<Arc<str>> for CharacterPage {
    fn from(name: Arc<str>) -> Self {
        Self::from(Character { name, spells: Default::default() })
    }
}

impl From<Character> for CharacterPage {
    fn from(character: Character) -> Self {
        Self {
            character,
            should_collapse_all: false,
            collapse_all: Default::default(),
            should_collapse_unprepared: true,
            collapse_unprepared: Default::default(),
            prepare_all: Default::default(),
            unprepare_all: Default::default(),
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
}

impl CharacterPage {
    pub fn add_spell(&mut self, spell: StaticCustomSpell) {
        let level = spell.level();
        let spell = spell.into();
        if !self.character.spells[level].iter().any(|(s, _)| *s == spell) {
            self.character.spells[level].push((spell, true));
        }
    }

    /// returns true if the character should be saved now
    pub fn update(&mut self, message: Message, custom: &[CustomSpell], num_cols: usize) -> bool {
        match message {
            Message::ToggleCollapse => {
                self.should_collapse_unprepared = !self.should_collapse_unprepared;
                false
            }
            Message::ToggleCollapseAll => {
                self.should_collapse_all = !self.should_collapse_all;
                false
            }
            Message::Prepare(id) => {
                let spells = &mut self.character.spells[id.level];
                let idx = spells.iter()
                    .position(|(spell, _)| spell.spell.name() == &*id.name);
                idx.map_or(false, |idx| {
                    spells[idx].1 = !spells[idx].1;
                    true
                })
            }
            Message::SpellTab(level) => {
                self.tab = level;
                false
            }
            Message::AddSpell(id) => {
                let spell = find_spell(&id.name, custom).unwrap();
                self.add_spell(spell);
                true
            }
            Message::RemoveSpell(id) => {
                let spells = &mut self.character.spells[id.level];
                let idx = spells.iter()
                    .position(|(spell, _)| spell.spell.name() == &*id.name);
                idx.map_or(false, |idx| {
                    spells.remove(idx);
                    true
                })
            }
            Message::MoveSpell(id, move_spell) => {
                let spells = &mut self.character.spells[id.level];
                let idx = spells.iter()
                    .position(|(spell, _)| spell.spell.name() == &*id.name);
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
            Message::PrepareAll(prepare) => {
                match self.tab {
                    0 => &mut self.character.spells[..],
                    t => &mut self.character.spells[t - 1..t],
                }.iter_mut()
                    .flatten()
                    .for_each(|(_, prepared)| *prepared = prepare);
                true
            }
        }
    }

    pub fn view(&mut self, index: usize, num_cols: usize, style: Style) -> Container<crate::Message> {
        let Self {
            character: Character {
                name,
                spells,
            },
            should_collapse_all,
            collapse_all,
            should_collapse_unprepared,
            collapse_unprepared,
            prepare_all,
            unprepare_all,
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

        // row with details: delete, move tab, etc
        let name_text = Text::new(name.to_string()).size(30);
        let buttons_row = Row::new()
            .spacing(6)
            .push_space(Length::Fill)
            .push(Tooltip::new(
                Button::new(
                    collapse_all,
                    Text::new(if *should_collapse_all { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                        .font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::Character(index, Message::ToggleCollapseAll)),
                if *should_collapse_all { "Expand all spells" } else { "Collapse all spells" },
                Position::FollowCursor,
            ))
            .push(Tooltip::new(
                Button::new(
                    collapse_unprepared,
                    Text::new(if *should_collapse_unprepared { Icon::ChevronExpand } else { Icon::ChevronContract })
                        .font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::Character(index, Message::ToggleCollapse)),
                if *should_collapse_unprepared { "Expand unprepared spells" } else { "Collapse unprepared spells" },
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(prepare_all, Text::new(Icon::Check).font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::Character(index, Message::PrepareAll(true))),
                "Prepare All",
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(unprepare_all, Text::new(Icon::X).font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::Character(index, Message::PrepareAll(false))),
                "Unprepare All",
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(move_left, Text::new(Icon::ArrowLeft).font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::MoveCharacter(index, -1)),
                "Move character left",
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(move_right, Text::new(Icon::ArrowRight).font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::MoveCharacter(index, 1)),
                "Move character right",
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(delete, Text::new(Icon::Archive).font(ICON_FONT))
                    .style(style)
                    .on_press(crate::Message::CloseCharacter(index)),
                "Close character",
                Position::FollowCursor))
            .push_space(Length::Fill);

        // spell tabs
        let make_button = |state, name, level| {
            let mut button = Button::new(state, Text::new(name))
                .style(style.tab_button());
            if level != selected_level {
                button = button.on_press(crate::Message::Character(index, Message::SpellTab(level)));
            }
            button
        };
        let mut tabs_row = Row::new()
            // .spacing(2)
            .push_space(Length::Fill);

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
            // spaces to pad the tab width
            tabs_row = tabs_row.push(make_button(state, format!(" {} ", level), level + 1));
        }
        let tabs_row = tabs_row.push_space(Length::Fill);

        // slightly cursed way to flatten spells if we're in the `all` tab
        let mut mut_spells = Vec::new();
        let search_row = if selected_level == 0 {
            let needle = search.to_lowercase();
            mut_spells.extend(
                spells.iter_mut()
                    .flatten()
                    .filter(|(spell, _)| spell.spell.name().to_lowercase().contains(&needle))
            );
            // only thing to focus on
            search_state.focus();
            Row::new()
                .push_space(Length::Fill)
                .push(TextInput::new(
                    search_state,
                    "search for a spell",
                    search,
                    move |s| crate::Message::Character(index, Message::Search(s)),
                ).style(style).width(Length::FillPortion(4)))
                .push_space(Length::Fill)
        } else {
            mut_spells.extend(&mut spells[selected_level - 1]);
            Row::new()
        };
        // let spells = &mut spells[selected_level];
        let spells = mut_spells;

        let len = spells.len();

        #[allow(clippy::if_not_else)]
            let spells_col = if num_cols != 0 {
            (&spells.into_iter().enumerate().chunks(num_cols))
                .into_iter()
                .fold(Column::new().spacing(18), |spells_col, mut chunk| {
                    let row = (0..num_cols).fold(Row::new(), |row, _| {
                        if let Some((idx, (spell, prepared))) = chunk.next() {
                            // let mut spell: Spell = spell;
                            let all_tab = selected_level == 0;
                            let button = CharacterPageButtons {
                                character: index,
                                name: &mut spell.name,
                                prepare: &mut spell.prepare,
                                remove: &mut spell.remove,
                                left: if all_tab || idx == 0 { None } else { Some(&mut spell.left) },
                                right: if all_tab || idx == len - 1 { None } else { Some(&mut spell.right) },
                                up: if all_tab || idx < num_cols { None } else { Some(&mut spell.up) },
                                down: if all_tab || len - idx - 1 <= {
                                    // this works but really... whyyyyyy is it a block
                                    let a = len % num_cols;
                                    let bottom_start_idx = if a == 0 { num_cols } else { a };
                                    bottom_start_idx - 1
                                } { None } else { Some(&mut spell.down) },
                            };
                            let collapse = *should_collapse_all || (*should_collapse_unprepared && !*prepared);
                            row.push(spell.spell.view(button, *prepared, collapse, style).width(Length::Fill))
                        } else {
                            row.push_space(Length::Fill)
                        }
                    });
                    spells_col.push(row)
                })
        } else {
            Column::new()
        };

        let scroll = Scrollable::new(scroll)
            .push(spells_col.padding(20))
            .height(Length::Fill)
            ;

        Container::new(Column::new()
            .align_items(Align::Center)
            .spacing(6)
            .push_space(10)
            .push(name_text)
            .push(buttons_row)
            .push(tabs_row)
            .push(search_row)
            .push(scroll)
        )
    }
}

struct CharacterPageButtons<'a> {
    character: usize,
    name: &'a mut button::State,
    prepare: &'a mut button::State,
    remove: &'a mut button::State,
    left: Option<&'a mut button::State>,
    right: Option<&'a mut button::State>,
    up: Option<&'a mut button::State>,
    down: Option<&'a mut button::State>,
}

impl<'a> SpellButtons<'a> for CharacterPageButtons<'a> {
    /// if this spell is prepared right now
    type Data = bool;

    fn view(self, id: SpellId, is_prepared: bool, style: Style) -> (Row<'a, crate::Message>, Element<'a, crate::Message>) {
        let character = self.character;
        let buttons = [
            (self.left, Icon::ArrowLeft, Message::MoveSpell(id.clone(), MoveSpell::Left)),
            (self.up, Icon::ArrowUp, Message::MoveSpell(id.clone(), MoveSpell::Up)),
            (Some(self.prepare), if is_prepared { Icon::Check2 } else { Icon::X }, Message::Prepare(id.clone())),
            (Some(self.remove), Icon::Trash, Message::RemoveSpell(id.clone())),
            (self.down, Icon::ArrowDown, Message::MoveSpell(id.clone(), MoveSpell::Down)),
            (self.right, Icon::ArrowRight, Message::MoveSpell(id.clone(), MoveSpell::Right)),
        ].array_iter()
            .fold(Row::new().spacing(2), |row, (state, icon, msg)|
                if let Some(state) = state {
                    row.push(Button::new(state, Text::new(icon).size(12).font(ICON_FONT))
                        .style(style)
                        .on_press(crate::Message::Character(character, msg)))
                } else {
                    row
                });
        let name = Button::new(
            self.name,
            Text::new(&*id.name).size(36),
        ).width(Length::FillPortion(23))
            .on_press(crate::Message::Character(self.character, Message::Prepare(id)))
            .style(style.background())
            .into();
        (buttons, name)
    }
}