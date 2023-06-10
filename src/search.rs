use std::convert::identity;
use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

use iced::{Alignment, Length};
use iced::widget::{button, container, scrollable, text, text_input};
use iced_native::Command;
use iced_native::widget::column;
use itertools::Itertools;

use crate::{character, Container, Element, ICON_FONT, Location, Row, Scrollable, SpellButtons, SpellId, SPELLS, Theme};
use crate::character::CharacterPage;
use crate::icon::Icon;
use crate::spells::data::{CastingTime, Class, Components, Level, School, Source};
use crate::spells::spell::{CustomSpell, Spell};
use crate::theme::types::Button;
use crate::utils::{IterExt, SpacingExt, Tap, text_icon, Toggle, TooltipExt};

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    CollapseAll,
    Collapse(SpellId),
    Search(String),
    // PickMode(Mode),
    ToggleAdvanced,
    ResetSearch,
    PickLevel(Level),
    PickCastingTime(CastingTime),
    PickClass(Class),
    PickSchool(School),
    PickSource(Source),
    ToggleRitual,
    ToggleRitualEnabled,
    ToggleConcentration,
    ToggleConcentrationEnabled,
    SearchText(String),
    ToggleComponent(usize),
    ToggleComponentEnabled(usize),
}

// pub trait PLNone {
//     fn title() -> &'static str;
// }

// pub trait Unwrap<T>: Sized {
//     fn unwrap(self) -> T;
// }
//
// impl<T> Unwrap<T> for Option<T> {
//     fn unwrap(self) -> T {
//         self.unwrap()
//     }
// }

#[derive(Debug, Default, Copy, Clone)]
pub struct Enable<T> {
    value: T,
    enabled: bool,
}

impl<T: Default> Enable<T> {
    fn clear(&mut self) {
        self.value = T::default();
        self.enabled = false;
    }
}

impl<T: PartialEq> PartialEq<T> for Enable<T> {
    fn eq(&self, other: &T) -> bool {
        !self.enabled || self.value == *other
    }
}

pub trait Searcher: Debug {
    fn clear(&mut self);

    fn is_empty(&self) -> bool;

    fn matches(&self, spell: &Spell) -> bool;

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c>;
}

fn wrap_character(character: Option<usize>, message: Message) -> crate::Message {
    match character {
        None => crate::Message::Search(message),
        Some(character) => crate::Message::Character(character, character::Message::Search(message))
    }
}

#[derive(Debug, Default)]
pub struct LevelSearch {
    pub levels: [bool; 10],
}

impl Searcher for LevelSearch {
    fn clear(&mut self) {
        self.levels = [false; 10];
    }

    fn is_empty(&self) -> bool {
        self.levels.into_iter().none(identity)
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.levels[spell.level() as usize]
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        iter::zip(self.levels, Level::ALL)
            .fold(
                row!["Levels:"].align_items(Alignment::Center).spacing(4),
                |row, (enabled, l)| row.push(
                    button(text(l).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled })
                        .on_press(wrap_character(character, Message::PickLevel(l)))
                ),
            )
    }
}

#[derive(Debug, Default)]
pub struct ClassSearch {
    pub classes: Vec<Class>,
}

impl Searcher for ClassSearch {
    fn clear(&mut self) {
        self.classes.clear();
    }

    fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        spell.classes().iter()
            .any(|class| self.classes.iter().any(|t| class == t))
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        Class::ALL.into_iter()
            .fold(
                row!["Classes:"].align_items(Alignment::Center).spacing(4),
                |row, class| row.push(
                    button(text(class).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled: self.classes.contains(&class) })
                        .on_press(wrap_character(character, Message::PickClass(class)))
                ),
            )
    }
}

#[derive(Debug, Default)]
pub struct CastingTimeSearch {
    pub times: Vec<CastingTime>,
}

impl Searcher for CastingTimeSearch {
    fn clear(&mut self) {
        self.times.clear();
    }

    fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.times.iter().any(|t|
            t.equals_ignore_reaction(spell.casting_time())
        )
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        const DURATIONS: [CastingTime; 10] = [
            CastingTime::Action,
            CastingTime::BonusAction,
            CastingTime::Reaction(None),
            CastingTime::Minute(1),
            CastingTime::Minute(10),
            CastingTime::Hour(1),
            CastingTime::Hour(8),
            CastingTime::Hour(12),
            CastingTime::Hour(24),
            CastingTime::Special,
        ];

        DURATIONS.into_iter()
            .fold(
                row!["Casting Time:"].align_items(Alignment::Center).spacing(4),
                |row, ct| row.push(
                    button(text(&ct).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled: self.times.contains(&ct) })
                        .on_press(wrap_character(character, Message::PickCastingTime(ct)))
                ),
            )
    }
}

#[derive(Debug, Default)]
pub struct SchoolSearch {
    pub schools: Vec<School>,
}

impl Searcher for SchoolSearch {
    fn clear(&mut self) {
        self.schools.clear();
    }

    fn is_empty(&self) -> bool {
        self.schools.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.schools.iter().any(|t| *t == spell.school())
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        School::ALL.into_iter()
            .fold(
                row!["School:"].align_items(Alignment::Center).spacing(4),
                |row, school| row.push(
                    button(text(school).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled: self.schools.contains(&school) })
                        .on_press(wrap_character(character, Message::PickSchool(school)))
                ),
            )
    }
}

#[derive(Debug, Default)]
pub struct RitualSearch {
    pub ritual: Enable<bool>,
}

impl Searcher for RitualSearch {
    fn clear(&mut self) {
        self.ritual.clear();
    }

    fn is_empty(&self) -> bool {
        !self.ritual.enabled
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.ritual == spell.ritual()
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        let Enable { value: ritual, enabled } = self.ritual;
        row![
            button(
                text("Ritual:")
            ).padding(0)
                .style(Location::AdvancedSearch { enabled })
                .on_press(wrap_character(character, Message::ToggleRitualEnabled))
                .tooltip("Enable ritual filtering"),
            button(
                text(if ritual { Icon::Check } else { Icon::X })
                    .font(ICON_FONT)
                    .size(15)
                    // .vertical_alignment(alignment::Vertical::Center)
            ).padding(0)
                .style(Location::AdvancedSearch { enabled })
                .tap_if(enabled, |b|
                    b.on_press(wrap_character(character, Message::ToggleRitual))
                )
        ].align_items(Alignment::Center)
            .spacing(4)
    }
}

#[derive(Debug, Default)]
pub struct ConcentrationSearch {
    pub concentration: Enable<bool>,
}

impl Searcher for ConcentrationSearch {
    fn clear(&mut self) {
        self.concentration.clear();
    }

    fn is_empty(&self) -> bool {
        !self.concentration.enabled
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.concentration == spell.concentration()
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        let Enable { value: concentration, enabled } = self.concentration;
        row![
            button(
                text("Concentration:")
            ).padding(0)
                .style(Location::AdvancedSearch { enabled })
                .on_press(wrap_character(character, Message::ToggleConcentrationEnabled))
                .tooltip("Enable concentration filtering"),
            button(
                text(if concentration { Icon::Check } else { Icon::X })
                    .font(ICON_FONT)
                    .size(15)
                    // .vertical_alignment(alignment::Vertical::Center)
            ).padding(0)
                .style(Location::AdvancedSearch { enabled })
                .tap_if(enabled, |b|
                    b.on_press(wrap_character(character, Message::ToggleConcentration))
                )
        ].align_items(Alignment::End)
            .spacing(4)
    }
}

#[derive(Debug)]
pub struct TextSearch {
    pub text: String,
    pub id: text_input::Id,
}

impl Default for TextSearch {
    fn default() -> Self {
        Self {
            text: Default::default(),
            id: text_input::Id::unique(),
        }
    }
}

impl Searcher for TextSearch {
    fn clear(&mut self) {
        self.text.clear();
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.text.split('|')
            .any(|search|
                spell.desc_lower().contains(search) ||
                    spell.higher_levels_lower()
                        .as_ref()
                        .filter(|lower| lower.contains(search))
                        .is_some()
            )
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        row![
            "Spell Text:",
            text_input(
                "int|wis",
                &self.text,
            ).on_input(move |s| wrap_character(character, Message::SearchText(s)))
        ].align_items(Alignment::Center)
            .spacing(4)
    }
}

#[derive(Debug, Default)]
pub struct SourceSearch {
    pub sources: Vec<Source>,
}

impl Searcher for SourceSearch {
    fn clear(&mut self) {
        self.sources.clear();
    }

    fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.sources.iter().any(|&t| t == spell.source())
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        Source::ALL.into_iter()
            .fold(
                row!["Source:"].align_items(Alignment::Center).spacing(4),
                |row, source| row.push(
                    button(text(source).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled: self.sources.contains(&source) })
                        .on_press(wrap_character(character, Message::PickSource(source)))
                ),
            )
    }
}

#[derive(Debug, Default)]
pub struct ComponentSearch {
    vsm: [Enable<bool>; 3],
}

impl Searcher for ComponentSearch {
    fn clear(&mut self) {
        self.vsm.iter_mut()
            .for_each(Enable::clear);
    }

    fn is_empty(&self) -> bool {
        self.vsm.iter()
            .none(|e| e.enabled)
    }

    fn matches(&self, spell: &Spell) -> bool {
        let vsm = match spell.components() {
            Some(Components { v, s, m }) => [*v, *s, m.is_some()],
            None => Default::default(),
        };
        iter::zip(self.vsm, vsm)
            .all(|(e, b)| e == b)
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        iter::zip(self.vsm, ["Verbal", "Somatic", "Material"])
            .enumerate()
            .fold(
                row!["Components:"].spacing(4).align_items(Alignment::Center),
                |row, (i, (Enable { value, enabled }, label))| row
                    .push_space(2)
                    .push(
                        button(text(label).size(15))
                            .padding(0)
                            .style(Location::AdvancedSearch { enabled })
                            .on_press(wrap_character(character, Message::ToggleComponentEnabled(i)))
                            .tooltip(format!("Enable {} filtering", label.to_ascii_lowercase()))
                    )
                    .push(
                        button(
                            text(if value { Icon::Check } else { Icon::X })
                                .font(ICON_FONT)
                                .size(15)
                        ).padding(0)
                            .style(Location::AdvancedSearch { enabled })
                            .tap_if(enabled, |b|
                                b.on_press(wrap_character(character, Message::ToggleComponent(i))),
                            )
                    ),
            )
    }
}

pub struct SearchOptions {
    pub search: String,
    pub id: text_input::Id,
    pub show_advanced_search: bool,
    pub level_search: LevelSearch,
    pub class_search: ClassSearch,
    pub casting_time_search: CastingTimeSearch,
    pub school_search: SchoolSearch,
    pub ritual_search: RitualSearch,
    pub concentration_search: ConcentrationSearch,
    pub source_search: SourceSearch,
    pub text_search: TextSearch,
    pub component_search: ComponentSearch,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            search: Default::default(),
            id: text_input::Id::unique(),
            show_advanced_search: false,
            level_search: Default::default(),
            class_search: Default::default(),
            casting_time_search: Default::default(),
            school_search: Default::default(),
            ritual_search: Default::default(),
            concentration_search: Default::default(),
            text_search: Default::default(),
            source_search: Default::default(),
            component_search: Default::default(),
        }
    }
}

impl SearchOptions {
    pub fn searchers(&self) -> [&dyn Searcher; 9] {
        [
            &self.level_search as &dyn Searcher,
            &self.class_search as &dyn Searcher,
            &self.school_search as &dyn Searcher,
            &self.casting_time_search as &dyn Searcher,
            &self.ritual_search as &dyn Searcher,
            &self.concentration_search as &dyn Searcher,
            &self.component_search as &dyn Searcher,
            &self.source_search as &dyn Searcher,
            &self.text_search as &dyn Searcher,
        ]
    }

    pub fn searchers_mut(&mut self) -> [&mut dyn Searcher; 9] {
        [
            &mut self.level_search as &mut dyn Searcher,
            &mut self.class_search as &mut dyn Searcher,
            &mut self.school_search as &mut dyn Searcher,
            &mut self.casting_time_search as &mut dyn Searcher,
            &mut self.ritual_search as &mut dyn Searcher,
            &mut self.concentration_search as &mut dyn Searcher,
            &mut self.component_search as &mut dyn Searcher,
            &mut self.source_search as &mut dyn Searcher,
            &mut self.text_search as &mut dyn Searcher,
        ]
    }

    pub fn search(&self, custom: &[CustomSpell], characters: &[CharacterPage]) -> Vec<SearchSpell> {
        let needle = &self.search;
        SPELLS.iter()
            .map(Spell::Static)
            .chain(custom.iter()
                // todo not clone them
                //  could Cow it?
                .cloned()
                .map(Spell::Custom))
            .filter(|spell| self.searchers()
                .into_iter()
                .filter(|searcher| !searcher.is_empty())
                .all(|searcher| searcher.matches(spell)))
            .filter(|spell| spell.name_lower().contains(needle))
            .sorted_unstable_by_key(Spell::name)
            // .sorted_unstable_by_key(|spell| levenshtein(spell.name_lower(), needle))
            .map(|spell| SearchSpell::from(spell, characters))
            .take(100)
            .collect()
    }

    pub fn update(&mut self, message: Message) -> bool {
        fn toggle<T: Ord>(vec: &mut Vec<T>, entry: T) {
            if let Some(idx) = vec.iter().position(|t| *t == entry) {
                vec.remove(idx);
            } else {
                vec.push(entry);
                vec.sort();
            }
        }

        match message {
            Message::Search(needle) => {
                self.search = needle.to_lowercase();
                true
            }
            Message::Refresh => {
                true
            }
            Message::ResetSearch => {
                self.search.clear();
                self.searchers_mut()
                    .into_iter()
                    .for_each(Searcher::clear);
                true
            }
            Message::PickLevel(level) => {
                self.level_search.levels[level as usize].toggle();
                true
            }
            Message::PickClass(class) => {
                toggle(&mut self.class_search.classes, class);
                true
            }
            Message::PickSchool(school) => {
                toggle(&mut self.school_search.schools, school);
                true
            }
            Message::PickCastingTime(casting_time) => {
                toggle(&mut self.casting_time_search.times, casting_time);
                true
            }
            Message::PickSource(source) => {
                toggle(&mut self.source_search.sources, source);
                true
            }
            Message::ToggleRitual => {
                self.ritual_search.ritual.value.toggle();
                true
            }
            Message::ToggleRitualEnabled => {
                self.ritual_search.ritual.enabled.toggle();
                true
            }
            Message::ToggleConcentration => {
                self.concentration_search.concentration.value.toggle();
                true
            }
            Message::ToggleConcentrationEnabled => {
                self.concentration_search.concentration.enabled.toggle();
                true
            }
            Message::SearchText(text) => {
                self.text_search.text = text.to_lowercase();
                true
            }
            Message::ToggleComponent(vsm) => {
                self.component_search.vsm[vsm].value.toggle();
                true
            }
            Message::ToggleComponentEnabled(vsm) => {
                self.component_search.vsm[vsm].enabled.toggle();
                true
            }
            Message::ToggleAdvanced => {
                self.show_advanced_search.toggle();
                false
            }
            // {Search,Character}Page specific options
            Message::CollapseAll
            | Message::Collapse(_) => false,
        }
    }

    pub fn view<'s, 'c: 's>(
        &'s self,
        before_search_bar: impl Into<Option<Button<'c>>>,
        character: Option<usize>,
    ) -> Container<'c> {
        let search = text_input(
            "search for a spell",
            self.search.as_str(),
        )
            .on_input(move |s| wrap_character(character, Message::Search(s)))
            .width(Length::FillPortion(4))
            .id(self.id.clone());
        let reset_modes = button(
            text("Reset").size(14),
        ).tap_if(
            !self.search.is_empty() ||
                !self.searchers()
                    .into_iter()
                    .all(Searcher::is_empty),
            |b| b.on_press(wrap_character(character, Message::ResetSearch)),
        );

        let toggle_advanced = button(text("Advanced Search").size(16))
            .on_press(wrap_character(character, Message::ToggleAdvanced));

        let advanced_search = if self.show_advanced_search {
            column(
                self.searchers()
                    .into_iter()
                    .map(|s| s.view(character).into())
                    .collect()
            ).spacing(1)
        } else {
            col!()
        };

        container(
            col![
                row![
                    Length::Fill,
                    toggle_advanced,
                    search,
                    reset_modes,
                ].align_items(Alignment::Center)
                 .spacing(8)
                 .tap_if_some(before_search_bar.into(), Row::push)
                 .push_space(Length::Fill),
                row![
                    Length::Fill,
                    advanced_search.width(Length::FillPortion(18)),
                    Length::Fill,
                ]
            ]
        )
    }
}

#[derive(Default)]
pub struct SearchPage {
    collapse_all: bool,
    pub search: SearchOptions,
    pub spells: Vec<SearchSpell>,
}

impl SearchPage {
    pub fn new(custom: &[CustomSpell], characters: &[CharacterPage]) -> Self {
        let search = SearchOptions::default();
        let spells = search.search(custom, characters);
        Self {
            collapse_all: false,
            search,
            spells,
        }
    }
}

pub struct SearchSpell {
    pub spell: Spell,
    collapse: Option<bool>,
    buttons: Vec<(Arc<str>, bool)>,
}

impl SearchSpell {
    fn from(spell: Spell, characters: &[CharacterPage]) -> Self {
        let buttons = characters.iter()
            .map(|page| {
                let active = !page.character.spells.iter()
                    .flatten()
                    .any(|(s, _)| *s == spell);
                (Arc::clone(&page.character.name), active)
            })
            .collect();
        Self {
            spell,
            collapse: None,
            buttons,
        }
    }
}

impl SearchPage {
    pub fn update(&mut self, message: Message, custom: &[CustomSpell], characters: &[CharacterPage]) -> Command<crate::Message> {
        let searched_text = matches!(message, Message::SearchText(_));

        match &message {
            Message::CollapseAll => {
                self.collapse_all.toggle();
                self.spells.iter_mut().for_each(|spell| spell.collapse = None);
            }
            Message::Collapse(id) => {
                if let Some(spell) = self.spells.iter_mut()
                    .find(|spell| spell.spell.id() == *id) {
                    if let Some(collapse) = &mut spell.collapse {
                        collapse.toggle();
                    } else {
                        spell.collapse = Some(!self.collapse_all);
                    }
                }
            }
            _ => {}
        };
        let search = self.search.update(message);

        if search {
            self.spells = self.search.search(custom, characters);
        }

        if searched_text {
            Command::none()
        } else {
            text_input::focus(self.search.id.clone())
        }
    }

    pub fn view<'s, 'c: 's>(&'s self) -> Container<'c> {
        let collapse_button = button(
            text_icon(if self.collapse_all { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                .size(15),
        ).on_press(crate::Message::Search(Message::CollapseAll));

        // scroll bar of spells
        let collapse_all = self.collapse_all;
        let spells_col = self.spells.iter()
            .fold(col!().align_items(Alignment::Center), |col, spell| {
                let collapse = match spell.collapse {
                    Some(collapse) => collapse,
                    None => collapse_all,
                };
                col.push(spell.spell.view(SearchPageButtons(&spell.buttons), (), collapse))
                    .push_space(40)
            });
        let scroll: Scrollable<'_> = scrollable::<'_, _, iced::Renderer<Theme>>(spells_col);

        col![
            10,
            self.search.view(collapse_button, None),
            scroll
        ].spacing(6)
            .align_items(Alignment::Center)
            .tap(container)
    }
}

struct SearchPageButtons<'a>(&'a [(Arc<str>, bool)]);

impl SpellButtons for SearchPageButtons<'_> {
    type Data = ();

    fn view<'c>(self, id: SpellId, (): Self::Data) -> (Row<'c>, Element<'c>) {
        let mut buttons = row!();
        if !self.0.is_empty() {
            buttons = buttons.push("Add to:")
                .push_space(15);
        }
        let buttons = self.0.iter()
            .enumerate()
            .fold(buttons, |row, (character, (name, active))|
                row.push({
                    let mut button = button(text(name.as_ref()).size(12));
                    if *active {
                        button = button.on_press(crate::Message::Character(character, character::Message::AddSpell(id.clone())));
                    }
                    button
                }).push_space(5),
            );
        let name = button(
            text(&*id.name).size(36),
        ).width(Length::FillPortion(18))
            .on_press(crate::Message::Search(Message::Collapse(id)))
            .style(Location::Transparent)
            .into();
        (buttons, name)
    }
}