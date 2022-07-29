use std::fmt::{self, Debug, Display};
use std::sync::Arc;

use iced::{Alignment, Length, pure::{*, widget::*}};
use iced_aw::Icon;
use itertools::Itertools;

use crate::{CastingTime, character, Class, CustomSpell, School, Source, SpellButtons, SpellId, SPELLS, StaticCustomSpell};
use crate::character::CharacterPage;
use crate::style::Style;
use crate::utils::{IterExt, SpacingExt, Tap, text_icon};

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    CollapseAll,
    Collapse(SpellId),
    Search(String),
    PickMode(Mode),
    ResetModes,
    PickLevel(u8),
    PickCastingTime(CastingTime),
    PickClass(Class),
    PickSchool(School),
    PickSource(Source),
    ToggleRitual(bool),
    ToggleConcentration(bool),
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub enum Mode {
    Level,
    Class,
    School,
    CastingTime,
    Ritual,
    Concentration,
    Text,
    Source,
}

impl Mode {
    pub(crate) const ALL: [Self; 8] = [
        Self::Level,
        Self::Class,
        Self::School,
        Self::CastingTime,
        Self::Ritual,
        Self::Concentration,
        Self::Text,
        Self::Source,
    ];
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // not debug
        f.write_str(match self {
            Mode::Level => "Level",
            Mode::Class => "Class",
            Mode::School => "School",
            Mode::CastingTime => "Casting Time",
            Mode::Ritual => "Ritual",
            Mode::Concentration => "Concentration",
            Mode::Text => "Text",
            Mode::Source => "Source",
        })
    }
}

pub trait Searcher {
    fn is_empty(&self) -> bool;

    fn matches(&self, spell: &StaticCustomSpell) -> bool;

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message>;
}

fn add_buttons<'s, 'c: 's, T: Display + Clone, F: Fn(T) -> Message + 'static>(
    vec: &'s [T],
    on_press: F,
    character: Option<usize>,
    style: Style,
    row: Row<'c, crate::Message>,
) -> Row<'c, crate::Message> {
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
                .style(style.background())
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

#[derive(Debug, Default)]
pub struct LevelSearch {
    pub levels: Vec<u8>,
}

impl Searcher for LevelSearch {
    fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.levels.iter().any(|&t| t == spell.level() as u8)
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let levels = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()
            .filter(|&lvl| self.levels.iter().none(|&l| l == lvl))
            .collect_vec();

        // todo placeholder
        let pick_list = pick_list(
            levels,
            None,
            on_selected(character, Message::PickLevel),
        ).style(style)
            .text_size(14)
            .placeholder("Level");
        add_buttons(&self.levels, Message::PickLevel, character, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct ClassSearch {
    pub classes: Vec<Class>,
}

impl Searcher for ClassSearch {
    fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.classes().iter()
            .any(|class| self.classes.iter().any(|t| class == t))
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let classes = Class::ALL.into_iter()
            .filter(|&class| self.classes.iter().none(|&c| c == class))
            .collect_vec();

        let pick_list = pick_list(
            classes,
            None,
            on_selected(character, Message::PickClass),
        ).style(style)
            .placeholder("Class")
            .text_size(14);
        add_buttons(&self.classes, Message::PickClass, character, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct CastingTimeSearch {
    pub times: Vec<CastingTime>,
}

impl Searcher for CastingTimeSearch {
    fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.times.iter().any(|t|
            t.equals_ignore_reaction(spell.casting_time())
        )
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let durations = [
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
        ].into_iter()
            .filter(|ct| self.times.iter().none(|t| t == ct))
            .collect_vec();

        let pick_list = pick_list(
            durations,
            None,
            on_selected(character, Message::PickCastingTime),
        ).style(style)
            .placeholder("Casting Time")
            .text_size(14);
        add_buttons(&self.times, Message::PickCastingTime, character, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct SchoolSearch {
    pub schools: Vec<School>,
}

impl Searcher for SchoolSearch {
    fn is_empty(&self) -> bool {
        self.schools.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.schools.iter().any(|t| *t == spell.school())
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let schools = School::ALL.into_iter()
            .filter(|&school| self.schools.iter().none(|&s| s == school))
            .collect_vec();

        let pick_list = pick_list(
            schools,
            None,
            on_selected(character, Message::PickSchool),
        ).style(style)
            .placeholder("School")
            .text_size(14);
        add_buttons(&self.schools, Message::PickSchool, character, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct RitualSearch {
    pub ritual: bool,
}

impl Searcher for RitualSearch {
    fn is_empty(&self) -> bool {
        false
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.ritual() == self.ritual
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let checkbox = checkbox(
            "Ritual",
            self.ritual,
            on_selected(character, Message::ToggleRitual),
        ).style(style);
        row.push(checkbox).push_space(5)
    }
}

#[derive(Debug, Default)]
pub struct ConcentrationSearch {
    pub concentration: bool,
}

impl Searcher for ConcentrationSearch {
    fn is_empty(&self) -> bool {
        false
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.concentration() == self.concentration
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let checkbox = checkbox(
            "Concentration",
            self.concentration,
            on_selected(character, Message::ToggleConcentration),
        ).style(style);
        row.push(checkbox).push_space(5)
    }
}

#[derive(Debug, Default)]
pub struct TextSearch {
    pub text: String,
}

impl Searcher for TextSearch {
    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.text.split('|')
            .any(|search|
                spell.desc_lower().contains(search) ||
                    spell.higher_levels_lower()
                        .as_ref()
                        .filter(|lower| lower.contains(search))
                        .is_some()
            )
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let text = "Spell Text:";
        let input = text_input(
            "int|wis",
            &self.text,
            on_selected(character, Message::SearchText),
        ).style(style);
        row.push(text).push_space(4).push(input)
    }
}

#[derive(Debug, Default)]
pub struct SourceSearch {
    pub sources: Vec<Source>,
}

impl Searcher for SourceSearch {
    fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.sources.iter().any(|&t| t == spell.source())
    }

    fn add_to_row<'s, 'c: 's>(
        &'s self,
        row: Row<'c, crate::Message>,
        character: Option<usize>,
        style: Style,
    ) -> Row<'c, crate::Message> {
        let sources = Source::ALL.into_iter()
            .filter(|&source| self.sources.iter().none(|&s| s == source))
            .collect_vec();

        let pick_list = pick_list(
            sources,
            None,
            on_selected(character, Message::PickSource),
        ).style(style)
            .placeholder("Source Book")
            .text_size(14);
        add_buttons(&self.sources, Message::PickSource, character, style, row.push(pick_list))
    }
}

#[derive(Default)]
pub struct SearchOptions {
    pub search: String,
    // todo make them always appear?
    pub level_search: Option<LevelSearch>,
    pub class_search: Option<ClassSearch>,
    pub casting_time_search: Option<CastingTimeSearch>,
    pub school_search: Option<SchoolSearch>,
    pub ritual_search: Option<RitualSearch>,
    pub concentration_search: Option<ConcentrationSearch>,
    pub text_search: Option<TextSearch>,
    pub source_search: Option<SourceSearch>,
}

impl SearchOptions {
    pub fn toggle_mode<T: Default>(search: &mut Option<T>) {
        *search = match search {
            Some(_) => None,
            None => Some(T::default()),
        };
    }

    pub fn search(&mut self, custom: &[CustomSpell], characters: &[CharacterPage]) -> Vec<Spell> {
        let needle = &self.search;
        SPELLS.iter()
            .map(StaticCustomSpell::Static)
            .chain(custom.iter()
                // todo not clone them
                .cloned()
                .map(StaticCustomSpell::Custom))
            .filter(|spell| [
                self.level_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.class_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.school_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.casting_time_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.ritual_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.concentration_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.text_search.as_ref().map::<&dyn Searcher, _>(|s| s),
                self.source_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            ].into_iter()
                .flatten()
                .filter(|searcher| !searcher.is_empty())
                .all(|searcher| searcher.matches(spell)))
            .filter(|spell| spell.name_lower().contains(needle))
            .sorted_unstable_by_key(StaticCustomSpell::name)
            // .sorted_unstable_by_key(|spell| levenshtein(spell.name_lower(), needle))
            .map(|spell| Spell::from(spell, characters))
            .take(100)
            .collect()
    }

    pub fn view<'s, 'c: 's, S, M>(
        &'s self,
        before_search_bar: impl Into<Option<Button<'c, crate::Message>>>,
        search_message: S,
        mode_message: M,
        reset_message: crate::Message,
        character: Option<usize>,
        style: Style,
    ) -> Container<'c, crate::Message>
        where
            S: Fn(String) -> crate::Message + 'static,
            M: Fn(Mode) -> crate::Message + 'static,
    {
        let search = text_input(
            "search for a spell",
            self.search.as_str(),
            search_message,
        ).style(style)
            .width(Length::FillPortion(4));
        let mode = pick_list(
            Mode::ALL.as_ref(),
            None,
            mode_message,
        ).style(style)
            .placeholder("Advanced Search")
            .width(Length::Units(114))
            .text_size(15);
        let reset_modes = button(
            text("Reset").size(14),
        ).style(style)
            .on_press(reset_message);

        // todo this doesn't work on character pages
        // additional search stuff
        let advanced_search = [
            self.level_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.class_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.school_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.casting_time_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.ritual_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.concentration_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.text_search.as_ref().map::<&dyn Searcher, _>(|x| x),
            self.source_search.as_ref().map::<&dyn Searcher, _>(|x| x),
        ].into_iter()
            .flatten()
            .fold(
                row().align_items(Alignment::Center),
                |row, searcher| searcher.add_to_row(row, character, style),
            );

        container(
            column()
                .push(row()
                    .align_items(Alignment::Center)
                    .push_space(Length::Fill)
                    .push(reset_modes)
                    .push_space(4)
                    .push(mode)
                    .push_space(8)
                    .push(search)
                    .tap_if_some(before_search_bar.into(), |row, btn| row
                        .push_space(8)
                        .push(btn))
                    .push_space(Length::Fill)
                )
                .push(row()
                    .push_space(Length::Fill)
                    .push(advanced_search.width(Length::FillPortion(18)))
                    // .push(advanced_search.width(Length::FillPortion(18)))
                    .push_space(Length::Fill)
                )
        )
    }
}

#[derive(Default)]
pub struct SearchPage {
    collapse: bool,
    pub search: SearchOptions,
    pub spells: Vec<Spell>,
}

pub struct Spell {
    pub spell: StaticCustomSpell,
    collapse: Option<bool>,
    buttons: Vec<(Arc<str>, bool)>,
}

impl Spell {
    fn from(spell: StaticCustomSpell, characters: &[CharacterPage]) -> Self {
        let buttons = characters.iter()
            .map(|page| {
                let active = !page.character.spells.iter()
                    .flatten()
                    .any(|(s, _)| s.spell == spell);
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
    pub fn update(&mut self, message: Message, custom: &[CustomSpell], characters: &[CharacterPage]) {
        fn toggle<T: Ord>(vec: &mut Vec<T>, entry: T) {
            if let Some(idx) = vec.iter().position(|t| *t == entry) {
                vec.remove(idx);
            } else {
                vec.push(entry);
                vec.sort();
            }
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
            Message::PickMode(mode) => {
                // most of the time, don't re-search here, because then no spells will match
                match mode {
                    Mode::Level => SearchOptions::toggle_mode(&mut self.search.level_search),
                    Mode::Class => SearchOptions::toggle_mode(&mut self.search.class_search),
                    Mode::School => SearchOptions::toggle_mode(&mut self.search.school_search),
                    Mode::CastingTime => SearchOptions::toggle_mode(&mut self.search.casting_time_search),
                    Mode::Ritual => SearchOptions::toggle_mode(&mut self.search.ritual_search),
                    Mode::Concentration => SearchOptions::toggle_mode(&mut self.search.concentration_search),
                    Mode::Text => SearchOptions::toggle_mode(&mut self.search.text_search),
                    Mode::Source => SearchOptions::toggle_mode(&mut self.search.source_search),
                }
                // the default (false) will still match spells, so redo the search
                mode == Mode::Ritual
            }
            Message::ResetModes => {
                self.search.level_search = None;
                self.search.class_search = None;
                self.search.casting_time_search = None;
                self.search.school_search = None;
                self.search.ritual_search = None;
                self.search.concentration_search = None;
                self.search.text_search = None;
                self.search.source_search = None;
                true
            }
            Message::PickLevel(level) => self.search.level_search.as_mut()
                .map(|levels| toggle(&mut levels.levels, level))
                .is_some(),
            Message::PickClass(class) => self.search.class_search.as_mut()
                .map(|classes| toggle(&mut classes.classes, class))
                .is_some(),
            Message::PickSchool(school) => self.search.school_search.as_mut()
                .map(|schools| toggle(&mut schools.schools, school))
                .is_some(),
            Message::PickCastingTime(casting_time) => self.search.casting_time_search.as_mut()
                .map(|casting_times| toggle(&mut casting_times.times, casting_time))
                .is_some(),
            Message::PickSource(source) => self.search.source_search.as_mut()
                .map(|sources| toggle(&mut sources.sources, source))
                .is_some(),
            Message::ToggleRitual(ritual) => self.search.ritual_search.as_mut()
                .map(|search| search.ritual = ritual)
                .is_some(),
            Message::ToggleConcentration(conc) => self.search.concentration_search.as_mut()
                .map(|search| search.concentration = conc)
                .is_some(),
            Message::SearchText(text) => self.search.text_search.as_mut()
                .map(|search| search.text = text.to_lowercase())
                .is_some(),
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
        };
        if search {
            self.spells = self.search.search(custom, characters);
        }
    }

    pub fn view<'s, 'c: 's>(&'s self, style: Style) -> Container<'c, crate::Message> {
        // todo focus
        // if !matches!(&self.search.text_search, Some(ts) if ts.state.is_focused()) {
        //     self.search.state.focus();
        // }

        let collapse_button = button(
            text_icon(if self.collapse { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                .size(15),
        ).style(style)
            .on_press(crate::Message::Search(Message::CollapseAll));

        // scroll bar of spells
        let collapse_all = self.collapse;
        let scroll = self.spells.iter()
            .fold(column().align_items(Alignment::Fill), |col, spell| {
                let collapse = match spell.collapse {
                    Some(collapse) => collapse,
                    None => collapse_all,
                };
                col.push(spell.spell.view(SearchPageButtons(&spell.buttons), (), collapse, style))
                    .push_space(40)
            })
            .tap(scrollable);

        let column = column()
            .align_items(Alignment::Fill)
            .spacing(6)
            .push_space(10)
            .push(self.search.view(
                collapse_button,
                |s| crate::Message::Search(Message::Search(s)),
                |m| crate::Message::Search(Message::PickMode(m)),
                crate::Message::Search(Message::ResetModes),
                None,
                style,
            ))
            .push(scroll);

        container(column)
    }
}

struct SearchPageButtons<'a>(&'a [(Arc<str>, bool)]);

impl SpellButtons for SearchPageButtons<'_> {
    type Data = ();

    fn view<'c>(self, id: SpellId, (): Self::Data, style: Style) -> (Row<'c, crate::Message>, Element<'c, crate::Message>) {
        let mut buttons = row();
        if !self.0.is_empty() {
            buttons = buttons.push("Add to:")
                .push_space(15);
        }
        let buttons = self.0.iter()
            .enumerate()
            .fold(buttons, |row, (character, (name, active))|
                row.push({
                    let mut button = button(text(name.as_ref()).size(12))
                        .style(style);
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
            .style(style.background())
            .into();
        (buttons, name)
    }
}