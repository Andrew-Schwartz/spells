use std::cmp::min;
use std::sync::Arc;

use iced::{Align, button, Button, Column, Container, Element, Length, Row, Rule, Scrollable, scrollable, Text, Tooltip};
use iced::tooltip::Position;
use iced_aw::{Icon, ICON_FONT, number_input, NumberInput};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{CustomSpell, find_spell, search, SpellButtons, SpellId, StArc, StaticCustomSpell};
use crate::search::{Mode, Searcher, SearchOptions, WithButton};
use crate::style::Style;
use crate::utils::SpacingExt;

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
    Search(search::Message),
    // level, num
    SlotsNum(usize, u32),
    SlotsReset(usize),
    SlotsCast(usize),
}

pub const TABS: usize = 11;

pub struct Character {
    pub name: Arc<str>,
    /// the spells this character knows, by level, and if it's prepared
    pub spells: [Vec<(Spell, bool)>; 10],
    /// slots (total, left) by level
    pub slots: [(u32, u32, (number_input::State, button::State, button::State)); 9],
}

impl Character {
    const MAX_SLOTS_BY_LEVEL: [u32; 9] = [4, 3, 3, 3, 3, 2, 2, 1, 1];

    pub fn from_serialized(serialized: &SerializeCharacter, custom: &[CustomSpell]) -> Self {
        let mut spells: [Vec<(Spell, bool)>; 10] = Default::default();
        serialized.spells.iter()
            .filter_map(|(name, prepared)| {
                find_spell(name, custom)
                    .map(Spell::from)
                    .map(|spell| (spell, *prepared))
            })
            .for_each(|spell_prepared| spells[spell_prepared.0.spell.level()].push(spell_prepared));
        let slots = serialized.slots.map_or_else(
            Default::default,
            |arr| arr.map(|(tot, left)| (tot, left, Default::default())),
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
                .map(|(spell, prepared)| (spell.spell.name(), *prepared))
                .collect(),
            slots: Some(self.slots.each_ref().map(|&(tot, left, _)| (tot, left))),
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
    all_scroll: scrollable::State,
    scroll: scrollable::State,
    pub search: SearchOptions,
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
        Self::from(Character { name, spells: Default::default(), slots: Default::default() })
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
            all_scroll: Default::default(),
            scroll: Default::default(),
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
            Message::PrepareAll(prepare) => {
                match self.tab {
                    0 => &mut self.character.spells[..],
                    t => &mut self.character.spells[t - 1..t],
                }.iter_mut()
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
                fn toggle<T: Ord>(vec: &mut Vec<WithButton<T>>, entry: T) {
                    if let Some(idx) = vec.iter().position(|WithButton { t, .. }| *t == entry) {
                        vec.remove(idx);
                    } else {
                        vec.push(WithButton::new(entry));
                        vec.sort();
                    }
                }

                let search = match search {
                    search::Message::Search(search) => {
                        // self.c
                        self.search.search = search;
                        true
                    }
                    search::Message::Refresh => {
                        // self.search.search()
                        false
                    }
                    search::Message::PickMode(mode) => {
                        match mode {
                            Mode::Level => SearchOptions::toggle_mode(&mut self.search.level_search),
                            Mode::Class => SearchOptions::toggle_mode(&mut self.search.class_search),
                            Mode::School => SearchOptions::toggle_mode(&mut self.search.school_search),
                            Mode::CastingTime => SearchOptions::toggle_mode(&mut self.search.casting_time_search),
                            Mode::Ritual => {
                                SearchOptions::toggle_mode(&mut self.search.ritual_search);
                                // the default (false) will still match spells, so redo the search
                                // self.spells = self.search.search(custom, characters);
                            }
                            Mode::Concentration => SearchOptions::toggle_mode(&mut self.search.concentration_search),
                            Mode::Text => SearchOptions::toggle_mode(&mut self.search.text_search),
                            Mode::Source => SearchOptions::toggle_mode(&mut self.search.source_search),
                        }
                        false
                    }
                    search::Message::ResetModes => {
                        self.search.level_search = None;
                        self.search.class_search = None;
                        self.search.casting_time_search = None;
                        self.search.school_search = None;
                        self.search.ritual_search = None;
                        self.search.concentration_search = None;
                        self.search.text_search = None;
                        self.search.source_search = None;
                        false
                    }
                    search::Message::CollapseAll => todo!(),
                    search::Message::Collapse(_id) => todo!(),
                    search::Message::PickLevel(level) => self.search.level_search
                        .as_mut()
                        .map(|levels| toggle(&mut levels.levels, level))
                        .is_some(),
                    search::Message::PickClass(class) => self.search.class_search
                        .as_mut()
                        .map(|classes| toggle(&mut classes.classes, class))
                        .is_some(),
                    search::Message::PickCastingTime(casting_time) => self.search.casting_time_search
                        .as_mut()
                        .map(|times| toggle(&mut times.times, casting_time))
                        .is_some(),
                    search::Message::PickSchool(school) => self.search.school_search.as_mut()
                        .map(|schools| toggle(&mut schools.schools, school))
                        .is_some(),
                    search::Message::PickSource(source) => self.search.source_search.as_mut()
                        .map(|sources| toggle(&mut sources.sources, source))
                        .is_some(),
                    search::Message::ToggleRitual(ritual) => self.search.ritual_search.as_mut()
                        .map(|search| search.ritual = ritual)
                        .is_some(),
                    search::Message::ToggleConcentration(conc) => self.search.concentration_search.as_mut()
                        .map(|search| search.concentration = conc)
                        .is_some(),
                    search::Message::SearchText(text) => self.search.text_search.as_mut()
                        .map(|search| search.text = text.to_lowercase())
                        .is_some(),
                };
                if search {
                    // todo!()
                }
                false
            }
            Message::SlotsNum(level, n) => {
                println!("n = {:?}", n);
                let (tot, left, _) = &mut self.character.slots[level - 1];
                *tot = n;
                *left = (*left).clamp(0, n);
                println!("*left = {:?}", *left);
                true
            }
            Message::SlotsReset(level) => {
                println!("reset level {level}");
                let (tot, left, _) = &mut self.character.slots[level - 1];
                *left = *tot;
                true
            }
            Message::SlotsCast(level) => {
                println!("cast level {level}");
                let (tot, left, _) = &mut self.character.slots[level - 1];
                *left += 1;
                *left = (*left).clamp(0, *tot);
                true
            }
        }
    }

    pub fn view(&mut self, index: usize, num_cols: usize, style: Style) -> Container<crate::Message> {
        let message = move |message: Message| crate::Message::Character(index, message);

        let Self {
            character: Character {
                name,
                spells,
                slots,
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
            all_scroll,
            scroll,
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
                    .on_press(message(Message::ToggleCollapseAll)),
                if *should_collapse_all { "Expand all spells" } else { "Collapse all spells" },
                Position::FollowCursor,
            ))
            .push(Tooltip::new(
                Button::new(
                    collapse_unprepared,
                    Text::new(if *should_collapse_unprepared { Icon::ChevronExpand } else { Icon::ChevronContract })
                        .font(ICON_FONT))
                    .style(style)
                    .on_press(message(Message::ToggleCollapse)),
                if *should_collapse_unprepared { "Expand unprepared spells" } else { "Collapse unprepared spells" },
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(prepare_all, Text::new(Icon::Check).font(ICON_FONT))
                    .style(style)
                    .on_press(message(Message::PrepareAll(true))),
                "Prepare All",
                Position::FollowCursor))
            .push(Tooltip::new(
                Button::new(unprepare_all, Text::new(Icon::X).font(ICON_FONT))
                    .style(style)
                    .on_press(message(Message::PrepareAll(false))),
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
                button = button.on_press(message(Message::SpellTab(level)));
            }
            button
        };
        let mut tabs_row = Row::new()
            .push_space(Length::Fill);

        // iterate through tabs, allowing for specific handling for "all" and "cantrip" tabs
        let mut iter = tabs.iter_mut();
        // all spells tab
        let all = iter.next().unwrap();
        tabs_row = tabs_row.push(make_button(all, "All".into(), 0));

        // attach spell levels
        // name cantrip tab
        let cantrip = iter.next().unwrap();
        tabs_row = tabs_row.push(make_button(cantrip, "Cantrip".to_string(), 1));

        // generic spell tab with some `level`
        for (level, state) in iter.enumerate() {
            let level = level + 1;
            // spaces to pad the tab width
            tabs_row = tabs_row.push(make_button(state, format!(" {} ", level), level + 1));
        }
        let tabs_row = tabs_row.push_space(Length::Fill);

        let page: Element<crate::Message> = if selected_level == 0 {
            // 'All' tab
            let needle = search.search.to_lowercase();

            let group_by = spells.iter_mut().flatten().group_by(|(s, _)| s.spell.level());
            let scroll = (&group_by).into_iter()
                .zip(slots)
                .map(|((level, g), stuff)| (
                    level,
                    stuff,
                    g.filter(|(spell, _)| [
                        search.level_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.class_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.school_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.casting_time_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.ritual_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.concentration_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.text_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                        search.source_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                    ].into_iter()
                        .flatten()
                        .filter(|searcher| !searcher.is_empty())
                        .all(|searcher| searcher.matches(&spell.spell))
                    )
                        .filter(|(spell, _)| spell.spell.name_lower().contains(&needle))
                        .fold(
                            Column::new().spacing(2),
                            |col, (spell, _)| col.push(Row::new()
                                .push_space(Length::Fill)
                                .push(Text::new(&*spell.spell.name()).size(15).width(Length::FillPortion(18)))
                                .push_space(Length::Fill)
                            ),
                        )
                ))
                .fold(
                    Scrollable::new(all_scroll).padding(20),
                    move |scroll, (level, (total, left, (num_state, reset_state, cast_state)), col)| {
                        // I really don't know why it doesn't know the type of col??
                        let col: Column<crate::Message> = col;
                        let mut row = Row::new().padding(2).align_items(Align::Center);
                        if level == 0 {
                            row = row.push_space(Length::Fill)
                                .push(Text::new("Cantrips").size(22).width(Length::FillPortion(18)))
                                .push_space(Length::Fill)
                        } else {
                            let slots = (0..*total).fold(
                                Row::new().spacing(2).align_items(Align::Center),
                                |row, i| row.push(
                                    Text::new(if i < *total - *left { Icon::DiamondFill } else { Icon::Diamond })
                                        .font(ICON_FONT)
                                        .vertical_alignment(VerticalAlignment::Center)
                                        .size(15)
                                ),
                            );
                            row = row.push_space(Length::Fill)
                                .push(Row::new().width(Length::FillPortion(18)).align_items(Align::Center)
                                    .push(Text::new(level.to_string()).size(22))
                                    .push_space(40)
                                    .push(NumberInput::new(
                                        num_state,
                                        *total,
                                        Character::MAX_SLOTS_BY_LEVEL[level - 1],
                                        move |n| {
                                            println!("n = {:?}", n);
                                            crate::Message::Character(index, Message::SlotsNum(level, n))
                                        },
                                    ).style(style))
                                    .push_space(20)
                                    .push(Button::new(
                                        reset_state,
                                        Text::new(Icon::ArrowClockwise).font(ICON_FONT).size(22),
                                    ).on_press(crate::Message::Character(index, Message::SlotsReset(level)))
                                        .style(style))
                                    .push_space(20)
                                    .push(Button::new(
                                        cast_state,
                                        Text::new(Icon::Lightning).font(ICON_FONT).size(22),
                                    ).on_press(crate::Message::Character(index, Message::SlotsCast(level)))
                                        .style(style))
                                    .push_space(Length::Fill)
                                    .push(slots)
                                )
                                .push_space(Length::Fill)
                        }
                        scroll.push(Rule::horizontal(0))
                            .push(row)
                            .push(Rule::horizontal(0))
                            .spacing(6)
                            .push(col)
                    },
                );

            // let (_, scroll) = spells.iter_mut()
            //     .flatten()
            //     .filter(|(spell, _)| [
            //         search.level_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.class_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.school_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.casting_time_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.ritual_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.concentration_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.text_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //         search.source_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            //     ].into_iter()
            //         .flatten()
            //         .filter(|searcher| !searcher.is_empty())
            //         .all(|searcher| searcher.matches(&spell.spell)))
            //     .filter(|(spell, _)| spell.spell.name_lower().contains(&needle))
            //     // .sorted_unstable_by_key(|(spell, _)| spell.spell.name())
            //     .fold(
            //         (0, Scrollable::new(all_scroll).padding(20)),
            //         |(mut next_level, mut scroll), (spell, prepared)| {
            //             let level = spell.spell.level();
            //             if level >= next_level {
            //                 let mut row = Row::new().padding(2).align_items(Align::Center);
            //                 if level == 0 {
            //                     row = row.push_space(Length::Fill)
            //                         .push(Text::new("Cantrips").size(22).width(Length::FillPortion(18)))
            //                         .push_space(Length::Fill)
            //                 } else {
            //                     let (total, left, (num_state, reset_state, cast_state)) = &mut slots[level - 1];
            //                     let slots = (0..*total).fold(
            //                         Row::new().spacing(2).align_items(Align::Center),
            //                         |row, i| row.push(
            //                             Text::new(if i < *total - *left { Icon::DiamondFill } else { Icon::Diamond })
            //                                 .font(ICON_FONT)
            //                                 .vertical_alignment(VerticalAlignment::Center)
            //                                 .size(15)
            //                         ),
            //                     );
            //                     row = row.push_space(Length::Fill)
            //                         .push(Row::new().width(Length::FillPortion(18)).align_items(Align::Center)
            //                             .push(Text::new(level.to_string()).size(22))
            //                             .push_space(40)
            //                             .push(NumberInput::new(
            //                                 num_state,
            //                                 *total,
            //                                 Character::MAX_SLOTS_BY_LEVEL[level - 1],
            //                                 |n| crate::Message::Character(index, Message::SlotsNum(level, n)),
            //                             ).style(style))
            //                             .push_space(20)
            //                             .push(Button::new(
            //                                 reset_state,
            //                                 Text::new(Icon::ArrowClockwise).font(ICON_FONT).size(22),
            //                             ).on_press(crate::Message::Character(index, Message::SlotsReset(level)))
            //                                 .style(style))
            //                             .push_space(20)
            //                             .push(Button::new(
            //                                 cast_state,
            //                                 Text::new(Icon::Lightning).font(ICON_FONT).size(22),
            //                             ).on_press(crate::Message::Character(index, Message::SlotsCast(level)))
            //                                 .style(style))
            //                             .push_space(Length::Fill)
            //                             .push(slots)
            //                         )
            //                         .push_space(Length::Fill)
            //                 }
            //                 scroll = scroll.push(Rule::horizontal(2))
            //                     .push(row)
            //                     .push(Rule::horizontal(2));
            //                 next_level = level + 1;
            //             }
            //             scroll = scroll.push(Row::new()
            //                 .push_space(Length::Fill)
            //                 .push(Text::new(&*spell.spell.name()).size(15).width(Length::FillPortion(18)))
            //                 .push_space(Length::Fill)
            //             );
            //             (next_level, scroll)
            //         });

            Row::new()
                .align_items(Align::Start)
                .push(scroll.width(Length::Fill))
                .push_space(30)
                .push_space(Length::FillPortion(2))
                .into()
        } else {
            let len = spells[selected_level - 1].len();
            let chunks = spells[selected_level - 1].iter_mut()
                .enumerate()
                .chunks(num_cols);
            (&chunks).into_iter()
                .fold(Column::new().spacing(18), |spells_col, mut chunk| {
                    let row = (0..num_cols).fold(Row::new(), |row, _| {
                        if let Some((idx, (spell, prepared))) = chunk.next() {
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
                }).into()
        };

        // // slightly cursed way to flatten spells if we're in the `all` tab
        // let (spells, search_col, level) = if selected_level == 0 {
        //     let needle = search.search.to_lowercase();
        //     let spells = spells.iter_mut()
        //         .flatten()
        //         .filter(|(spell, _)| [
        //             search.level_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.class_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.school_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.casting_time_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.ritual_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.concentration_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.text_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //             search.source_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        //         ].into_iter()
        //             .flatten()
        //             .filter(|searcher| !searcher.is_empty())
        //             .all(|searcher| searcher.matches(&spell.spell)))
        //         .filter(|(spell, _)| spell.spell.name_lower().contains(&needle))
        //         // .sorted_unstable_by_key(|(spell, _)| spell.spell.name())
        //         .collect_vec();
        //     // let spells = spells.iter_mut()
        //     //     .flatten()
        //     //     .filter(|(spell, _)| spell.spell.name().to_lowercase().contains(&needle))
        //     //     .collect_vec();
        //     // only thing to focus on
        //     search.state.focus();
        //     let search_col = Column::new()
        //         .align_items(Align::Center)
        //         .push(search.view(
        //             None,
        //             move |s| message(Message::Search(search::Message::Search(s))),
        //             move |m| message(Message::Search(search::Message::PickMode(m))),
        //             message(Message::Search(search::Message::ResetModes)),
        //             Some(index),
        //             style,
        //         ));
        //     (spells, search_col, None)
        // } else {
        //     (spells[selected_level - 1].iter_mut().collect(), Column::new(), (selected_level >= 2).then(|| selected_level - 2))
        // };
        //
        // let len = spells.len();
        //
        // let spells_col = if num_cols == 0 {
        //     Column::new()
        // } else {
        //     (&spells.into_iter().enumerate().chunks(num_cols))
        //         .into_iter()
        //         .fold(Column::new().spacing(18), |spells_col, mut chunk| {
        //             let row = (0..num_cols).fold(Row::new(), |row, _| {
        //                 if let Some((idx, (spell, prepared))) = chunk.next() {
        //                     // let spell: &mut Spell = spell;
        //                     let all_tab = selected_level == 0;
        //                     let button = CharacterPageButtons {
        //                         character: index,
        //                         name: &mut spell.name,
        //                         prepare: &mut spell.prepare,
        //                         remove: &mut spell.remove,
        //                         left: if all_tab || idx == 0 { None } else { Some(&mut spell.left) },
        //                         right: if all_tab || idx == len - 1 { None } else { Some(&mut spell.right) },
        //                         up: if all_tab || idx < num_cols { None } else { Some(&mut spell.up) },
        //                         down: if all_tab || len - idx - 1 <= {
        //                             // this works but really... whyyyyyy is it a block
        //                             let a = len % num_cols;
        //                             let bottom_start_idx = if a == 0 { num_cols } else { a };
        //                             bottom_start_idx - 1
        //                         } { None } else { Some(&mut spell.down) },
        //                     };
        //                     let collapse = *should_collapse_all || (*should_collapse_unprepared && !*prepared);
        //                     row.push(spell.spell.view(button, *prepared, collapse, style).width(Length::Fill))
        //                 } else {
        //                     row.push_space(Length::Fill)
        //                 }
        //             });
        //             spells_col.push(row)
        //         })
        // };

        let scroll = Scrollable::new(scroll)
            .push(page)
            .height(Length::Fill)
            ;

        Container::new(Column::new()
            .align_items(Align::Center)
            .spacing(6)
            .push_space(10)
            .push(name_text)
            .push(buttons_row)
            .push(tabs_row)
            // .push(search_col)
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
        ].into_iter()
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