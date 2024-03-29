use std::cmp::min;
use std::iter;
use std::sync::Arc;

use iced::{Alignment, Length};
use iced::alignment::Vertical;
use iced::widget::{button, Column, container, horizontal_rule, scrollable, text};
use iced_core::Color;
use iced_native::widget::tooltip::Position;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{Container, Element, ICON_FONT, Level, Location, Row, search, SpellButtons, SpellId, Tap};
use crate::icon::Icon;
use crate::search::SearchOptions;
use crate::spells::spell::{CustomSpell, find_spell, Spell};
use crate::spells::static_arc::StArc;
use crate::utils::{SpacingExt, text_icon, TooltipExt};
use crate::widgets::click_button::ClickButton;

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

    pub fn delta(self, num_cols: usize, all_tab: bool) -> usize {
        match self {
            Self::Up | Self::Down if !all_tab => num_cols,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleCollapse,
    ToggleCollapseAll,
    Prepare(SpellId),
    PrepareAll(bool),
    SpellTab(Option<Level>),
    AddSpell(SpellId),
    RemoveSpell(SpellId),
    /// delta to move the spell
    MoveSpell(SpellId, MoveSpell),
    Search(search::Message),
    // level, delta
    ChangeNumSlots(Level, i32),
    SlotsCast(Level, i32),
    SlotsReset,
    ViewSpell(SpellId),
}

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Slots {
    total: u32,
    used: u32,
}

impl Slots {
    const MAX_BY_LEVEL: [u32; 9] = [4, 3, 3, 3, 3, 2, 2, 1, 1];
}

pub struct Character {
    pub name: Arc<str>,
    /// the spells this character knows, by level, and if it's prepared
    pub spells: [Vec<(Spell, bool)>; 10],
    /// slots (total, left) by level
    pub slots: [Slots; 9],
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
            .for_each(|spell_prepared| spells[spell_prepared.0.level()].push(spell_prepared));
        let slots = serialized.slots.map_or_else(
            Default::default,
            |arr| arr.map(|(total, used)| Slots { total, used }),
        );
        Self {
            name: Arc::clone(&serialized.name),
            spells,
            slots,
        }
    }

    pub fn serialize(&self) -> SerializeCharacter {
        SerializeCharacter {
            name: Arc::clone(&self.name),
            spells: self.spells.iter()
                .flatten()
                .map(|(spell, prepared)| (spell.name(), *prepared))
                .collect(),
            slots: Some(self.slots.each_ref().map(|&Slots { total, used, .. }| (total, used))),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializeCharacter {
    // todo make sure this is true
    // fine to Deserialize Arc because we only ever do so once, when the program starts
    name: Arc<str>,
    spells: Vec<(StArc<str>, bool)>,
    slots: Option<[(u32, u32); 9]>,
}

pub struct CharacterPage {
    pub character: Character,
    pub view_spell: Option<SpellId>,
    should_collapse_all: bool,
    should_collapse_unprepared: bool,
    pub tab: Option<Level>,
    pub search: SearchOptions,
    pub search_results: [Vec<usize>; 10],
}

impl From<Arc<str>> for CharacterPage {
    fn from(name: Arc<str>) -> Self {
        Self::from(Character { name, spells: Default::default(), slots: Default::default() })
    }
}

impl From<Character> for CharacterPage {
    fn from(character: Character) -> Self {
        let search_results = character.spells.each_ref()
            .map(|spells| (0..spells.len()).collect_vec());
        let view_spell = character.spells.iter()
            .flatten()
            .next()
            .map(|(s, _)| s.id());
        Self {
            character,
            view_spell,
            should_collapse_all: false,
            should_collapse_unprepared: true,
            tab: None,
            search: Default::default(),
            search_results,
        }
    }
}

impl CharacterPage {
    #[allow(clippy::cast_possible_truncation)]
    pub fn tab_index(&self) -> usize {
        match self.tab {
            None => 0,
            Some(level) => self.character.spells.iter()
                .enumerate()
                .map(|(level, spells)| (Level::from_u8(level as u8).unwrap(), spells))
                .filter(|(_, spells)| !spells.is_empty())
                .enumerate()
                .find(|&(_, (l, _))| l == level)
                .unwrap()
                .0 + 1,
        }
    }

    pub fn add_spell(&mut self, spell: Spell) {
        let level = spell.level();
        if !self.character.spells[level].iter().any(|(s, _)| *s == spell) {
            self.character.spells[level].push((spell, true));
        }
    }

    fn search(&mut self) {
        let needle = self.search.search.to_lowercase();
        self.search_results = self.character.spells.each_ref()
            .map(|spells| spells.iter()
                .enumerate()
                .filter(|(_, (spell, _))| self.search.searchers()
                    .into_iter()
                    .filter(|searcher| !searcher.is_empty())
                    .all(|searcher| searcher.matches(spell)))
                .filter(|(_, (spell, _))| spell.name_lower().contains(&needle))
                .map(|(index, _)| index)
                .collect_vec());
        let n_results = self.search_results.iter()
            .flatten()
            .count();
        if self.tab.is_none() && n_results == 1 {
            let id = self.search_results.iter()
                .enumerate()
                .find_map(|(level, indices)| indices.first().map(|&idx| &self.character.spells[level][idx].0))
                .map(Spell::id)
                .unwrap();
            self.view_spell = Some(id);
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
                    .position(|(spell, _)| spell.name() == &*id.name);
                idx.map_or(false, |idx| {
                    spells[idx].1 = !spells[idx].1;
                    true
                })
            }
            Message::PrepareAll(prepare) => {
                self.character.spells.iter_mut()
                    .flatten()
                    .for_each(|(_, prepared)| *prepared = prepare);
                true
            }
            Message::SpellTab(level) => {
                self.tab = level;
                false
            }
            Message::AddSpell(id) => {
                let spell = find_spell(&id.name, custom).unwrap();
                self.add_spell(spell);
                self.search();
                true
            }
            Message::RemoveSpell(id) => {
                let spells = &mut self.character.spells[id.level];
                let idx = spells.iter()
                    .position(|(spell, _)| spell.name() == &*id.name);
                if let Some(idx) = idx {
                    spells.remove(idx);
                    self.search();
                }
                idx.is_some()
            }
            Message::MoveSpell(id, move_spell) => {
                let spells = &mut self.character.spells[id.level];
                let idx = spells.iter()
                    .position(|(spell, _)| spell.name() == &*id.name);
                if let Some(idx) = idx {
                    let all_tab = self.tab.is_none();
                    let new_idx = if move_spell.is_negative() {
                        idx.saturating_sub(move_spell.delta(num_cols, all_tab))
                    } else {
                        min(idx + move_spell.delta(num_cols, all_tab), spells.len() - 1)
                    };
                    spells.swap(idx, new_idx);
                    self.search();
                }
                idx.is_some()
            }
            Message::Search(search) => {
                let search = self.search.update(search);
                if search {
                    self.search();
                }
                false
            }
            Message::ChangeNumSlots(level, delta) => {
                let level = level as usize;
                let Slots { total, used, .. } = &mut self.character.slots[level - 1];
                *total = total.saturating_add_signed(delta)
                    .clamp(0, Slots::MAX_BY_LEVEL[level - 1]);
                *used = (*used).clamp(0, *total);
                true
            }
            Message::SlotsCast(level, delta) => {
                let Slots { used, total, .. } = &mut self.character.slots[level as usize - 1];
                *used = used.saturating_add_signed(delta)
                    .clamp(0, *total);
                true
            }
            Message::SlotsReset => {
                for slots in &mut self.character.slots {
                    slots.used = 0;
                }
                true
            }
            Message::ViewSpell(id) => {
                self.view_spell = Some(id);
                false
            }
        }
    }

    pub fn view<'s, 'c: 's>(&'s self, index: usize, num_cols: usize, summary_tooltip: bool) -> Container<'c> {
        let message = move |message: Message| crate::Message::Character(index, message);

        let Self {
            character: Character {
                name,
                spells,
                slots,
            },
            view_spell,
            should_collapse_all,
            should_collapse_unprepared,
            tab,
            search,
            search_results,
        } = self;
        let selected_level = *tab;

        // row with details: delete, move tab, etc
        let name_text = text(name.to_string()).size(30);

        let buttons_row = row![
            Length::Fill,
            button(text_icon(Icon::ArrowClockwise))
                .on_press(message(Message::SlotsReset))
                .tooltip("Long Rest"),
            button(
                text_icon(if *should_collapse_all { Icon::ArrowsExpand } else { Icon::ArrowsCollapse }))
                .on_press(message(Message::ToggleCollapseAll))
                .tooltip(if *should_collapse_all { "Expand all spells" } else { "Collapse all spells" }),
            button(
                text_icon(if *should_collapse_unprepared { Icon::ChevronExpand } else { Icon::ChevronContract }))
                .on_press(message(Message::ToggleCollapse))
                .tooltip(if *should_collapse_unprepared { "Expand unprepared spells" } else { "Collapse unprepared spells" }),
            button(text_icon(Icon::Check))
                .on_press(message(Message::PrepareAll(true)))
                .tooltip("Prepare All"),
            button(text_icon(Icon::X))
                .on_press(message(Message::PrepareAll(false)))
                .tooltip("Unprepare All"),
            button(text_icon(Icon::ArrowLeft))
                .on_press(crate::Message::MoveCharacter(index, -1))
                .tooltip("Move character left"),
            button(text_icon(Icon::ArrowRight))
                .on_press(crate::Message::MoveCharacter(index, 1))
                .tooltip("Move character right"),
            button(text_icon(Icon::Archive))
                .on_press(crate::Message::CloseCharacter(index))
                .tooltip("Close character"),
            Length::Fill
        ].spacing(6);

        // spell tabs
        let make_button = |name, level| {
            let is_selected_tab = level == selected_level;
            let mut button = button(text(name))
                .style(if is_selected_tab { Location::Default } else { Location::Transparent });
            if !is_selected_tab {
                button = button.on_press(message(Message::SpellTab(level)));
            }
            button
        };

        let tabs_row = Level::ALL.into_iter()
            .filter(|&l| !spells[l].is_empty())
            .fold(
                row![
                    Length::Fill,
                    make_button(" All ".into(), None),
                ],
                |row, level| row.push(make_button(format!(" {level} "), Some(level))),
            ).push_space(Length::Fill);

        let page: Element<'_> = if let Some(level) = selected_level {
            let len = search_results[level].len();
            let chunks = search_results[level].iter()
                .map(|&idx| &spells[level][idx])
                .enumerate()
                .chunks(num_cols);
            (&chunks).into_iter()
                .fold(Column::new().spacing(18), |spells_col, mut chunk| {
                    let row = (0..num_cols).fold(row![], |row, _| {
                        if let Some((idx, (spell, prepared))) = chunk.next() {
                            let button = CharacterPageButtons {
                                character: index,
                                left: idx != 0,
                                right: idx != len - 1,
                                up: idx >= num_cols,
                                down: len - idx - 1 > {
                                    // this works but really... whyyyyyy is it a block
                                    let a = len % num_cols;
                                    let bottom_start_idx = if a == 0 { num_cols } else { a };
                                    bottom_start_idx - 1
                                },
                            };
                            let collapse = *should_collapse_all || (*should_collapse_unprepared && !*prepared);
                            row.push(spell.view(button, *prepared, collapse).width(Length::Fill))
                        } else {
                            row.push_space(Length::Fill)
                        }
                    });
                    spells_col.push(row)
                })
                .tap(scrollable)
                .into()
        } else {
            // 'All' tab
            let list_spells = search_results.iter()
                .zip(Level::ALL)
                // .enumerate()
                // cantrip always have no slot
                .zip(iter::once(&Slots::default()).chain(slots))
                .filter(|((indices, _), _)| !indices.is_empty())
                .map(|((indices, level), slots)| (
                    level,
                    slots,
                    indices.iter()
                        .map(|&idx| &spells[level as usize][idx])
                        .fold(
                            Column::new(),
                            |col, (spell, prepped)| {
                                col.push(text(&*spell.name())
                                    .size(18)
                                    .style({
                                        let selected = view_spell.as_ref().filter(|s| s.name == spell.name()).is_some();
                                        let selected_highlight = if selected { 0.8 } else { 1.0 };
                                        let prepared_opacity = if *prepped { 1.0 } else { 0.5 };
                                        Color {
                                            r: selected_highlight,
                                            g: selected_highlight,
                                            b: 1.0,
                                            a: prepared_opacity,
                                        }
                                    })
                                    .tap(|text| button(text))
                                    .style(Location::Transparent)
                                    .padding(0)
                                    .on_press(message(Message::ViewSpell(spell.id())))
                                    .tap_if_else(
                                        summary_tooltip,
                                        |b| b.tooltip_at(
                                            Position::Right,
                                            format!("{}     {}", spell.casting_time(), spell.duration().unwrap_or("")),
                                        ).into(),
                                        Element::from,
                                    )
                                )
                            },
                        )))
                .fold(
                    Column::new().padding(20),
                    move |col, (level, Slots { total, used }, spells_col)| {
                        let mut slots_row = row![].padding(2).align_items(Alignment::Center);
                        if level == Level::Cantrip {
                            slots_row = slots_row
                                .push(text(Level::Cantrip).size(26));
                        } else {
                            let slot_max_picker = Column::new().align_items(Alignment::Center)
                                .push(button(
                                    text_icon(Icon::ArrowUp)
                                        .size(10))
                                    .style(Location::Transparent)
                                    .padding(0)
                                    .tap_if_else(
                                        *total != Slots::MAX_BY_LEVEL[level],
                                        |b| b
                                            .on_press(message(Message::ChangeNumSlots(level, 1)))
                                            .tooltip("Gain a spell slot")
                                            .into(),
                                        Element::from,
                                    )
                                )
                                .push(button(
                                    text(Icon::ArrowDown)
                                        .font(ICON_FONT)
                                        .size(10))
                                    .style(Location::Transparent)
                                    .padding(0)
                                    .tap_if_else(
                                        *total != 0,
                                        |b| b
                                            .on_press(message(Message::ChangeNumSlots(level, -1)))
                                            .tooltip("Lose a spell slot")
                                            .into(),
                                        Element::from,
                                    )
                                );
                            let slots_text = format!(
                                "{empty}{filled}",
                                filled = Icon::DiamondFill.to_string().repeat(*used as usize),
                                empty = Icon::Diamond.to_string().repeat((*total - *used) as usize),
                            );
                            let slots = ClickButton::new(
                                text(slots_text)
                                    .font(ICON_FONT)
                                    .vertical_alignment(Vertical::Center)
                                    .size(15),
                            )
                                .style(Location::Transparent)
                                .padding([2, 3])
                                .on_left_press(message(Message::SlotsCast(level, 1)))
                                .on_right_press(message(Message::SlotsCast(level, -1)));
                            let uncast = button(
                                text_icon(Icon::ArrowDown)
                                    .size(15)
                            )
                                .style(Location::Transparent)
                                .padding(0)
                                .tap_if(*used != 0, |btn|
                                    btn.on_press(message(Message::SlotsCast(level, -1))),
                                );
                            slots_row = slots_row
                                .push(row![
                                    text(format!("{level} Level")).size(26),
                                    10,
                                    slot_max_picker,
                                    Length::Fill,
                                    slots,
                                ].align_items(Alignment::Center)
                                    .tap_if(*total != 0, |r| r.push(uncast))
                                );
                        }
                        col.push(horizontal_rule(0))
                            .push(slots_row)
                            .push(horizontal_rule(0))
                            .spacing(6)
                            .push(spells_col)
                    },
                );

            let view_spell = view_spell.as_ref()
                .and_then(|id| self.character.spells[id.level]
                    .iter()
                    .find(|(s, _)| s.name() == id.name))
                .map_or_else(|| container(""),
                             |(spell, prepared)| spell.view(CharacterPageButtons {
                                 character: index,
                                 left: false,
                                 right: false,
                                 // todo false if can't move up/down
                                 up: true,
                                 down: true,
                             }, *prepared, false));
            row![
                container(scrollable(list_spells)).width(Length::FillPortion(3)),
                container(scrollable(view_spell)).width(Length::FillPortion(4)).padding([0, 0, 10, 0])
            ].align_items(Alignment::Start)
                .into()
        };

        // let search_col = col![
        //     search.view(None, Some(index))
        // ].align_items(Alignment::Center);

        container(col![
            10,
            name_text,
            buttons_row,
            tabs_row,
            search.view(None, Some(index)),
            page
        ].align_items(Alignment::Center)
            .spacing(6))
    }
}

#[allow(clippy::struct_excessive_bools)]
struct CharacterPageButtons {
    character: usize,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl SpellButtons for CharacterPageButtons {
    /// if this spell is prepared right now
    type Data = bool;

    fn view<'c>(self, id: SpellId, data: Self::Data) -> (Row<'c>, Element<'c>) {
        let character = self.character;
        let buttons = [
            (self.left, "Move left", Icon::ArrowLeft, Message::MoveSpell(id.clone(), MoveSpell::Left)),
            (self.up, "Move up", Icon::ArrowUp, Message::MoveSpell(id.clone(), MoveSpell::Up)),
            (true, if data { "Unprepare" } else { "Prepare" }, if data { Icon::X } else { Icon::Check2 }, Message::Prepare(id.clone())),
            (true, "Remove", Icon::Trash, Message::RemoveSpell(id.clone())),
            (self.down, "Move down", Icon::ArrowDown, Message::MoveSpell(id.clone(), MoveSpell::Down)),
            (self.right, "Move right", Icon::ArrowRight, Message::MoveSpell(id.clone(), MoveSpell::Right)),
        ].into_iter()
            .fold(row!().spacing(2), |row, (enable, tooltip, icon, msg)|
                if enable {
                    row.push(button(text_icon(icon).size(12))
                        .on_press(crate::Message::Character(character, msg))
                        .tooltip(tooltip))
                } else {
                    row
                });
        let name = button(
            text(&*id.name).size(36),
        ).width(Length::FillPortion(23))
            .on_press(crate::Message::Character(self.character, Message::Prepare(id)))
            .style(Location::Transparent)
            .into();
        (buttons, name)
    }
}