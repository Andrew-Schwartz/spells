use std::cmp::min;
use std::sync::Arc;

use iced::{Alignment, Length, pure::{*, widget::*}};
use iced::alignment::Vertical;
use iced_aw::{Icon, ICON_FONT};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{CustomSpell, find_spell, search, SpellButtons, SpellId, StArc, StaticCustomSpell};
use crate::search::{Mode, Searcher, SearchOptions};
use crate::style::Style;
use crate::utils::{SpacingExt, text_icon, TooltipExt};

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
    // level, delta
    ChangeNumSlots(usize, i32),
    SlotsCast(usize),
    SlotsReset,
}

pub const TABS: usize = 11;

#[derive(Default)]
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
            .for_each(|spell_prepared| spells[spell_prepared.0.spell.level()].push(spell_prepared));
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
                .map(|(spell, prepared)| (spell.spell.name(), *prepared))
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
    should_collapse_all: bool,
    should_collapse_unprepared: bool,
    pub tab: usize,
    pub search: SearchOptions,
}

// todo does this need to exist now that it doesn't store state?
#[derive(Debug)]
pub struct Spell {
    pub spell: StaticCustomSpell,
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
            should_collapse_unprepared: true,
            tab: 0,
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
                fn toggle<T: Ord>(vec: &mut Vec<T>, entry: T) {
                    if let Some(idx) = vec.iter().position(|t| *t == entry) {
                        vec.remove(idx);
                    } else {
                        vec.push(entry);
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
            Message::ChangeNumSlots(level, delta) => {
                println!("Set slots num {} level {level}", self.character.name);
                println!("delta = {:?}", delta);
                let Slots { total, used, .. } = &mut self.character.slots[level];
                *total = total.saturating_add_signed(delta).clamp(0, Slots::MAX_BY_LEVEL[level - 1]);
                *used = (*used).clamp(0, *total);
                true
            }
            Message::SlotsCast(level) => {
                println!("Cast spell {} level {level}", self.character.name);
                let Slots { used, total, .. } = &mut self.character.slots[level];
                *used += 1;
                *used = (*used).clamp(0, *total);
                true
            }
            Message::SlotsReset => {
                println!("Reset slots {}", self.character.name);
                for slots in &mut self.character.slots {
                    slots.used = 0;
                }
                true
            }
        }
    }

    pub fn view<'s, 'c: 's>(&'s self, index: usize, num_cols: usize, style: Style) -> Container<'c, crate::Message> {
        let message = move |message: Message| crate::Message::Character(index, message);

        let Self {
            character: Character {
                name,
                spells,
                slots,
            },
            should_collapse_all,
            should_collapse_unprepared,
            tab,
            search
        } = self;
        let selected_level = *tab;

        // row with details: delete, move tab, etc
        let name_text = text(name.to_string()).size(30);
        let buttons_row = row()
            .spacing(6)
            .push_space(Length::Fill)
            .push(button(text_icon(Icon::ArrowClockwise))
                .style(style)
                .on_press(message(Message::SlotsReset))
                .tooltip("Long Rest"))
            .push(button(
                text_icon(if *should_collapse_all { Icon::ArrowsExpand } else { Icon::ArrowsCollapse }))
                .style(style)
                .on_press(message(Message::ToggleCollapseAll))
                .tooltip(if *should_collapse_all { "Expand all spells" } else { "Collapse all spells" }))
            .push(button(
                text_icon(if *should_collapse_unprepared { Icon::ChevronExpand } else { Icon::ChevronContract }))
                .style(style)
                .on_press(message(Message::ToggleCollapse))
                .tooltip(if *should_collapse_unprepared { "Expand unprepared spells" } else { "Collapse unprepared spells" }))
            .push(button(text_icon(Icon::Check))
                .style(style)
                .on_press(message(Message::PrepareAll(true)))
                .tooltip("Prepare All"))
            .push(button(text_icon(Icon::X))
                .style(style)
                .on_press(message(Message::PrepareAll(false)))
                .tooltip("Unprepare All"))
            .push(button(text_icon(Icon::ArrowLeft))
                .style(style)
                .on_press(crate::Message::MoveCharacter(index, -1))
                .tooltip("Move character left"))
            .push(button(text(Icon::ArrowRight))
                .style(style)
                .on_press(crate::Message::MoveCharacter(index, 1))
                .tooltip("Move character right"))
            .push(button(text(Icon::Archive))
                .style(style)
                .on_press(crate::Message::CloseCharacter(index))
                .tooltip("Close character"))
            .push_space(Length::Fill);

        // spell tabs
        let make_button = |name, level| {
            let mut button = button(text(name))
                .style(style.tab_button());
            if level != selected_level {
                button = button.on_press(message(Message::SpellTab(level)));
            }
            button
        };
        let mut tabs_row = row()
            .push_space(Length::Fill);

        tabs_row = tabs_row.push(make_button("All".into(), 0));
        tabs_row = tabs_row.push(make_button("Cantrip".to_string(), 1));
        // todo only go up to max level this character knows
        for level in 1..=9 {
            // spaces to pad the tab width
            tabs_row = tabs_row.push(make_button(format!(" {} ", level), level + 1));
        }
        let tabs_row = tabs_row.push_space(Length::Fill);

        let page: Element<'_, crate::Message> = if selected_level == 0 {
            // 'All' tab
            let needle = search.search.to_lowercase();

            let group_by = spells.iter().flatten().group_by(|(s, _)| s.spell.level());
            let col = (&group_by).into_iter()
                .zip(slots)
                // .zip(&mut slots[1..])
                .map(|((level, g), slots)| (
                    level,
                    slots,
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
                            column().spacing(2),
                            |col, (spell, _)| col.push(row()
                                .push_space(Length::Fill)
                                .push(text(&*spell.spell.name()).size(15).width(Length::FillPortion(18)))
                                .push_space(Length::Fill)
                            ),
                        )
                ))
                .fold(
                    column().padding(20),
                    move |scroll, (level, Slots { total, used }, col)| {
                        let mut slots_row = row().padding(2).align_items(Alignment::Center);
                        if level == 0 {
                            slots_row = slots_row.push_space(Length::Fill)
                                .push(text("Cantrips").size(22).width(Length::FillPortion(18)))
                                .push_space(Length::Fill)
                        } else {
                            let slot_max_picker = column().align_items(Alignment::Center)
                                .push(button(
                                    text(Icon::ArrowUp)
                                        .font(ICON_FONT)
                                        .size(10),
                                ).style(style.background())
                                    .padding(0)
                                    .on_press(message(Message::ChangeNumSlots(level, 1))))
                                .push(button(
                                    text(Icon::ArrowDown)
                                        .font(ICON_FONT)
                                        .size(10),
                                ).style(style.background())
                                    .padding(0)
                                    .on_press(message(Message::ChangeNumSlots(level, -1))));
                            let slots_text = format!("{empty}{filled}",
                                                     filled = Icon::DiamondFill.to_string().repeat(*used as usize),
                                                     empty = Icon::Diamond.to_string().repeat((*total - *used) as usize),
                            );
                            let slots = button(
                                text(slots_text)
                                    .font(ICON_FONT)
                                    .vertical_alignment(Vertical::Center)
                                    .size(15),
                            ).style(style.background())
                                .on_press(message(Message::SlotsCast(level)));
                            slots_row = slots_row.push_space(Length::Fill)
                                .push(row().width(Length::FillPortion(18)).align_items(Alignment::Center)
                                    .push(text(level.to_string()).size(24))
                                    .push_space(10)
                                    .push(slot_max_picker)
                                    .push_space(Length::Fill)
                                    .push(slots)
                                )
                                .push_space(Length::Fill)
                        }
                        scroll.push(horizontal_rule(0))
                            .push(slots_row)
                            .push(horizontal_rule(0))
                            .spacing(6)
                            .push(col)
                    },
                );

            /*            let (_, scroll) = spells.iter_mut()
                            .flatten()
                            .filter(|(spell, _)| [
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
                                .all(|searcher| searcher.matches(&spell.spell)))
                            .filter(|(spell, _)| spell.spell.name_lower().contains(&needle))
                            // .sorted_unstable_by_key(|(spell, _)| spell.spell.name())
                            .fold(
                                (0, Scrollable::new(all_scroll).padding(20)),
                                |(mut next_level, mut scroll), (spell, prepared)| {
                                    let level = spell.spell.level();
                                    if level >= next_level {
                                        let mut row = row().padding(2).align_items(Alignment::Center);
                                        if level == 0 {
                                            row = row.push_space(Length::Fill)
                                                .push(text("Cantrips").size(22).width(Length::FillPortion(18)))
                                                .push_space(Length::Fill)
                                        } else {
                                            let (total, left, (num_state, reset_state, cast_state)) = &mut slots[level - 1];
                                            let slots = (0..*total).fold(
                                                row().spacing(2).align_items(Alignment::Center),
                                                |row, i| row.push(
                                                    text(if i < *total - *left { Icon::DiamondFill } else { Icon::Diamond })
                                                        .font(ICON_FONT)
                                                        .vertical_alignment(Alignment::Center)
                                                        .size(15)
                                                ),
                                            );
                                            row = row.push_space(Length::Fill)
                                                .push(row().width(Length::FillPortion(18)).align_items(Alignment::Center)
                                                    .push(text(level.to_string()).size(22))
                                                    .push_space(40)
                                                    .push(NumberInput::new(
                                                        num_state,
                                                        *total,
                                                        Character::MAX_SLOTS_BY_LEVEL[level - 1],
                                                        |n| crate::Message::Character(index, Message::SlotsNum(level, n)),
                                                    ).style(style))
                                                    .push_space(20)
                                                    .push(Button::new(
                                                        reset_state,
                                                        text(Icon::ArrowClockwise).font(ICON_FONT).size(22),
                                                    ).on_press(crate::Message::Character(index, Message::SlotsReset(level)))
                                                        .style(style))
                                                    .push_space(20)
                                                    .push(Button::new(
                                                        cast_state,
                                                        text(Icon::Lightning).font(ICON_FONT).size(22),
                                                    ).on_press(crate::Message::Character(index, Message::SlotsCast(level)))
                                                        .style(style))
                                                    .push_space(Length::Fill)
                                                    .push(slots)
                                                )
                                                .push_space(Length::Fill)
                                        }
                                        scroll = scroll.push(Rule::horizontal(2))
                                            .push(row)
                                            .push(Rule::horizontal(2));
                                        next_level = level + 1;
                                    }
                                    scroll = scroll.push(row()
                                        .push_space(Length::Fill)
                                        .push(text(&*spell.spell.name()).size(15).width(Length::FillPortion(18)))
                                        .push_space(Length::Fill)
                                    );
                                    (next_level, scroll)
                                });
            */
            row()
                .align_items(Alignment::Start)
                .push(scrollable(col.width(Length::Fill)))
                .push_space(30)
                .push_space(Length::FillPortion(2))
                .into()
        } else {
            let len = spells[selected_level - 1].len();
            let chunks = spells[selected_level - 1].iter()
                .enumerate()
                .chunks(num_cols);
            (&chunks).into_iter()
                .fold(column().spacing(18), |spells_col, mut chunk| {
                    let row = (0..num_cols).fold(row(), |row, _| {
                        if let Some((idx, (spell, prepared))) = chunk.next() {
                            let all_tab = selected_level == 0;
                            let button = CharacterPageButtons {
                                character: index,
                                left: !(all_tab || idx == 0),
                                right: !(all_tab || idx == len - 1),
                                up: !all_tab && idx >= num_cols,
                                down: !all_tab && len - idx - 1 > {
                                    // this works but really... whyyyyyy is it a block
                                    let a = len % num_cols;
                                    let bottom_start_idx = if a == 0 { num_cols } else { a };
                                    bottom_start_idx - 1
                                },
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

        /*// slightly cursed way to flatten spells if we're in the `all` tab
        let (spells, search_col, level) = if selected_level == 0 {
            let needle = search.search.to_lowercase();
            let spells = spells.iter_mut()
                .flatten()
                .filter(|(spell, _)| [
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
                    .all(|searcher| searcher.matches(&spell.spell)))
                .filter(|(spell, _)| spell.spell.name_lower().contains(&needle))
                // .sorted_unstable_by_key(|(spell, _)| spell.spell.name())
                .collect_vec();
            // let spells = spells.iter_mut()
            //     .flatten()
            //     .filter(|(spell, _)| spell.spell.name().to_lowercase().contains(&needle))
            //     .collect_vec();
            // only thing to focus on
            search.state.focus();
            let search_col = column()
                .align_items(Alignment::Center)
                .push(search.view(
                    None,
                    move |s| message(Message::Search(search::Message::Search(s))),
                    move |m| message(Message::Search(search::Message::PickMode(m))),
                    message(Message::Search(search::Message::ResetModes)),
                    Some(index),
                    style,
                ));
            (spells, search_col, None)
        } else {
            (spells[selected_level - 1].iter_mut().collect(), column(), (selected_level >= 2).then(|| selected_level - 2))
        };

        let len = spells.len();

        let spells_col = if num_cols == 0 {
            column()
        } else {
            (&spells.into_iter().enumerate().chunks(num_cols))
                .into_iter()
                .fold(column().spacing(18), |spells_col, mut chunk| {
                    let row = (0..num_cols).fold(row(), |row, _| {
                        if let Some((idx, (spell, prepared))) = chunk.next() {
                            // let spell: &mut Spell = spell;
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
        };*/

        let scroll = scrollable(page)
            .height(Length::Fill);

        container(column()
            .align_items(Alignment::Center)
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

    fn view<'c>(self, id: SpellId, is_prepared: bool, style: Style) -> (Row<'c, crate::Message>, Element<'c, crate::Message>) {
        let character = self.character;
        let buttons = [
            (self.left, Icon::ArrowLeft, Message::MoveSpell(id.clone(), MoveSpell::Left)),
            (self.up, Icon::ArrowUp, Message::MoveSpell(id.clone(), MoveSpell::Up)),
            (true, if is_prepared { Icon::Check2 } else { Icon::X }, Message::Prepare(id.clone())),
            (true, Icon::Trash, Message::RemoveSpell(id.clone())),
            (self.down, Icon::ArrowDown, Message::MoveSpell(id.clone(), MoveSpell::Down)),
            (self.right, Icon::ArrowRight, Message::MoveSpell(id.clone(), MoveSpell::Right)),
        ].into_iter()
            .fold(row().spacing(2), |row, (enable, icon, msg)|
                if enable {
                    row.push(button(text(icon).size(12).font(ICON_FONT))
                        .style(style)
                        .on_press(crate::Message::Character(character, msg)))
                } else {
                    row
                });
        let name = button(
            text(&*id.name).size(36),
        ).width(Length::FillPortion(23))
            .on_press(crate::Message::Character(self.character, Message::Prepare(id)))
            .style(style.background())
            .into();
        (buttons, name)
    }
}