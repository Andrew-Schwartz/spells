use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::sync::Arc;

use iced::{Align, button, Button, Checkbox, Column, Container, Element, Length, pick_list, PickList, Row, Scrollable, Text, TextInput};
use iced::widget::{scrollable, text_input};
use iced_aw::{Icon, ICON_FONT};
use itertools::Itertools;

use crate::{CastingTime, character, Class, CustomSpell, School, Source, SpellButtons, SpellId, SPELLS, StaticCustomSpell};
use crate::character::CharacterPage;
use crate::style::Style;
use crate::utils::{IterExt, SpacingExt};

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
    SearchText(String),
}

pub trait PLNone {
    fn title() -> &'static str;
}

/// `PickListOption`, meant to be used as the title for a `PickList` but not in the set of items
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PLOption<T> {
    None,
    Some(T),
}

impl<T> PLOption<T> {
    pub fn unwrap(self) -> T {
        match self {
            PLOption::Some(t) => t,
            PLOption::None => panic!("called `PLOption::unwrap()` on a `None` value"),
        }
    }
}

impl<T: PLNone + Display + Eq> From<Option<T>> for PLOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(t) => Self::Some(t),
            None => Self::None,
        }
    }
}

impl<T: PLNone + Display + Eq> Display for PLOption<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PLOption::None => f.write_str(T::title()),
            PLOption::Some(t) => t.fmt(f),
        }
    }
}

macro_rules! plopt {
    ($ty:ty, $none:literal) => {
        impl PLNone for $ty {
            fn title() -> &'static str { $none }
        }
    };
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub enum Mode {
    Level,
    Class,
    School,
    CastingTime,
    Ritual,
    Text,
    Source,
}
plopt!(Mode, "Advanced Search");
plopt!(Class, "Class");
plopt!(School, "School");
plopt!(Source, "Source");

impl Mode {
    pub(crate) const ALL: [PLOption<Self>; 7] = [
        PLOption::Some(Self::Level),
        PLOption::Some(Self::Class),
        PLOption::Some(Self::School),
        PLOption::Some(Self::CastingTime),
        PLOption::Some(Self::Ritual),
        PLOption::Some(Self::Text),
        PLOption::Some(Self::Source),
    ];
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct PickListLevel(u8);

impl PickListLevel {
    const ALL: [Self; 10] = [
        Self(0),
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
        Self(9),
    ];

    const NONE: Self = Self(u8::MAX);

    pub fn unwrap(self) -> u8 {
        if self == Self::NONE {
            panic!("called `PickListLevel::unwrap()` on a `NONE` value")
        } else {
            self.0
        }
    }
}

impl Display for PickListLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            l @ 0..=9 => Display::fmt(&l, f),
            _ => f.write_str("Level"),
        }
    }
}

pub trait Searcher {
    fn is_empty(&self) -> bool;

    fn matches(&self, spell: &StaticCustomSpell) -> bool;

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message>;
}

#[derive(Debug)]
struct WithButton<T> {
    t: T,
    state: button::State,
}

impl<T> WithButton<T> {
    fn new(t: T) -> Self {
        Self { t, state: Default::default() }
    }
}

impl<T: PartialEq> PartialEq for WithButton<T> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl<T: Eq> Eq for WithButton<T> {}

impl<T: PartialOrd> PartialOrd for WithButton<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl<T: Ord> Ord for WithButton<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.t.cmp(&other.t)
    }
}

fn add_buttons<'a, T: Display + Clone, F: Fn(T) -> Message>(
    vec: &'a mut Vec<WithButton<T>>,
    on_press: F,
    style: Style,
    row: Row<'a, crate::Message>,
) -> Row<'a, crate::Message> {
    let len = vec.len();
    vec.iter_mut()
        .enumerate()
        .map(|(i, WithButton { t, state })| Button::new(
            state,
            Text::new(format!("{}{}", *t, if i + 1 == len { "" } else { ", " })).size(13),
        ).on_press(crate::Message::Search(on_press(t.clone())))
            .style(style.background())
            .padding(0)
        )
        .fold(row.push_space(3), Row::push)
        .push_space(5)
}

#[derive(Debug, Default)]
pub struct LevelSearch {
    levels: Vec<WithButton<u8>>,
    state: pick_list::State<PickListLevel>,
}

impl Searcher for LevelSearch {
    fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.levels.iter().any(|WithButton { t, .. }| *t == spell.level() as u8)
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let levels = PickListLevel::ALL.into_iter()
            .filter(|lvl| self.levels.iter().none(|wb| wb.t == lvl.0))
            .collect_vec();

        let pick_list = PickList::new(
            &mut self.state,
            levels,
            Some(PickListLevel::NONE),
            |pll| crate::Message::Search(Message::PickLevel(pll.unwrap())),
        ).style(style).text_size(14);
        add_buttons(&mut self.levels, Message::PickLevel, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct ClassSearch {
    classes: Vec<WithButton<Class>>,
    state: pick_list::State<PLOption<Class>>,
}

impl Searcher for ClassSearch {
    fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.classes().iter()
            .any(|class| self.classes.iter().any(|WithButton { t, .. }| class == t))
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let classes = Class::ALL.into_iter()
            .filter(|class| self.classes.iter().none(|wb| wb.t == *class))
            .map(PLOption::Some)
            .collect_vec();

        let pick_list = PickList::new(
            &mut self.state,
            classes,
            Some(PLOption::None),
            |c| crate::Message::Search(Message::PickClass(c.unwrap())),
        ).style(style).text_size(14);
        add_buttons(&mut self.classes, Message::PickClass, style, row.push(pick_list))
    }
}

plopt!(CastingTime, "Casting Time");

#[derive(Debug, Default)]
pub struct CastingTimeSearch {
    times: Vec<WithButton<CastingTime>>,
    state: pick_list::State<PLOption<CastingTime>>,
}

impl Searcher for CastingTimeSearch {
    fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.times.iter().any(|WithButton { t, .. }| t == spell.casting_time())
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
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
            .filter(|ct| self.times.iter().none(|t| t.t == *ct))
            .map(PLOption::Some)
            .collect_vec();

        let pick_list = PickList::new(
            &mut self.state,
            durations,
            Some(PLOption::None),
            |s| crate::Message::Search(Message::PickCastingTime(s.unwrap())),
        ).style(style).text_size(14);
        add_buttons(&mut self.times, Message::PickCastingTime, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct SchoolSearch {
    schools: Vec<WithButton<School>>,
    state: pick_list::State<PLOption<School>>,
}

impl Searcher for SchoolSearch {
    fn is_empty(&self) -> bool {
        self.schools.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.schools.iter().any(|WithButton { t, .. }| *t == spell.school())
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let schools = School::ALL.into_iter()
            .filter(|school| self.schools.iter().none(|wb| wb.t == *school))
            .map(PLOption::Some)
            .collect_vec();

        let pick_list = PickList::new(
            &mut self.state,
            schools,
            Some(PLOption::None),
            |s| crate::Message::Search(Message::PickSchool(s.unwrap())),
        ).style(style).text_size(14);
        add_buttons(&mut self.schools, Message::PickSchool, style, row.push(pick_list))
    }
}

#[derive(Debug, Default)]
pub struct RitualSearch {
    ritual: bool,
}

impl Searcher for RitualSearch {
    fn is_empty(&self) -> bool {
        false
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.ritual() == self.ritual
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let checkbox = Checkbox::new(
            self.ritual,
            "Ritual",
            |b| crate::Message::Search(Message::ToggleRitual(b)),
        ).style(style);
        row.push(checkbox).push_space(5)
    }
}

#[derive(Debug, Default)]
pub struct TextSearch {
    text: String,
    state: text_input::State,
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

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let text = Text::new("Spell Text:");
        let input = TextInput::new(
            &mut self.state,
            "int|wis",
            &self.text,
            |s| crate::Message::Search(Message::SearchText(s)),
        ).style(style);
        row.push(text).push_space(4).push(input)
    }
}

#[derive(Debug, Default)]
pub struct SourceSearch {
    sources: Vec<WithButton<Source>>,
    state: pick_list::State<PLOption<Source>>,
}

impl Searcher for SourceSearch {
    fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.sources.iter().any(|wb| wb.t == spell.source())
    }

    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let sources = Source::ALL.into_iter()
            .filter(|source| self.sources.iter().none(|wb| wb.t == *source))
            .map(PLOption::Some)
            .collect_vec();

        let pick_list = PickList::new(
            &mut self.state,
            sources,
            Some(PLOption::None),
            |s| crate::Message::Search(Message::PickSource(s.unwrap())),
        ).style(style).text_size(14);
        add_buttons(&mut self.sources, Message::PickSource, style, row.push(pick_list))
    }
}

#[derive(Default)]
pub struct SearchOptions {
    pub state: text_input::State,
    pub search: String,
    pub mode_state: pick_list::State<PLOption<Mode>>,
    pub reset_modes: button::State,
    // todo make them always appear?
    pub level_search: Option<LevelSearch>,
    pub class_search: Option<ClassSearch>,
    pub casting_time_search: Option<CastingTimeSearch>,
    pub school_search: Option<SchoolSearch>,
    pub ritual_search: Option<RitualSearch>,
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

    fn search(&mut self, custom: &[CustomSpell], characters: &[CharacterPage]) -> Vec<Spell> {
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

    pub fn view<'s, S, M>(
        &'s mut self,
        before_search_bar: impl Into<Option<Button<'s, crate::Message>>>,
        search_message: S,
        mode_message: M,
        reset_message: crate::Message,
        style: Style,
    ) -> Container<'s, crate::Message>
        where
            S: 'static + Fn(String) -> crate::Message,
            M: 'static + Fn(Mode) -> crate::Message,
    {
        let search = TextInput::new(
            &mut self.state,
            "search for a spell",
            self.search.as_str(),
            search_message,
        ).style(style)
            .width(Length::FillPortion(4));
        let mode = PickList::new(
            &mut self.mode_state,
            Mode::ALL.as_ref(),
            Some(PLOption::None),
            move |m| mode_message(m.unwrap()),
        ).style(style)
            .width(Length::Units(114))
            .text_size(15);
        let reset_modes = Button::new(
            &mut self.reset_modes,
            Text::new("Reset").size(14),
        ).style(style)
            .on_press(reset_message);

        // additional search stuff
        let advanced_search = [
            self.level_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.class_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.school_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.casting_time_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.ritual_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.text_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
            self.source_search.as_mut().map::<&mut dyn Searcher, _>(|x| x),
        ].into_iter()
            .flatten()
            .fold(
                Row::new().align_items(Align::Center),
                |row, searcher| searcher.add_to_row(row, style),
            );

        Container::new(
            Column::new().push(
                before_search_bar.into()
                    .map_or_else(
                        || Row::new()
                            .align_items(Align::Center)
                            .push_space(Length::Fill),
                        |btn| Row::new()
                            .align_items(Align::Center)
                            .push_space(Length::Fill)
                            .push(btn)
                            .push_space(8))
                    .push(search)
                    .push_space(3)
                    .push(mode)
                    .push_space(3)
                    .push(reset_modes)
                    .push_space(Length::Fill)
            ).push(Row::new()
                .push_space(Length::Fill)
                .push(advanced_search.width(Length::FillPortion(18)))
                .push_space(Length::Fill)
            )
        )
    }
}

#[derive(Default)]
pub struct SearchPage {
    collapse_state: button::State,
    collapse: bool,
    pub search: SearchOptions,
    pub scroll: scrollable::State,
    pub spells: Vec<Spell>,
}

pub struct Spell {
    pub spell: StaticCustomSpell,
    collapse: Option<bool>,
    name: button::State,
    buttons: Vec<(Arc<str>, button::State, bool)>,
}

impl Spell {
    fn from(spell: StaticCustomSpell, characters: &[CharacterPage]) -> Self {
        let buttons = characters.iter()
            .map(|page| {
                let active = !page.character.spells.iter()
                    .flatten()
                    .any(|(s, _)| s.spell == spell);
                (Arc::clone(&page.character.name), Default::default(), active)
            })
            .collect();
        Self {
            spell,
            collapse: None,
            name: Default::default(),
            buttons,
        }
    }
}

impl SearchPage {
    pub fn update(&mut self, message: Message, custom: &[CustomSpell], characters: &[CharacterPage]) {
        fn toggle<T: Ord>(vec: &mut Vec<WithButton<T>>, entry: T) {
            if let Some(idx) = vec.iter().position(|WithButton { t, .. }| *t == entry) {
                vec.remove(idx);
            } else {
                vec.push(WithButton::new(entry));
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
                    Mode::Text => SearchOptions::toggle_mode(&mut self.search.text_search),
                    Mode::Source => SearchOptions::toggle_mode(&mut self.search.source_search)
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
                self.search.text_search = None;
                self.search.source_search = None;
                true
            }
            Message::PickLevel(level) => if let Some(levels) = &mut self.search.level_search {
                toggle(&mut levels.levels, level);
                true
            } else {
                false
            },
            Message::PickClass(class) => if let Some(classes) = &mut self.search.class_search {
                toggle(&mut classes.classes, class);
                true
            } else {
                false
            },
            Message::PickSchool(school) => if let Some(schools) = &mut self.search.school_search {
                toggle(&mut schools.schools, school);
                true
            } else {
                false
            },
            Message::PickCastingTime(casting_time) => if let Some(casting_times) = &mut self.search.casting_time_search {
                toggle(&mut casting_times.times, casting_time);
                true
            } else {
                false
            },
            Message::PickSource(source) => if let Some(sources) = &mut self.search.source_search {
                toggle(&mut sources.sources, source);
                true
            } else {
                false
            },
            Message::ToggleRitual(ritual) => if let Some(search) = &mut self.search.ritual_search {
                search.ritual = ritual;
                true
            } else {
                false
            },
            Message::SearchText(text) => if let Some(search) = &mut self.search.text_search {
                search.text = text.to_lowercase();
                true
            } else {
                false
            },
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

    pub fn view(&mut self, style: Style) -> Container<crate::Message> {
        if !matches!(&self.search.text_search, Some(ts) if ts.state.is_focused()) {
            self.search.state.focus();
        }

        let collapse_button = Button::new(
            &mut self.collapse_state,
            Text::new(if self.collapse { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                .font(ICON_FONT)
                .size(15),
        ).style(style)
            .on_press(crate::Message::Search(Message::CollapseAll));

        // scroll bar of spells
        let collapse_all = self.collapse;
        let scroll = self.spells.iter_mut()
            .fold(Scrollable::new(&mut self.scroll), |scroll, spell| {
                let collapse = match spell.collapse {
                    Some(collapse) => collapse,
                    None => collapse_all,
                };
                scroll.push(spell.spell.view(SearchPageButtons(&mut spell.name, &mut spell.buttons), (), collapse, style))
                    .push_space(40)
            });

        let column = Column::new()
            .align_items(Align::Center)
            .spacing(6)
            .push_space(10)
            .push(self.search.view(
                collapse_button,
                |s| crate::Message::Search(Message::Search(s)),
                |m| crate::Message::Search(Message::PickMode(m)),
                crate::Message::Search(Message::ResetModes),
                style,
            ))
            .push(scroll);

        Container::new(column)
    }
}

struct SearchPageButtons<'a>(&'a mut button::State, &'a mut [(Arc<str>, button::State, bool)]);

impl<'a> SpellButtons<'a> for SearchPageButtons<'a> {
    type Data = ();

    fn view(self, id: SpellId, (): (), style: Style) -> (Row<'a, crate::Message>, Element<'a, crate::Message>) {
        let mut buttons = Row::new();
        if !self.1.is_empty() {
            buttons = buttons.push(Text::new("Add to:"))
                .push_space(15);
        }
        let buttons = self.1.iter_mut()
            .enumerate()
            .fold(buttons, |row, (character, (name, state, active))|
                row.push({
                    let mut button = Button::new(state, Text::new(name.as_ref()).size(12))
                        .style(style);
                    if *active {
                        button = button.on_press(crate::Message::Character(character, character::Message::AddSpell(id.clone())));
                    }
                    button
                }).push_space(5),
            );
        let name = Button::new(
            self.0,
            Text::new(&*id.name)
                .size(36),
        ).width(Length::FillPortion(18))
            .on_press(crate::Message::Search(Message::Collapse(id)))
            .style(style.background())
            .into();
        (buttons, name)
    }
}