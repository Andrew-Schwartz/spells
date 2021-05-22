// ignored on other targets
#![windows_subsystem = "windows"]
#![warn(clippy::pedantic)]
//! @formatter:off
#![allow(
    clippy::module_name_repetitions,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::filter_map,
    clippy::default_trait_access,
    clippy::cast_sign_loss,
    clippy::option_if_let_else,
)]
//! @formatter:on

use std::cmp::min;
use std::collections::HashMap;
use std::convert::{Infallible, TryFrom};
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use iced::{Application, Button, button, Column, Command, Container, Element, Font, Length, Row, Rule, Settings, Slider, slider, Space, Text, VerticalAlignment};
use iced::window::Icon;
use iced_native::{Event, Subscription, window};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use search::SearchPage;
use tabs::Tabs;

use crate::character::{CharacterPage, SerializeCharacter};
use crate::hotkey::Move;
use crate::hotmouse::{ButtonPress, Pt};
use crate::new::NewPage;
use crate::search::PLOption;
use crate::style::Style;
use crate::tabs::Tab;

mod fetch;
mod style;
mod search;
mod tabs;
mod new;
mod character;
mod hotkey;
mod hotmouse;

const JSON: &str = include_str!("../resources/spells.json");
pub static SPELLS: Lazy<Vec<Spell>> = Lazy::new(|| serde_json::from_str(&JSON).expect("json error in `data/spells.json`"));
static SAVE_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = dirs::data_local_dir().unwrap_or_default();
    path.push("dndspells/");
    std::fs::create_dir_all(&path).unwrap();
    path.push("characters.json");
    std::fs::OpenOptions::new().create(true).append(true).open(&path).unwrap();
    path
});
const FONT: Font = Font::External {
    name: "Arial",
    bytes: include_bytes!("../resources/arial.ttf"),
};

// default window size is 1024, want two columns for that
const COLUMN_WIDTH: f32 = (1024 / 2) as _;

fn main() -> iced::Result {
    const LOGO: &[u8] = include_bytes!("../resources/logo.png");
    const WIDTH: u32 = 1500;
    const HEIGHT: u32 = 1500;
    let image = image::load_from_memory(LOGO).expect("failed to read logo");

    let icon = Icon::from_rgba(image.into_bytes(), WIDTH, HEIGHT).unwrap();

    Window::run(Settings {
        window: iced::window::Settings {
            min_size: Some((1024 / 2, 600)),
            icon: Some(icon),
            ..Default::default()
        },
        default_font: Some(include_bytes!("../resources/arial.ttf")),
        default_text_size: 18,
        antialiasing: true,
        ..Default::default()
    })
}

struct Window {
    style: Style,
    tabs: Tabs,
    width: u32,
    col_scale: f32,
    col_reset: button::State,
    col_slider: slider::State,
    style_button: button::State,
    search_page: SearchPage,
    new_page: NewPage,
    characters: Vec<Arc<str>>,
    character_pages: HashMap<Arc<str>, CharacterPage>,
    save_states: Vec<Vec<(Arc<str>, SerializeCharacter<'static>)>>,
    state: Option<usize>,
    num_cols: usize,
    mouse: hotmouse::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTheme,
    SetColScale(f32),
    SwitchTab(Tab),
    Search(search::Message),
    New(new::Message),
    Character(Arc<str>, character::Message),
    MoveCharacter(Arc<str>, isize),
    DeleteCharacter(Arc<str>),
    Hotkey(hotkey::Message),
    MouseState(hotmouse::StateMessage),
    ScrollIGuessHopefully(Pt),
    Resize(u32),
}

impl Window {
    fn set_num_columns(&mut self) {
        self.num_cols = (self.width as f32 / (COLUMN_WIDTH * self.col_scale)).ceil() as _;
    }

    fn add_character(&mut self, name: Arc<str>) {
        let clone = || Arc::clone(&name);
        self.character_pages.insert(clone(), CharacterPage::new(clone()));
        self.characters.push(clone());
        self.refresh_search();
        self.tabs.characters.push((clone(), Default::default()));
        self.tabs.state = Tab::Character(name);
        self.save().expect("failed to save");
    }

    fn swap_characters(&mut self, a: usize, b: usize) {
        self.characters.swap(a, b);
        self.refresh_search();
        self.tabs.characters.swap(a, b);
        self.save().expect("blah");
    }

    fn remove_character(&mut self, name: &Arc<str>) {
        if let Some(idx) = self.characters.iter().position(|c| c == name) {
            self.characters.remove(idx);
            self.tabs.characters.remove(idx);
        }
        self.character_pages.remove(name);
        self.refresh_search();
        self.save().expect("waa haa");
    }

    fn save_state(&mut self) {
        if let Some(idx) = self.state.take() {
            self.save_states.truncate(idx + 1);
        }
        let state = self.characters.iter()
            .map(|name| (
                Arc::clone(&name),
                self.character_pages.get(name)
                    .unwrap()
                    .serialize())
            )
            .collect();
        self.save_states.push(state);
    }

    fn load_state(&mut self, idx: usize) {
        let state = self.save_states.get(idx).unwrap();
        self.characters = state.iter()
            .map(|(c, _)| c)
            .map(Arc::clone)
            .collect();
        self.character_pages = state.iter()
            .map(|(c, page)| (Arc::clone(c), CharacterPage::from_serialized(page)))
            .collect();
        self.tabs.characters = state.iter()
            .map(|(c, _)| (Arc::clone(c), Default::default()))
            .collect();
    }

    fn open() -> anyhow::Result<Self> {
        let (characters, character_pages) = match File::open(&*SAVE_FILE) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut vec = Vec::new();
                let mut map = HashMap::new();
                for line in reader.lines() {
                    let line = line.unwrap();
                    let c = serde_json::from_str(&line)?;
                    let c = CharacterPage::from_serialized(&c);
                    vec.push(Arc::clone(&c.name));
                    map.insert(Arc::clone(&c.name), c);
                }
                (vec, map)
            }
            Err(e) if matches!(e.kind(), ErrorKind::NotFound) => {
                // std::fs::create_dir("data")?;
                File::create(&*SAVE_FILE)?;
                (Vec::default(), HashMap::default())
            }
            Err(e) => return Err(e.into()),
        };
        let mut window = Self {
            style: Style::default(),
            tabs: Tabs::new(characters.iter().map(Arc::clone)),
            width: iced::window::Settings::default().size.0,
            col_scale: 1.0,
            col_reset: Default::default(),
            col_slider: Default::default(),
            style_button: Default::default(),
            search_page: Default::default(),
            new_page: Default::default(),
            characters,
            character_pages,
            save_states: Default::default(),
            state: None,
            num_cols: 2,
            mouse: Default::default(),
        };
        window.save_state();
        Ok(window)
    }

    fn save(&mut self) -> anyhow::Result<()> {
        self.save_state();
        let mut file = File::create(&*SAVE_FILE)?;
        for c in &self.characters {
            if let Some(c) = self.character_pages.get(c) {
                serde_json::to_writer(&mut file, &c.serialize())?;
                file.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    fn refresh_search(&mut self) {
        self.search_page.update(search::Message::Refresh, &self.characters, &self.character_pages);
    }
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        let window = Self::open().expect("failed to start");
        (window, async { Message::Search(search::Message::Refresh) }.into())
    }

    fn title(&self) -> String {
        "D&D Spells".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::ToggleTheme => self.style = match self.style {
                Style::Light => Style::Dark,
                Style::Dark => Style::Light,
            },
            Message::SetColScale(mult) => {
                self.col_scale = mult;
                self.set_num_columns();
            },
            Message::SwitchTab(tab) => {
                self.tabs.update(tab, &mut self.search_page, &mut self.new_page);
            }
            Message::Search(msg) => self.search_page.update(msg, &self.characters, &self.character_pages),
            Message::New(msg) => {
                if let Some(name) = self.new_page.update(msg, &self.characters) {
                    self.add_character(name);
                }
            }
            Message::Character(name, msg) => {
                let add = matches!(msg, character::Message::AddSpell(_));
                let num_cols = self.num_cols;
                let must_save = self.character_pages.get_mut(&name)
                    .map(|c| c.update(msg, num_cols));
                if add {
                    self.search_page.state.focus();
                    // have to update after adding the spell
                    self.refresh_search();
                }
                if let Some(true) = must_save {
                    self.refresh_search();
                    self.save().expect("todo #2");
                }
            }
            Message::MoveCharacter(name, delta) => {
                let idx = self.characters.iter().position(|c| *c == name);
                if let Some(idx) = idx {
                    let new_idx = if delta.is_negative() {
                        idx.saturating_sub(delta.abs() as usize)
                    } else {
                        min(idx + delta as usize, self.characters.len() - 1)
                    };
                    self.swap_characters(idx, new_idx);
                }
            }
            Message::DeleteCharacter(name) => {
                self.remove_character(&name);
            }
            Message::Hotkey(message) => {
                use hotkey::Message;
                match message {
                    Message::ToCharacter(idx) => {
                        let idx = if idx == 0 {
                            // go to last tab
                            self.characters.len() - 1
                        } else {
                            idx - 1
                        };
                        if let Some(name) = self.characters.get(idx) {
                            self.tabs.state = Tab::Character(Arc::clone(&name))
                        }
                    }
                    Message::Find(main_page) => {
                        match (main_page, &self.tabs.state) {
                            (true, _) | (false, Tab::New) | (false, Tab::Search) => {
                                self.tabs.state = Tab::Search;
                                self.refresh_search();
                            }
                            (false, Tab::Character(name)) => {
                                if let Some(page) = self.character_pages.get_mut(name) {
                                    page.tab = 0;
                                    page.search_state.focus();
                                }
                            }
                        }
                    }
                    Message::NewCharacter => self.tabs.state = Tab::New,
                    Message::Move(dir, tab_only) => {
                        if tab_only {
                            let new_tab_idx = self.character_pages.len() + 1;
                            let orig_idx = match &self.tabs.state {
                                Tab::Search => 0,
                                Tab::Character(name) => self.characters.iter()
                                    .position(|c| c == name)
                                    .unwrap() + 1,
                                Tab::New => new_tab_idx,
                            };
                            let idx = match dir {
                                Move::Left => min(orig_idx.wrapping_sub(1), new_tab_idx),
                                Move::Right => {
                                    let idx = orig_idx + 1;
                                    if idx > new_tab_idx { 0 } else { idx }
                                }
                            };
                            match idx {
                                0 => self.tabs.state = Tab::Search,
                                new_tab if new_tab == new_tab_idx => self.tabs.state = Tab::New,
                                idx => {
                                    let character = &self.characters[idx - 1];
                                    self.tabs.state = Tab::Character(Arc::clone(character));
                                }
                            }
                        } else {
                            let new_tab_idx = self.character_pages.len() * character::TABS + 1;
                            let (orig_idx, orig_character) = match &self.tabs.state {
                                Tab::Search => (0, None),
                                Tab::Character(name) => {
                                    let character_idx = self.characters.iter()
                                        .position(|c| c == name)
                                        .unwrap();
                                    let character_tab = self.character_pages.get(name).unwrap().tab;
                                    (1 + // first tab of first char is index 1
                                         character::TABS * character_idx + // 11 tabs in each character
                                         character_tab, Some(name))
                                }
                                Tab::New => (new_tab_idx, None),
                            };
                            let idx = match dir {
                                Move::Left => min(orig_idx.wrapping_sub(1), new_tab_idx),
                                Move::Right => {
                                    let idx = orig_idx + 1;
                                    if idx > new_tab_idx { 0 } else { idx }
                                }
                            };
                            match idx {
                                0 => self.tabs.state = Tab::Search,
                                new_tab if new_tab == new_tab_idx => self.tabs.state = Tab::New,
                                idx => {
                                    let character = (idx - 1) / character::TABS;
                                    let character = &self.characters[character];
                                    let tab = (idx - 1) % character::TABS;
                                    if orig_character != Some(character) {
                                        self.tabs.state = Tab::Character(Arc::clone(character));
                                    }
                                    self.character_pages.get_mut(character).unwrap().tab = tab;
                                }
                            }
                        }
                    }
                    Message::Undo => {
                        let orig_idx = self.state;
                        let idx = if let Some(idx) = &mut self.state {
                            *idx = idx.saturating_sub(1);
                            *idx
                        } else {
                            let idx = self.save_states.len().saturating_sub(2);
                            self.state = Some(idx);
                            idx
                        };
                        if orig_idx != Some(idx) {
                            self.load_state(idx)
                        }
                    }
                    Message::Redo => {
                        if let Some(idx) = &mut self.state {
                            let orig_idx = *idx;
                            *idx += 1;
                            *idx = min(*idx, self.save_states.len().saturating_sub(1));
                            let idx = *idx;
                            if orig_idx != idx {
                                // only update if it changed
                                self.load_state(idx);
                            }
                        }
                    }
                    Message::CharacterTab(tab) => {
                        if let Tab::Character(name) = &self.tabs.state {
                            if let Some(page) = self.character_pages.get_mut(name) {
                                page.tab = tab;
                            }
                        }
                    }
                    Message::AddSpell(idx) => {
                        if let Some(spell) = self.search_page.spells.first().map(|s| s.spell.id()) {
                            let character = self.characters.get(idx)
                                .cloned()
                                .and_then(|c| self.character_pages.get_mut(&c));
                            if let Some(character) = character {
                                character.add_spell(spell);
                                self.refresh_search();
                            }
                        }
                    }
                }
            }
            Message::Resize(width) => {
                self.width = width;
                self.set_num_columns();
            }
            Message::MouseState(msg) => {
                // println!("self.mouse = {:?}", self.mouse);
                // println!("msg = {:?}", msg);
                match msg {
                    hotmouse::StateMessage::MoveTo(pt) => self.mouse.pt = pt,
                    hotmouse::StateMessage::ButtonPress(ctor) => {
                        self.mouse.press = ctor(Instant::now(), self.mouse.pt);
                        match self.mouse.press {
                            ButtonPress::Middle(_, pt) => {
                                return Command::from(async move {
                                    println!("pt = {:?}", pt);
                                    Message::ScrollIGuessHopefully(pt)
                                })
                            }
                            ButtonPress::Left(_, _)
                            | ButtonPress::Right(_, _) => {}
                            ButtonPress::None => unreachable!("Pressed a non-existent button?!?"),
                        }
                    },
                    hotmouse::StateMessage::ButtonRelease(button) => {
                        use iced::mouse::Button as Button;
                        match (button, self.mouse.press) {
                            (Button::Right, ButtonPress::Right(_, pt)) => {
                                if let Some(message) = hotmouse::gesture(self.mouse.pt - pt) {
                                    return self.update(message)
                                }
                            }
                            // (Button::Left, ButtonPress::Left(_, _))
                            // | (Button::Middle, ButtonPress::Middle(_, _)) => {}
                            _ => {}
                        };
                        if self.mouse.press == button {
                            self.mouse.press = hotmouse::ButtonPress::None;
                        }
                    }
                }
            }
            Message::ScrollIGuessHopefully(pt) => {
                println!("matched: {:?}", pt);
            }
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(|event, _status| {
            match event {
                Event::Keyboard(e) => hotkey::handle(e),
                Event::Window(e) => match e {
                    window::Event::Resized { width, .. } => Some(Message::Resize(width)),
                    _ => None,
                },
                Event::Mouse(e) => hotmouse::handle(e),
            }
        })
    }

    fn view(&mut self) -> Element<Self::Message> {
        let tab = self.tabs.state.clone();

        // top bar: tabs, column width slider, toggle light/dark mode
        let col_slider_reset = Button::new(
            &mut self.col_reset,
            Text::new("Reset")
                .vertical_alignment(VerticalAlignment::Center)
                .size(12),
        ).on_press(Message::SetColScale(1.0)).style(self.style);
        let slider_text = Text::new(
            format!("{:3.0}%", self.col_scale * 100.0)
        ).size(10).vertical_alignment(VerticalAlignment::Bottom);
        let col_slider = Slider::new(
            &mut self.col_slider,
            0.5..=4.0,
            self.col_scale,
            Message::SetColScale,
        )
            .width(Length::Units(80))
            .step(0.01)
            // .style(self.style)
            ;
        let toggle_style = Button::new(
            &mut self.style_button,
            Text::new(self.style).font(FONT),
        ).style(self.style)
            .on_press(Message::ToggleTheme);
        let tabs = Row::new()
            .push(Space::with_width(Length::FillPortion(5)))
            .push(self.tabs.view(self.style).width(Length::Shrink))
            .push(Space::with_width(Length::FillPortion(4)))
            .push(Row::new()
                .push(col_slider_reset)
                .push(col_slider)
                .push(slider_text)
                .push(toggle_style)
                .width(Length::Shrink));

        let mut column = Column::new()
            .push(tabs)
            .push(Rule::horizontal(20));

        column = match tab {
            Tab::Search => column.push(self.search_page.view(self.style)),
            Tab::Character(name) => match self.character_pages.get_mut(&name) {
                Some(page) => column.push(page.view(self.num_cols, self.style)),
                None => column,
            },
            Tab::New => column.push(self.new_page.view(self.style)),
        };

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(self.style)
            .into()
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Class {
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
    pub const ALL: [PLOption<Self>; 8] = [
        PLOption::Some(Self::Bard),
        PLOption::Some(Self::Cleric),
        PLOption::Some(Self::Druid),
        PLOption::Some(Self::Paladin),
        PLOption::Some(Self::Ranger),
        PLOption::Some(Self::Sorcerer),
        PLOption::Some(Self::Warlock),
        PLOption::Some(Self::Wizard),
    ];
}

impl Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
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
    pub const ALL: [PLOption<Self>; 8] = [
        PLOption::Some(Self::Abjuration),
        PLOption::Some(Self::Conjuration),
        PLOption::Some(Self::Divination),
        PLOption::Some(Self::Enchantment),
        PLOption::Some(Self::Evocation),
        PLOption::Some(Self::Illusion),
        PLOption::Some(Self::Transmutation),
        PLOption::Some(Self::Necromancy),
    ];
}

impl Display for School {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl From<School> for String {
    fn from(school: School) -> Self {
        school.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(try_from = "DeserSpell")]
pub struct Spell {
    name: &'static str,
    level: usize,
    casting_time: &'static str,
    range: &'static str,
    duration: &'static str,
    components: &'static str,
    school: School,
    ritual: bool,
    conc: bool,
    description: String,
    #[serde(skip_serializing)]
    desc_lower: String,
    higher_levels: Option<String>,
    #[serde(skip_serializing)]
    higher_levels_lower: Option<String>,
    classes: Vec<Class>,
    source: &'static str,
    page: u32,
}

#[derive(Deserialize)]
struct DeserSpell {
    name: &'static str,
    level: usize,
    casting_time: &'static str,
    range: &'static str,
    duration: &'static str,
    components: &'static str,
    school: School,
    ritual: bool,
    conc: bool,
    description: String,
    higher_levels: Option<String>,
    classes: Vec<Class>,
    source: &'static str,
    page: u32,
}

impl TryFrom<DeserSpell> for Spell {
    type Error = Infallible;

    fn try_from(value: DeserSpell) -> Result<Self, Self::Error> {
        let desc_lower = value.description.to_lowercase();
        let higher_levels_lower = value.higher_levels
            .as_ref()
            .map(|s| s.to_lowercase());
        Ok(Self {
            name: value.name,
            level: value.level,
            casting_time: value.casting_time,
            range: value.range,
            duration: value.duration,
            components: value.components,
            school: value.school,
            ritual: value.ritual,
            conc: value.conc,
            description: value.description,
            desc_lower,
            higher_levels: value.higher_levels,
            higher_levels_lower,
            classes: value.classes,
            source: value.source,
            page: value.page,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpellId {
    name: &'static str,
    level: usize,
}

impl PartialEq<Spell> for SpellId {
    fn eq(&self, other: &Spell) -> bool {
        self.level == other.level && self.name == other.name
    }
}

impl PartialEq<SpellId> for Spell {
    fn eq(&self, other: &SpellId) -> bool {
        other == self
    }
}

fn space() -> Space {
    Space::with_width(Length::Fill)
}

fn text<T: Into<String>>(label: T) -> Row<'static, Message> {
    Row::new()
        .push(space())
        .push(Text::new(label).size(16).width(Length::FillPortion(18)))
        .push(space())
}

pub trait SpellButtonTrait<'a> {
    fn view(self, id: SpellId, style: Style) -> Row<'a, Message>;
}

impl Spell {
    fn id(&self) -> SpellId {
        SpellId {
            name: self.name,
            level: self.level,
        }
    }

    fn view<'a, B: SpellButtonTrait<'a>>(&'a self, button: B, style: Style) -> Column<'a, Message> {
        let title = Row::new()
            .push(space())
            .push(Text::new(self.name).size(36).width(Length::FillPortion(18)))
            .push(space());

        let buttons = Row::new()
            .push(space())
            .push(button.view(self.id(), style).width(Length::FillPortion(18)))
            .push(space());

        let mut column = Column::new()
            .push(title)
            .push(buttons)
            .push(Rule::horizontal(8))
            .push(text(self.school))
            .push(Space::with_height(Length::Units(4)))
            .push(text(format!("Level: {}", self.level)))
            .push(text(format!("Casting time: {}", self.casting_time)))
            .push(text(format!("Range: {}", self.range)))
            .push(text(format!("Components: {}", self.components)))
            .push(text(format!("Duration: {}", self.duration)))
            .push(text(format!("Ritual: {}", if self.ritual { "Yes" } else { "No" })))
            .push(Rule::horizontal(10))
            .push(text(&self.description));
        if let Some(higher) = &self.higher_levels {
            column = column
                .push(Rule::horizontal(8))
                .push(Row::new()
                    .push(space())
                    .push(Text::new("At higher levels").size(20).width(Length::FillPortion(18)))
                    .push(space()))
                .push(Space::with_height(Length::Units(3)))
                .push(text(higher));
        }
        let classes = self.classes.iter().list_grammatically();
        column = column
            .push(Rule::horizontal(8))
            .push(text(format!("A {} spell, from {} Page {}", classes, self.source, self.page)));
        column
    }
}

trait IterExt: ExactSizeIterator + Sized {
    fn list_grammatically(self) -> String where Self::Item: Display {
        let last = self.len() - 1;
        self.enumerate()
            .fold(String::new(), |mut acc, (i, new)| {
                if i != 0 {
                    acc.push_str(if i == last {
                        if i == 1 {
                            " and "
                        } else {
                            ", and "
                        }
                    } else {
                        ", "
                    });
                }
                acc = format!("{}{}", acc, new);
                acc
            })
    }
}

impl<T: Display, I: ExactSizeIterator<Item=T>> IterExt for I {}