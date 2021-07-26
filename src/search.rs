use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::sync::Arc;

use iced::{Align, button, Button, Checkbox, Column, Container, Element, Length, pick_list, PickList, Row, Scrollable, Text, TextInput};
use iced::widget::{scrollable, text_input};
use iced_aw::{Icon, ICON_FONT};
use itertools::Itertools;
use levenshtein::levenshtein;

use crate::{character, Class, CustomSpell, School, SpellButtons, SpellId, SPELLS, StaticCustomSpell};
use crate::character::CharacterPage;
use crate::style::Style;
use crate::utils::SpacingExt;

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    CollapseAll,
    Collapse(SpellId),
    Search(String),
    PickMode(Mode),
    PickLevel(u8),
    PickClass(Class),
    PickSchool(School),
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
    Ritual,
    Text,
}
plopt!(Mode, "Advanced Search");
plopt!(Class, "Class");
plopt!(School, "School");

impl Mode {
    const ALL: [PLOption<Self>; 5] = [
        PLOption::Some(Self::Level),
        PLOption::Some(Self::Class),
        PLOption::Some(Self::School),
        PLOption::Some(Self::Ritual),
        PLOption::Some(Self::Text),
    ];
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Mode::Level => "Level",
            Mode::Class => "Class",
            Mode::School => "School",
            Mode::Ritual => "Ritual",
            Mode::Text => "Text",
        })
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
            l @ 0..=9 => l.fmt(f),
            _ => f.write_str("Level"),
        }
    }
}

trait Searcher {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message>;

    fn matches(&self, spell: &StaticCustomSpell) -> bool;
}

#[derive(Debug, Default)]
struct LevelSearch {
    /// [bool; 10]
    bitmask: u16,
    state: pick_list::State<PickListLevel>,
}

impl Searcher for LevelSearch {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let text = Text::new(self.to_string()).size(14);
        let pick_list = PickList::new(
            &mut self.state,
            &PickListLevel::ALL[..],
            Some(PickListLevel::NONE),
            |pll| crate::Message::Search(Message::PickLevel(pll.unwrap())),
        ).style(style).text_size(14);
        row.push(pick_list).push(text)
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        let bit = 1 << spell.level();
        self.bitmask & bit == bit
    }
}

impl Display for LevelSearch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for pll in &PickListLevel::ALL {
            let bit = 1 << pll.0;
            if self.bitmask & bit == bit {
                write!(f, "{}", pll.0)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct ClassSearch {
    classes: BTreeSet<Class>,
    state: pick_list::State<PLOption<Class>>,
}

impl Searcher for ClassSearch {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let pick_list = PickList::new(
            &mut self.state,
            &Class::ALL[..],
            Some(PLOption::None),
            |c| crate::Message::Search(Message::PickClass(c.unwrap())),
        ).style(style).text_size(14);
        let text = Text::new(self.classes.iter().join(", ")).size(14);
        row.push(pick_list).push(text)
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.classes().iter()
            .any(|s| self.classes.contains(s))
    }
}

#[derive(Debug, Default)]
struct SchoolSearch {
    schools: BTreeSet<School>,
    state: pick_list::State<PLOption<School>>,
}

impl Searcher for SchoolSearch {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let pick_list = PickList::new(
            &mut self.state,
            &School::PL_ALL[..],
            Some(PLOption::None),
            |s| crate::Message::Search(Message::PickSchool(s.unwrap())),
        ).style(style).text_size(14);
        let text = Text::new(self.schools.iter().join(", ")).size(14);
        row.push(pick_list).push(text)
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        self.schools.contains(&spell.school())
    }
}

#[derive(Debug, Default)]
struct RitualSearch {
    ritual: bool,
}

impl Searcher for RitualSearch {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let checkbox = Checkbox::new(
            self.ritual,
            "Ritual",
            |b| crate::Message::Search(Message::ToggleRitual(b)),
        ).style(style);
        row.push(checkbox)
    }

    fn matches(&self, spell: &StaticCustomSpell) -> bool {
        spell.ritual() == self.ritual
    }
}

#[derive(Debug, Default)]
struct TextSearch {
    text: String,
    state: text_input::State,
}

impl Searcher for TextSearch {
    fn add_to_row<'a>(&'a mut self, row: Row<'a, crate::Message>, style: Style) -> Row<'a, crate::Message> {
        let text = Text::new("Spell Text:");
        let input = TextInput::new(
            &mut self.state,
            "int|wis",
            &self.text,
            |s| crate::Message::Search(Message::SearchText(s)),
        ).style(style);
        row.push(text).push(input)
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
}

// todo
// #[derive(Debug, Default)]
// struct CastingTimeSearch {
//     text: String,
//     state: text_input::State,
// }
//
// impl Searcher for CastingTimeSearch {
//     fn add_to_row<'a>(&'a mut self, row: Row<'a, Message>, style: Style) -> Row<'a, Message> {
//         let text = Text::new("Casting Time:");
//         let input = TextInput::new(
//             &mut self.state,
//             "",
//             &self.text,
//             |s| crate::Message::Search(Message::SearchCastingTime(s))
//         )
//     }
//
//     fn matches(&self, spell: &Spell) -> bool {
//         todo!()
//     }
// }

pub struct SearchPage {
    collapse_state: button::State,
    collapse: bool,
    pub state: text_input::State,
    search: String,
    mode_state: pick_list::State<PLOption<Mode>>,
    level_search: Option<LevelSearch>,
    class_search: Option<ClassSearch>,
    school_search: Option<SchoolSearch>,
    ritual_search: Option<RitualSearch>,
    text_search: Option<TextSearch>,
    pub scroll: scrollable::State,
    pub spells: Vec<Spell>,
}

impl Default for SearchPage {
    fn default() -> Self {
        Self {
            collapse_state: Default::default(),
            collapse: false,
            state: text_input::State::focused(),
            search: Default::default(),
            mode_state: Default::default(),
            level_search: None,
            class_search: None,
            school_search: None,
            ritual_search: None,
            text_search: None,
            scroll: Default::default(),
            spells: Default::default(),
        }
    }
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
        fn toggle<T: Ord>(map: &mut BTreeSet<T>, entry: T) {
            if map.contains(&entry) {
                map.remove(&entry);
            } else {
                map.insert(entry);
            }
        }

        match message {
            Message::Search(needle) => {
                self.search = needle.to_lowercase();
                self.search(custom, characters);
            }
            Message::Refresh => self.search(custom, characters),
            Message::PickMode(mode) => {
                fn toggle_search<T: Default>(search: &mut Option<T>) {
                    *search = match search {
                        Some(_) => None,
                        None => Some(T::default())
                    }
                }
                // most of the time, don't re-search here, because then no spells will match
                match mode {
                    Mode::Level => toggle_search(&mut self.level_search),
                    Mode::Class => toggle_search(&mut self.class_search),
                    Mode::School => toggle_search(&mut self.school_search),
                    Mode::Ritual => {
                        toggle_search(&mut self.ritual_search);
                        // the default (false) will still match spells, so redo the search
                        self.search(custom, characters);
                    }
                    Mode::Text => toggle_search(&mut self.text_search),
                }
            }
            Message::PickLevel(level) => {
                if let Some(levels) = &mut self.level_search {
                    let bit = 1 << level;
                    if levels.bitmask & (bit) == bit {
                        levels.bitmask -= bit;
                    } else {
                        levels.bitmask |= bit;
                    }
                    self.search(custom, characters);
                }
            }
            Message::PickClass(class) => {
                if let Some(classes) = &mut self.class_search {
                    toggle(&mut classes.classes, class);
                    self.search(custom, characters);
                }
            }
            Message::PickSchool(school) => {
                if let Some(schools) = &mut self.school_search {
                    toggle(&mut schools.schools, school);
                    self.search(custom, characters);
                }
            }
            Message::ToggleRitual(ritual) => {
                if let Some(search) = &mut self.ritual_search {
                    search.ritual = ritual;
                    self.search(custom, characters);
                }
            }
            Message::SearchText(text) => {
                if let Some(search) = &mut self.text_search {
                    search.text = text.to_lowercase();
                    self.search(custom, characters);
                }
            }
            Message::CollapseAll => {
                self.collapse = !self.collapse;
                self.spells.iter_mut().for_each(|spell| spell.collapse = None);
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
            }
        }
    }

    fn search(&mut self, custom: &[CustomSpell], characters: &[CharacterPage]) {
        let needle = &self.search;
        let searches: [Option<&dyn Searcher>; 5] = [
            self.level_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            self.class_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            self.school_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            self.ritual_search.as_ref().map::<&dyn Searcher, _>(|s| s),
            self.text_search.as_ref().map::<&dyn Searcher, _>(|s| s),
        ];
        self.spells = SPELLS.iter()
            .map(StaticCustomSpell::Static)
            .chain(custom.iter()
                // todo not clone them
                .cloned()
                .map(StaticCustomSpell::Custom))
            .filter(|spell| std::array::IntoIter::new(searches)
                .flatten()
                .all(|searcher| searcher.matches(spell)))
            // .map(|spell| (spell.name_lower().clone(), spell))
            .filter(|spell| spell.name_lower().contains(needle))
            .sorted_unstable_by_key(|spell| levenshtein(spell.name_lower(), needle))
            .map(|spell| Spell::from(spell, characters))
            .take(100)
            .collect();
    }

    pub fn view(&mut self, style: Style) -> Container<crate::Message> {
        if !matches!(&self.text_search, Some(ts) if ts.state.is_focused()) {
            self.state.focus();
        }

        let collapse_button = Button::new(
            &mut self.collapse_state,
            Text::new(if self.collapse { Icon::ArrowsExpand } else { Icon::ArrowsCollapse })
                .font(ICON_FONT)
                .size(15),
        ).style(style)
            .on_press(crate::Message::Search(Message::CollapseAll));
        let search = TextInput::new(
            &mut self.state,
            "search for a spell",
            self.search.as_str(),
            |s| crate::Message::Search(Message::Search(s)),
        ).style(style)
            .width(Length::FillPortion(4));
        let mode = PickList::new(
            &mut self.mode_state,
            Mode::ALL.as_ref(),
            Some(PLOption::None),
            |m| crate::Message::Search(Message::PickMode(m.unwrap())),
        ).style(style)
            .width(Length::Units(114))
            .text_size(15);

        // additional search stuff
        let searchers: [Option<&mut dyn Searcher>; 5] = [
            self.level_search.as_mut().map::<&mut dyn Searcher, _>(|s| s),
            self.class_search.as_mut().map::<&mut dyn Searcher, _>(|s| s),
            self.school_search.as_mut().map::<&mut dyn Searcher, _>(|s| s),
            self.ritual_search.as_mut().map::<&mut dyn Searcher, _>(|s| s),
            self.text_search.as_mut().map::<&mut dyn Searcher, _>(|s| s),
        ];
        let advanced_search = std::array::IntoIter::new(searchers)
            .flatten()
            .fold(Row::new().spacing(8), |row, searcher| searcher.add_to_row(row, style));

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
            },
            );

        let column = Column::new()
            .align_items(Align::Center)
            .spacing(6)
            .push_space(10)
            .push(Row::new()
                .push_space(Length::Fill)
                .push(collapse_button)
                .push_space(4)
                .push(search)
                .push_space(3)
                .push(mode)
                .push_space(Length::Fill)
                .align_items(Align::Center))
            .push(Row::new()
                .push_space(Length::Fill)
                .push(advanced_search.width(Length::FillPortion(18)))
                .push_space(Length::Fill))
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