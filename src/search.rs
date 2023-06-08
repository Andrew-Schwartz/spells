use std::convert::identity;
use std::fmt::{Debug, Display};
use std::iter;
use std::sync::Arc;

use iced::{Alignment, Length};
use iced::widget::{button, container, scrollable, text, text_input};
use iced_aw::Icon;
use iced_native::Command;
use iced_native::widget::column;
use itertools::Itertools;

use crate::{character, Container, Element, ICON_FONT, Location, Row, Scrollable, SpellButtons, SpellId, SPELLS, Theme};
use crate::character::CharacterPage;
use crate::spells::data::{CastingTime, Class, Level, School, Source};
use crate::spells::spell::{CustomSpell, Spell};
use crate::style::types::Button;
use crate::utils::{IterExt, SpacingExt, Tap, text_icon, Toggle};

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    CollapseAll,
    Collapse(SpellId),
    Search(String),
    // PickMode(Mode),
    ToggleAdvanced,
    ResetModes,
    PickLevel(u8),
    PickCastingTime(CastingTime),
    PickClass(Class),
    PickSchool(School),
    PickSource(Source),
    ToggleRitual,
    ToggleRitualEnabled,
    ToggleConcentration,
    ToggleConcentrationEnabled,
    SearchText(String),
}

pub trait PLNone {
    fn title() -> &'static str;
}

pub trait Unwrap<T>: Sized {
    fn unwrap(self) -> T;
}

impl<T> Unwrap<T> for Option<T> {
    fn unwrap(self) -> T {
        self.unwrap()
    }
}

// #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
// pub enum Mode {
//     Level,
//     Class,
//     School,
//     CastingTime,
//     Ritual,
//     Concentration,
//     Text,
//     Source,
// }
// 
// impl Mode {
//     pub(crate) const ALL: [Self; 8] = [
//         Self::Level,
//         Self::Class,
//         Self::School,
//         Self::CastingTime,
//         Self::Ritual,
//         Self::Concentration,
//         Self::Text,
//         Self::Source,
//     ];
// }
// 
// impl Display for Mode {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // not debug
//         f.write_str(match self {
//             Mode::Level => "Level",
//             Mode::Class => "Class",
//             Mode::School => "School",
//             Mode::CastingTime => "Casting Time",
//             Mode::Ritual => "Ritual",
//             Mode::Concentration => "Concentration",
//             Mode::Text => "Text",
//             Mode::Source => "Source",
//         })
//     }
// }

pub trait Searcher: Debug {
    fn clear(&mut self);

    fn is_empty(&self) -> bool;

    fn matches(&self, spell: &Spell) -> bool;

    fn name(&self) -> &'static str;

    // fn message(&self) -> Message;

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c>;

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    //     // style: Style,
    // ) -> Row<'c>;
}

fn add_buttons<'s, 'c: 's, T: Display + Clone, F: Fn(T) -> Message + 'static>(
    vec: &'s [T],
    on_press: F,
    character: Option<usize>,
    row: Row<'c>,
) -> Row<'c> {
    let len = vec.len();
    vec.iter()
        .enumerate()
        .map(|(i, t)| {
            button(
                text(format!("{}{}", *t, if i + 1 == len { "" } else { ", " })).size(13)
            ).on_press({
                let message = on_press(t.clone());
                match character {
                    Some(i) => crate::Message::Character(i, character::Message::Search(message)),
                    None => crate::Message::Search(message),
                }
            })
                // todo turn off highlight
                .style(Location::Transparent)
                .padding(0)
        })
        .fold(row.push_space(3), Row::push)
        .push_space(5)
}

fn on_selected<F, R>(character: Option<usize>, f: F) -> impl Fn(R) -> crate::Message + 'static
    where
        F: 'static + Fn(R) -> Message,
{
    move |r: R| {
        let search_message = f(r);
        match character {
            Some(i) => crate::Message::Character(i, character::Message::Search(search_message)),
            None => crate::Message::Search(search_message),
        }
    }
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
        // self.levels.iter().any(|&t| t == spell.level() as u8)
    }

    fn name(&self) -> &'static str {
        "Level"
    }

    // fn message(&self) -> Message {
    //     Message::PickMode()
    // }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        iter::zip(self.levels, Level::ALL)
            .fold(
                row!["Levels:"].align_items(Alignment::Center).spacing(4),
                |row, (enabled, l)| row.push(
                    button(text(l).size(14))
                        .padding(0)
                        .style(Location::AdvancedSearch { enabled })
                        .on_press(wrap_character(character, Message::PickLevel(l.as_u8())))
                ),
            )
    }

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     // let levels = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()
    //     //     .filter(|&lvl| self.levels.iter().none(|&l| l == lvl))
    //     //     .collect_vec();
    //     //
    //     // let pick_list = pick_list(
    //     //     levels,
    //     //     None,
    //     //     on_selected(character, Message::PickLevel),
    //     // )
    //     //     .text_size(14)
    //     //     .placeholder("Level");
    //     // add_buttons(&self.levels, Message::PickLevel, character, row.push(pick_list))
    //     todo!()
    // }
}

#[derive(Debug, Default)]
pub struct ClassSearch {
    pub classes: Vec<Class>,
}

impl Searcher for ClassSearch {
    fn clear(&mut self) {
        self.classes.clear()
    }

    fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        spell.classes().iter()
            .any(|class| self.classes.iter().any(|t| class == t))
    }

    fn name(&self) -> &'static str {
        "Class"
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

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let classes = Class::ALL.into_iter()
    //         .filter(|&class| self.classes.iter().none(|&c| c == class))
    //         .collect_vec();
    //
    //     let pick_list = pick_list(
    //         classes,
    //         None,
    //         on_selected(character, Message::PickClass),
    //     )
    //         .placeholder("Class")
    //         .text_size(14);
    //     add_buttons(&self.classes, Message::PickClass, character, row.push(pick_list))
    // }
}

#[derive(Debug, Default)]
pub struct CastingTimeSearch {
    pub times: Vec<CastingTime>,
}

impl Searcher for CastingTimeSearch {
    fn clear(&mut self) {
        self.times.clear()
    }

    fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.times.iter().any(|t|
            t.equals_ignore_reaction(spell.casting_time())
        )
    }

    fn name(&self) -> &'static str {
        "Casting Time"
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

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let durations = [
    //         CastingTime::Action,
    //         CastingTime::BonusAction,
    //         CastingTime::Reaction(None),
    //         CastingTime::Minute(1),
    //         CastingTime::Minute(10),
    //         CastingTime::Hour(1),
    //         CastingTime::Hour(8),
    //         CastingTime::Hour(12),
    //         CastingTime::Hour(24),
    //         CastingTime::Special,
    //     ].into_iter()
    //         .filter(|ct| self.times.iter().none(|t| t == ct))
    //         .collect_vec();
    //
    //     let pick_list = pick_list(
    //         durations,
    //         None,
    //         on_selected(character, Message::PickCastingTime),
    //     )
    //         .placeholder("Casting Time")
    //         .text_size(14);
    //     add_buttons(&self.times, Message::PickCastingTime, character, row.push(pick_list))
    // }
}

#[derive(Debug, Default)]
pub struct SchoolSearch {
    pub schools: Vec<School>,
}

impl Searcher for SchoolSearch {
    fn clear(&mut self) {
        self.schools.clear()
    }

    fn is_empty(&self) -> bool {
        self.schools.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.schools.iter().any(|t| *t == spell.school())
    }

    fn name(&self) -> &'static str {
        "School"
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

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let schools = School::ALL.into_iter()
    //         .filter(|&school| self.schools.iter().none(|&s| s == school))
    //         .collect_vec();
    //
    //     let pick_list = pick_list(
    //         schools,
    //         None,
    //         on_selected(character, Message::PickSchool),
    //     )
    //         .placeholder("School")
    //         .text_size(14);
    //     add_buttons(&self.schools, Message::PickSchool, character, row.push(pick_list))
    // }
}

#[derive(Debug, Default)]
pub struct RitualSearch {
    pub ritual: bool,
    pub enabled: bool,
}

impl Searcher for RitualSearch {
    fn clear(&mut self) {
        self.ritual = false;
        self.enabled = false;
    }

    fn is_empty(&self) -> bool {
        !self.enabled
    }

    fn matches(&self, spell: &Spell) -> bool {
        !self.enabled || spell.ritual() == self.ritual
    }

    fn name(&self) -> &'static str {
        "Ritual"
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        row![
            button(
                text("Ritual:")
            ).padding(0)
                .style(Location::AdvancedSearch { enabled: self.enabled })
                .on_press(wrap_character(character, Message::ToggleRitualEnabled)),
            button(
                text(if self.ritual { Icon::Check } else { Icon::X })
                    .font(ICON_FONT)
                    .size(15)
                    // .vertical_alignment(alignment::Vertical::Center)
            ).padding(0)
                .style(Location::AdvancedSearch { enabled: self.enabled })
                .tap_if(self.enabled, |b|
                    b.on_press(wrap_character(character, Message::ToggleRitual))
                )
        ].align_items(Alignment::Center)
            .spacing(4)
    }

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let checkbox = checkbox(
    //         "Ritual",
    //         self.ritual,
    //         on_selected(character, Message::ToggleRitual),
    //     );
    //     row.push(checkbox).push_space(5)
    // }
}

#[derive(Debug, Default)]
pub struct ConcentrationSearch {
    pub concentration: bool,
    pub enabled: bool,
}

impl Searcher for ConcentrationSearch {
    fn clear(&mut self) {
        self.concentration = false;
        self.enabled = false
    }

    fn is_empty(&self) -> bool {
        !self.enabled
    }

    fn matches(&self, spell: &Spell) -> bool {
        !self.enabled || spell.concentration() == self.concentration
    }

    fn name(&self) -> &'static str {
        "Concentration"
    }

    fn view<'s, 'c: 's>(&'s self, character: Option<usize>) -> Row<'c> {
        row![
            button(
                text("Concentration:")
            ).padding(0)
                .style(Location::AdvancedSearch { enabled: self.enabled })
                .on_press(wrap_character(character, Message::ToggleConcentrationEnabled)),
            button(
                text(if self.concentration { Icon::Check } else { Icon::X })
                    .font(ICON_FONT)
                    .size(15)
                    // .vertical_alignment(alignment::Vertical::Center)
            ).padding(0)
                .style(Location::AdvancedSearch { enabled: self.enabled })
                .tap_if(self.enabled, |b|
                    b.on_press(wrap_character(character, Message::ToggleConcentration))
                )
        ].align_items(Alignment::End)
            .spacing(4)
    }

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let checkbox = checkbox(
    //         "Concentration",
    //         self.concentration,
    //         on_selected(character, Message::ToggleConcentration),
    //     );
    //     row.push(checkbox).push_space(5)
    // }
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
        self.text.clear()
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

    fn name(&self) -> &'static str {
        "Text"
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

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let text = "Spell Text:";
    //     let input = text_input(
    //         "int|wis",
    //         &self.text,
    //     ).on_input(on_selected(character, Message::SearchText));
    //     row.push(text).push_space(4).push(input)
    // }
}

#[derive(Debug, Default)]
pub struct SourceSearch {
    pub sources: Vec<Source>,
}

impl Searcher for SourceSearch {
    fn clear(&mut self) {
        self.sources.clear()
    }

    fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    fn matches(&self, spell: &Spell) -> bool {
        self.sources.iter().any(|&t| t == spell.source())
    }

    fn name(&self) -> &'static str {
        "Source"
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

    // fn add_to_row<'s, 'c: 's>(
    //     &'s self,
    //     row: Row<'c>,
    //     character: Option<usize>,
    // ) -> Row<'c> {
    //     let sources = Source::ALL.into_iter()
    //         .filter(|&source| self.sources.iter().none(|&s| s == source))
    //         .collect_vec();
    //
    //     let pick_list = pick_list(
    //         sources,
    //         None,
    //         on_selected(character, Message::PickSource),
    //     )
    //         .placeholder("Source Book")
    //         .text_size(14);
    //     add_buttons(&self.sources, Message::PickSource, character, row.push(pick_list))
    // }
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
    pub text_search: TextSearch,
    pub source_search: SourceSearch,
    // todo VSM search
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
        }
    }
}

impl SearchOptions {
    pub fn searchers(&self) -> [&dyn Searcher; 8] {
        [
            &self.level_search as &dyn Searcher,
            &self.class_search as &dyn Searcher,
            &self.school_search as &dyn Searcher,
            &self.casting_time_search as &dyn Searcher,
            &self.ritual_search as &dyn Searcher,
            &self.concentration_search as &dyn Searcher,
            &self.source_search as &dyn Searcher,
            &self.text_search as &dyn Searcher,
        ]
    }

    pub fn search(&self, custom: &[CustomSpell], characters: &[CharacterPage]) -> Vec<SearchSpell> {
        let needle = &self.search;
        SPELLS.iter()
            .map(Spell::Static)
            .chain(custom.iter()
                // todo not clone them
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

    pub fn view<'s, 'c: 's, S>(
        &'s self,
        before_search_bar: impl Into<Option<Button<'c>>>,
        search_message: S,
        reset_message: crate::Message,
        character: Option<usize>,
    ) -> Container<'c>
        where S: Fn(String) -> crate::Message + 'static,
    {
        let search = text_input(
            "search for a spell",
            self.search.as_str(),
        )
            .on_input(search_message)
            .width(Length::FillPortion(4))
            .id(self.id.clone());
        // todo did I do this?
        // text_input::focus(self.search_id.clone());
        let reset_modes = button(
            text("Reset").size(14),
        )
            // todo make this only enable if there's anything to reset
            .on_press(reset_message);

        // let toggle_advanced_modes = self.searchers()
        //     .into_iter()
        //     .map(|s|
        //              button(text(s.name()).size(15))
        //                  .padding(2.0)
        //                  .on_press(wrap_character(character, s.message())),
        //     ).collect_vec();

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
                 .tap_if_some(
                    before_search_bar.into(),
                    |row, btn| row.push(btn)
                 )
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
    collapse: bool,
    pub search: SearchOptions,
    pub spells: Vec<SearchSpell>,
}

impl SearchPage {
    pub fn new(custom: &[CustomSpell], characters: &[CharacterPage]) -> Self {
        let search = SearchOptions::default();
        let spells = search.search(custom, characters);
        Self {
            collapse: false,
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
        fn toggle<T: Ord>(vec: &mut Vec<T>, entry: T) -> bool {
            if let Some(idx) = vec.iter().position(|t| *t == entry) {
                vec.remove(idx);
            } else {
                vec.push(entry);
                vec.sort();
            }
            // !vec.is_empty()
            true
        }

        let search = match message {
            Message::Search(needle) => {
                self.search.search = needle.to_lowercase();
                true
            }
            Message::Refresh => {
                self.spells = self.search.search(custom, characters);
                false
            }
            // Message::PickMode(mode) => {
            //     // most of the time, don't re-search here, because then no spells will match
            //     // todo
            //     // match mode {
            //     //     Mode::Level => SearchOptions::toggle_mode(&mut self.search.level_search),
            //     //     Mode::Class => SearchOptions::toggle_mode(&mut self.search.class_search),
            //     //     Mode::School => SearchOptions::toggle_mode(&mut self.search.school_search),
            //     //     Mode::CastingTime => SearchOptions::toggle_mode(&mut self.search.casting_time_search),
            //     //     Mode::Ritual => SearchOptions::toggle_mode(&mut self.search.ritual_search),
            //     //     Mode::Concentration => SearchOptions::toggle_mode(&mut self.search.concentration_search),
            //     //     Mode::Text => SearchOptions::toggle_mode(&mut self.search.text_search),
            //     //     Mode::Source => SearchOptions::toggle_mode(&mut self.search.source_search),
            //     // }
            //     // the default (false) will still match spells, so redo the search
            //     // mode == Mode::Ritual
            // }
            Message::ResetModes => {
                self.search.level_search.clear();
                self.search.class_search.clear();
                self.search.casting_time_search.clear();
                self.search.school_search.clear();
                self.search.ritual_search.clear();
                self.search.concentration_search.clear();
                self.search.text_search.clear();
                self.search.source_search.clear();
                true
            }
            Message::PickLevel(level) => {
                self.search.level_search.levels[level as usize].toggle();
                true
            }
            Message::PickClass(class) => toggle(&mut self.search.class_search.classes, class),
            Message::PickSchool(school) => toggle(&mut self.search.school_search.schools, school),
            Message::PickCastingTime(casting_time) => toggle(&mut self.search.casting_time_search.times, casting_time),
            Message::PickSource(source) => toggle(&mut self.search.source_search.sources, source),
            Message::ToggleRitual => {
                self.search.ritual_search.ritual.toggle();
                true
            }
            Message::ToggleRitualEnabled => {
                todo!()
            }
            Message::ToggleConcentration => {
                self.search.concentration_search.concentration.toggle();
                true
            }
            Message::ToggleConcentrationEnabled => {
                self.search.concentration_search.enabled.toggle();
                true
            }
            Message::SearchText(text) => {
                self.search.text_search.text = text.to_lowercase();
                !self.search.text_search.text.is_empty()
            }
            Message::CollapseAll => {
                self.collapse = !self.collapse;
                self.spells.iter_mut().for_each(|spell| spell.collapse = None);
                false
            }
            Message::Collapse(id) => {
                if let Some(spell) = self.spells.iter_mut()
                    .find(|spell| spell.spell.id() == id) {
                    if let Some(collapse) = &mut spell.collapse {
                        *collapse = !*collapse;
                    } else {
                        spell.collapse = Some(!self.collapse);
                    }
                }
                false
            }
            Message::ToggleAdvanced => {
                self.search.show_advanced_search = !self.search.show_advanced_search;
                false
            }
        };
        if search {
            self.spells = self.search.search(custom, characters);
        }

        // todo focus
        // if !matches!(&self.search.text_search, Some(ts) if ts.id.is_focused()) {
        //     text_input::focus(self.search.search_id.clone())
        // } else {
        text_input::focus(self.search.id.clone())
        // Command::none()
        // }
    }

    pub fn view<'s, 'c: 's>(&'s self) -> Container<'c> {
        let collapse_button = button(
            text_icon(if self.collapse { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                .size(15),
        ).on_press(crate::Message::Search(Message::CollapseAll));

        // scroll bar of spells
        let collapse_all = self.collapse;
        let spells_col = self.spells.iter()
            // todo is center right? it was Full before
            .fold(col!().align_items(Alignment::Center), |col, spell| {
                let collapse = match spell.collapse {
                    Some(collapse) => collapse,
                    None => collapse_all,
                };
                col.push(spell.spell.view(SearchPageButtons(&spell.buttons), (), collapse))
                    .push_space(40)
            });
        let scroll: Scrollable<'_> = scrollable::<'_, _, iced::Renderer<Theme>>(spells_col);

        let column = col!()
            .align_items(Alignment::Center)
            .spacing(6)
            .push_space(10)
            .push(self.search.view(
                collapse_button,
                |s| crate::Message::Search(Message::Search(s)),
                // |m| crate::Message::Search(m),
                // |m| crate::Message::Search(Message::PickMode(m)),
                crate::Message::Search(Message::ResetModes),
                None,
            ))
            .push(scroll);

        container(column)
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
            // todo no highlight
            .style(Location::Transparent)
            .into();
        (buttons, name)
    }
}