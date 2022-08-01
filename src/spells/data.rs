use std::fmt;
use std::fmt::Display;
use std::intrinsics::transmute;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};

use crate::StArc;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Class {
    Artificer,
    Bard,
    Cleric,
    Druid,
    Paladin,
    Ranger,
    Sorcerer,
    Warlock,
    Wizard,
}

impl Class {
    pub const ALL: [Self; 9] = [
        Self::Artificer,
        Self::Bard,
        Self::Cleric,
        Self::Druid,
        Self::Paladin,
        Self::Ranger,
        Self::Sorcerer,
        Self::Warlock,
        Self::Wizard,
    ];
}

impl Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Artificer => "Artificer",
            Self::Bard => "Bard",
            Self::Cleric => "Cleric",
            Self::Druid => "Druid",
            Self::Paladin => "Paladin",
            Self::Ranger => "Ranger",
            Self::Sorcerer => "Sorcerer",
            Self::Warlock => "Warlock",
            Self::Wizard => "Wizard",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub enum School {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Transmutation,
    Necromancy,
}

impl School {
    pub const ALL: [Self; 8] = [
        Self::Abjuration,
        Self::Conjuration,
        Self::Divination,
        Self::Enchantment,
        Self::Evocation,
        Self::Illusion,
        Self::Transmutation,
        Self::Necromancy,
    ];
}

impl Display for School {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Abjuration => "Abjuration",
            Self::Conjuration => "Conjuration",
            Self::Enchantment => "Enchantment",
            Self::Evocation => "Evocation",
            Self::Illusion => "Illusion",
            Self::Transmutation => "Transmutation",
            Self::Necromancy => "Necromancy",
            Self::Divination => "Divination",
        })
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Ord, PartialOrd)]
pub enum CastingTime {
    Special,
    Action,
    BonusAction,
    Reaction(Option<StArc<str>>),
    Minute(usize),
    Hour(usize),
}

impl CastingTime {
    pub const ALL: [Self; 6] = [
        Self::Action,
        Self::BonusAction,
        Self::Reaction(None),
        Self::Minute(1),
        Self::Hour(1),
        Self::Special,
    ];

    const REACTION_PHRASE: &'static str = ", which you take when ";

    pub fn from_static(str: &'static str) -> Result<Self, String> {
        let space_idx = str.find(' ');
        let get_num = || {
            let space_idx = space_idx.ok_or_else(|| format!("No number specified in casting time \"{}\"", str))?;
            let num = &str[..space_idx];
            num.parse()
                .map_err(|_| format!("{} is not a positive integer", num))
        };
        let comma = str.find(',').unwrap_or(str.len());
        let rest = &str[space_idx.map_or(0, |i| i + 1)..comma];
        match rest {
            "Special" => Ok(Self::Special),
            "Action" => Ok(Self::Action),
            "Bonus Action" => Ok(Self::BonusAction),
            "Reaction" => {
                if str[comma..].starts_with(Self::REACTION_PHRASE) {
                    Ok(Self::Reaction(Some(str[comma + Self::REACTION_PHRASE.len()..].into())))
                } else {
                    Err(String::from("No reaction when phrase"))
                }
            }
            "Minute" | "Minutes" => Ok(Self::Minute(get_num()?)),
            "Hour" | "Hours" => Ok(Self::Hour(get_num()?)),
            _ => Err(format!("{} is not a casting time", rest))
        }
    }

    pub fn equals_ignore_reaction(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Special, Self::Special) => true,
            (Self::Action, Self::Action) => true,
            (Self::BonusAction, Self::BonusAction) => true,
            (Self::Reaction(_), Self::Reaction(_)) => true,
            (&Self::Minute(m1), &Self::Minute(m2)) if m1 == m2 => true,
            (&Self::Hour(h1), &Self::Hour(h2)) if h1 == h2 => true,
            _ => false,
        }
    }
}

impl Display for CastingTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Special => f.write_str("Special"),
            Self::Action => f.write_str("1 Action"),
            Self::BonusAction => f.write_str("1 Bonus Action"),
            Self::Reaction(when) => if let Some(when) = when {
                write!(f, "1 Reaction, which you take when {when}")
            } else {
                f.write_str("1 Reaction")
            }
            &Self::Minute(n) => write!(f, "{n} Minute{}", if n == 1 { "" } else { "s" }),
            &Self::Hour(n) => write!(f, "{n} Hour{}", if n == 1 { "" } else { "s" }),
        }
    }
}

impl<'de> Deserialize<'de> for CastingTime {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let str = <&'de str>::deserialize(d)?;
        let space_idx = str.find(' ');
        let get_num = || {
            let space_idx = space_idx.ok_or_else(|| D::Error::custom(format!("No number specified in casting time \"{}\"", str)))?;
            let num = &str[..space_idx];
            num.parse()
                .map_err(|_| D::Error::custom(format!("{} is not a positive integer", num)))
        };
        let comma = str.find(',').unwrap_or(str.len());
        let rest = &str[space_idx.map_or(0, |i| i + 1)..comma];
        match rest {
            "Special" => Ok(Self::Special),
            "Action" => Ok(Self::Action),
            "Bonus Action" => Ok(Self::BonusAction),
            "Reaction" => {
                Ok(Self::Reaction(
                    str[comma..].starts_with(Self::REACTION_PHRASE)
                        .then(|| StArc::Arc(Arc::from(&str[comma + Self::REACTION_PHRASE.len()..])))
                ))
            }
            "Minute" | "Minutes" => Ok(Self::Minute(get_num()?)),
            "Hour" | "Hours" => Ok(Self::Hour(get_num()?)),
            _ => Err(D::Error::custom(format!("{} is not a casting time", rest)))
        }
    }
}

impl Serialize for CastingTime {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Special => "Special".serialize(s),
            Self::Action => "1 Action".serialize(s),
            Self::BonusAction => "1 Bonus Action".serialize(s),
            Self::Reaction(None) => "1 Reaction".serialize(s),
            Self::Reaction(Some(when)) => format!("1 Reaction{}{}", Self::REACTION_PHRASE, when).serialize(s),
            &Self::Minute(n) => if n == 1 {
                "1 Minute".serialize(s)
            } else {
                format!("{} Minutes", n).serialize(s)
            },
            &Self::Hour(n) => if n == 1 {
                "1 Hour".serialize(s)
            } else {
                format!("{} Hours", n).serialize(s)
            },
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Ord, PartialOrd, Default)]
pub struct Components {
    pub v: bool,
    pub s: bool,
    pub m: Option<String>,
}

impl Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prev = false;
        if self.v {
            write!(f, "V")?;
            prev = true;
        }
        if self.s {
            if prev { write!(f, ", ")? }
            write!(f, "S")?;
            prev = true;
        }
        if let Some(material) = &self.m {
            if prev { write!(f, ", ")? }
            write!(f, "M ({material})")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Components {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let str = <&'de str>::deserialize(d)?.trim();

        fn vsm(str: &str) -> (bool, bool, bool) {
            let mut vsm = (false, false, false);
            for char in str.chars() {
                match char {
                    'V' => vsm.0 = true,
                    'S' => vsm.1 = true,
                    'M' => vsm.2 = true,
                    ' ' | ',' => {}
                    _ => println!("Bad character {char} in {str}"),
                }
            }
            vsm
        }

        let ((v, s, _), material) = if let (Some(start), Some(end)) = (str.find('('), str.rfind(')')) {
            let vsm = vsm(&str[..start]);
            assert_eq!(vsm.2, true);
            (vsm, Some(&str[start + 1..end]))
        } else {
            let vsm = vsm(str);
            assert_eq!(vsm.2, false);
            (vsm, None)
        };

        let components = Self {
            v,
            s,
            m: material.map(|str| str.to_string()),
        };
        Ok(components)
    }
}

impl Serialize for Components {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        format!("{self}").serialize(s)
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Ord, PartialOrd)]
pub enum Source {
    PlayersHandbook,
    XanatharsGuideToEverything,
    TashasCauldronOfEverything,
    // todo Custom(String)
    Custom,
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::STRINGS[*self as usize])
    }
}

impl Source {
    pub const ALL: [Self; 4] = [
        Self::PlayersHandbook,
        Self::XanatharsGuideToEverything,
        Self::TashasCauldronOfEverything,
        Self::Custom,
    ];

    const STRINGS: [&'static str; 4] = [
        "Player's Handbook",
        "Xanathar's Guide to Everything",
        "Tasha's Cauldron of Everything",
        "Custom",
    ];
}

impl<'de> Deserialize<'de> for Source {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let str = <&'de str>::deserialize(d)?;
        match str {
            "Player's Handbook" => Ok(Self::PlayersHandbook),
            "Xanathar's Guide to Everything" => Ok(Self::XanatharsGuideToEverything),
            "Tasha's Cauldron of Everything" => Ok(Self::TashasCauldronOfEverything),
            "Custom" => Ok(Self::Custom),
            _ => Err(D::Error::unknown_variant(str, &Self::STRINGS)),
        }
    }
}

impl Serialize for Source {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Self::STRINGS[*self as usize].serialize(s)
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Level {
    Cantrip,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,
    L9,
}

impl Level {
    pub const ALL: [Self; 10] = [
        Self::Cantrip,
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L6,
        Self::L7,
        Self::L8,
        Self::L9,
    ];

    pub const fn from_u8(n: u8) -> Option<Self> {
        match n {
            // SAFETY: D&D spell levels are 0 (cantrip) and 1-9 ONLY
            0..=9 => Some(unsafe { transmute(n) }),
            _ => None,
        }
    }

    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub const fn next_checked(self) -> Option<Self> {
        Self::from_u8(self.as_u8() + 1)
    }

    pub const fn next_saturating(self) -> Self {
        self.next_checked().unwrap_or(self)
    }

    pub const fn prev_checked(self) -> Option<Self> {
        self.as_u8().checked_sub(1)
            .and_then(Self::from_u8)
    }

    pub const fn prev_saturating(self) -> Self {
        self.prev_checked().unwrap_or(self)
    }

    pub const fn add_checked(self, offset: isize) -> Option<Self> {
        match offset {
            1 => self.next_checked(),
            -1 => self.prev_checked(),
            _ => unreachable!(),
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cantrip => f.write_str("Cantrip"),
            &level => (level as u64).fmt(f),
        }
    }
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let num = u8::deserialize(d)?;
        Self::from_u8(num)
            .ok_or_else(|| D::Error::invalid_value(Unexpected::Unsigned(num as _), &"An integer in the range 0..=9"))
    }
}

impl Serialize for Level {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (*self as u64).serialize(s)
    }
}

impl<T> Index<Level> for [T; 10] {
    type Output = T;

    fn index(&self, index: Level) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Level> for [T; 10] {
    fn index_mut(&mut self, index: Level) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> Index<Level> for [T; 9] {
    type Output = T;

    fn index(&self, index: Level) -> &Self::Output {
        &self[index as usize - 1]
    }
}

impl<T> IndexMut<Level> for [T; 9] {
    fn index_mut(&mut self, index: Level) -> &mut Self::Output {
        &mut self[index as usize - 1]
    }
}

pub trait GetLevel<T> {
    fn get_lvl(&self, level: Level) -> Option<&T>;

    fn get_lvl_mut(&mut self, level: Level) -> Option<&mut T>;
}

impl<T> GetLevel<T> for [T; 10] {
    fn get_lvl(&self, level: Level) -> Option<&T> {
        self.get(level as usize)
    }

    fn get_lvl_mut(&mut self, level: Level) -> Option<&mut T> {
        self.get_mut(level as usize)
    }
}

impl<T> GetLevel<T> for [T; 9] {
    fn get_lvl(&self, level: Level) -> Option<&T> {
        self.get((level as usize).checked_sub(1)?)
    }

    fn get_lvl_mut(&mut self, level: Level) -> Option<&mut T> {
        self.get_mut((level as usize).checked_sub(1)?)
    }
}