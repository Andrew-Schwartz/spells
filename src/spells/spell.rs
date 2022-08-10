use std::sync::Arc;

use iced::{Alignment, Length, widget};
use iced::widget::{column, container, Container, horizontal_rule, row, text};
use serde::{Deserialize, Serialize};

use crate::{DeserializeSpell, ListGrammaticallyExt, Message, SpacingExt, SpellButtons, SPELLS, Tap};
use crate::spells::data::{CastingTime, Class, Components, Level, School, Source};
use crate::spells::static_arc::StArc;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(try_from = "DeserializeSpell")]
pub struct StaticSpell {
    pub name: &'static str,
    #[serde(skip_serializing)]
    pub name_lower: &'static str,
    pub level: Level,
    pub casting_time: CastingTime,
    pub range: &'static str,
    pub duration: &'static str,
    pub components: Components,
    pub school: School,
    pub ritual: bool,
    pub conc: bool,
    pub description: &'static str,
    #[serde(skip_serializing)]
    pub desc_lower: &'static str,
    pub higher_levels: Option<&'static str>,
    #[serde(skip_serializing)]
    pub higher_levels_lower: Option<&'static str>,
    pub classes: &'static [Class],
    pub source: Source,
    pub page: u32,
}

impl TryFrom<DeserializeSpell> for StaticSpell {
    type Error = String;

    fn try_from(value: DeserializeSpell) -> Result<Self, Self::Error> {
        // we leak stuff since it will be around for the entire time the gui is open
        fn static_str(string: String) -> &'static str {
            Box::leak(string.into_boxed_str())
        }
        let name_lower = static_str(value.name.to_lowercase());
        let desc_lower = static_str(value.description.to_lowercase());
        let higher_levels_lower = value.higher_levels
            .as_ref()
            .map(|s| s.to_lowercase())
            .map(static_str);
        Ok(Self {
            name: value.name,
            name_lower,
            level: value.level,
            casting_time: CastingTime::from_static(value.casting_time)?,
            range: value.range,
            duration: value.duration,
            components: value.components,
            school: value.school,
            ritual: value.ritual,
            conc: value.conc,
            description: static_str(value.description),
            desc_lower,
            higher_levels: value.higher_levels.map(static_str),
            higher_levels_lower,
            classes: value.classes.leak(),
            source: value.source,
            page: value.page,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomSpell {
    pub name: Arc<str>,
    pub name_lower: String,
    pub level: Level,
    pub casting_time: CastingTime,
    pub range: Option<String>,
    pub components: Option<Components>,
    pub duration: Option<String>,
    pub school: School,
    #[serde(default)]
    pub ritual: bool,
    #[serde(default)]
    pub conc: bool,
    pub description: String,
    pub desc_lower: String,
    pub higher_levels: Option<String>,
    pub higher_levels_lower: Option<String>,
    pub classes: Vec<Class>,
    pub page: Option<u32>,
}

impl PartialEq for CustomSpell {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl CustomSpell {
    #[must_use]
    pub fn new(name: String) -> Self {
        let name_lower = name.to_lowercase();
        Self {
            name: Arc::from(name),
            name_lower,
            // name_state: Default::default(),
            level: Level::Cantrip,
            casting_time: CastingTime::Action,
            range: None,
            duration: None,
            components: None,
            school: School::Abjuration,
            ritual: false,
            conc: false,
            description: String::new(),
            desc_lower: String::new(),
            higher_levels: None,
            higher_levels_lower: None,
            classes: Vec::new(),
            page: None,
        }
    }

    #[must_use]
    pub fn id(&self) -> SpellId {
        SpellId {
            name: self.name.clone().into(),
            level: self.level,
        }
    }
}

impl StaticSpell {
    #[must_use]
    pub fn id(&self) -> SpellId {
        SpellId {
            name: self.name.into(),
            level: self.level,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Spell {
    Static(&'static StaticSpell),
    Custom(CustomSpell),
}

macro_rules! delegate {
    ($self:ident, ref ref $delegate:tt $($paren:tt)?) => {
        match $self {
            Self::Static(spell) => &spell.$delegate$($paren)?,
            Self::Custom(spell) => &spell.$delegate$($paren)?,
        }
    };
    ($self:ident, ref $delegate:tt $($paren:tt)?) => {
        match $self {
            Self::Static(spell) => spell.$delegate$($paren)?,
            Self::Custom(spell) => &spell.$delegate$($paren)?,
        }
    };
    ($self:ident, $delegate:tt $($paren:tt)?) => {
        match $self {
            Self::Static(spell) => spell.$delegate$($paren)?,
            Self::Custom(spell) => spell.$delegate$($paren)?,
        }
    };
}

impl Spell {
    #[must_use]
    pub fn id(&self) -> SpellId {
        delegate!(self, id())
    }

    #[must_use]
    pub fn level(&self) -> Level {
        delegate!(self, level)
    }

    #[must_use]
    pub fn classes(&self) -> &[Class] {
        delegate!(self, ref classes)
    }

    #[must_use]
    pub fn school(&self) -> School {
        delegate!(self, school)
    }

    #[must_use]
    pub fn ritual(&self) -> bool {
        delegate!(self, ritual)
    }

    #[must_use]
    pub fn concentration(&self) -> bool {
        delegate!(self, conc)
    }

    #[must_use]
    pub fn name(&self) -> StArc<str> {
        // pedantic clippy wrong
        #[allow(clippy::needless_borrow)]
        match self {
            Self::Static(spell) => spell.name.into(),
            Self::Custom(spell) => (&spell.name).into(),
        }
    }

    #[must_use]
    pub fn name_lower(&self) -> &str {
        delegate!(self, ref name_lower)
    }

    pub fn description(&self) -> &str {
        delegate!(self, ref description)
    }

    #[must_use]
    pub fn desc_lower(&self) -> &str {
        delegate!(self, ref desc_lower)
    }

    pub fn higher_levels(&self) -> Option<&str> {
        match self {
            Self::Static(spell) => spell.higher_levels,
            Self::Custom(spell) => spell.higher_levels.as_deref(),
        }
    }

    #[must_use]
    pub fn higher_levels_lower(&self) -> Option<&str> {
        match self {
            Self::Static(spell) => spell.higher_levels_lower,
            Self::Custom(spell) => spell.higher_levels_lower.as_deref(),
        }
    }

    #[must_use]
    pub fn casting_time(&self) -> &CastingTime {
        // match self {
        //     Self::Static(spell) => &spell.casting_time,
        //     Self::Custom(spell) => &spell.casting_time,
        // }
        delegate!(self, ref ref casting_time)
    }

    pub fn range(&self) -> Option<&str> {
        match self {
            Self::Static(spell) => Some(spell.duration),
            Self::Custom(spell) => spell.duration.as_deref(),
        }
    }

    pub fn components(&self) -> Option<&Components> {
        match self {
            Self::Static(spell) => Some(&spell.components),
            Self::Custom(spell) => spell.components.as_ref(),
        }
    }

    pub fn duration(&self) -> Option<&str> {
        match self {
            Self::Static(spell) => Some(spell.duration),
            Self::Custom(spell) => spell.duration.as_deref(),
        }
    }

    #[must_use]
    pub fn source(&self) -> Source {
        match self {
            Self::Static(spell) => spell.source,
            // todo
            Self::Custom(_) => Source::Custom,
        }
    }

    #[must_use]
    pub fn page(&self) -> Option<u32> {
        match self {
            Self::Static(spell) => Some(spell.page),
            // todo
            Self::Custom(_) => None,
        }
    }

    pub fn view<'s, 'c: 's, B: SpellButtons>(
        &'s self,
        button: B,
        data: B::Data,
        collapse: bool,
        style: Style,
    ) -> Container<'c, Message> {
        let text = |label: String| row(vec![])
            .push(text(label).size(16).width(Length::FillPortion(18)));

        let (buttons, title) = button.view(self.id(), data, style);
        let title = row(vec![]).push(title);

        let buttons = row(vec![]).push(buttons.width(Length::FillPortion(18)));

        let mut column = column(vec![])
            .align_items(Alignment::Center)
            .push(title)
            .push(buttons);
        if !collapse {
            let classes = self.classes().iter().list_grammatically();
            let an_grammar = classes.chars().next()
                .filter(|c| *c == 'A')
                .map_or('\0', |_| 'n');
            let page = match self.page() {
                Some(page) => format!(" page {page}"),
                None => String::new(),
            };
            let about = text(format!("A{an_grammar} {classes} spell, from {}{page}", self.source()));

            column = column
                .push(horizontal_rule(8))
                .push(text(self.school().to_string()))
                .push_space(4)
                .push(text(format!("Level: {}", self.level())))
                .push(text(format!("Casting time: {}", self.casting_time())))
                .tap_if_some(self.range(), |col, range|
                    col.push(text(format!("Range: {}", range))))
                .tap_if_some(self.components(), |col, comp|
                    col.push(text(format!("Components: {}", comp))))
                .tap_if_some(self.duration(), |col, duration|
                    col.push(text(format!("Duration: {}", duration))))
                .push(text(format!("Ritual: {}", if self.ritual() { "Yes" } else { "No" })))
                .push(horizontal_rule(10))
                .push(row(vec![])
                    .push(widget::text(self.description())
                        .size(16)
                        // todo maybe change font to be monospace? have to find a better font
                        // .font(CONSOLAS)
                        .width(Length::FillPortion(18))
                    ))
                .tap_if_some(self.higher_levels(), |col, higher| col
                    .push(horizontal_rule(8))
                    .push(row(vec![]).push(crate::text("At higher levels").size(20).width(Length::FillPortion(18))))
                    .push_space(3)
                    .push(text(higher.to_string())))
                .push(horizontal_rule(8))
                .push(about);
        }

        container(
            row(vec![])
                .push_space(Length::FillPortion(1))
                .push(column.width(Length::FillPortion(18)))
                .push_space(Length::FillPortion(1)))
            .width(Length::Fill)
            .center_x()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SpellId {
    pub name: StArc<str>,
    pub level: Level,
}

#[must_use]
pub fn find_spell(spell_name: &str, custom: &[CustomSpell]) -> Option<Spell> {
    // TODO remove this after its been enough time that everyone probably updated it
    fn fix_name_changes(spell_name: &str, spell: &StaticSpell) -> bool {
        match spell_name {
            // Feb 21, 2022
            "Enemies abound" => spell.name == "Enemies Abound",
            _ => false
        }
    }

    SPELLS.iter()
        .find(|s| s.name == spell_name || fix_name_changes(spell_name, s))
        .map(Spell::Static)
        .or_else(|| custom.iter()
            .find(|s| &*s.name == spell_name)
            .cloned()
            .map(Spell::Custom))
}