// ignored on other targets
#![windows_subsystem = "windows"]

#![warn(clippy::pedantic)]
// @formatter:off
#![allow(
clippy::module_name_repetitions,
clippy::items_after_statements,
clippy::too_many_lines,
clippy::default_trait_access,
clippy::cast_sign_loss,
clippy::option_if_let_else,
clippy::shadow_unrelated,
clippy::redundant_static_lifetimes,
clippy::wildcard_imports,
clippy::enum_glob_use,
)]
// @formatter:on

use std::cmp::min;
use std::convert::TryFrom;
use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write as _};
use std::mem;
use std::ops::{Deref, Not};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

// use iced_native
use iced::{Align, Application, Button, button, Column, Command, Container, Element, Length, pick_list, ProgressBar, Row, Rule, Settings, Slider, slider, Text, text_input, Tooltip, VerticalAlignment};
use iced::mouse::ScrollDelta;
use iced::tooltip::Position;
use iced::window::Icon;
use iced_aw::{ICON_FONT, TabLabel, Tabs};
use iced_native::{Event, Subscription, window};
use itertools::Either;
use once_cell::sync::Lazy;
use self_update::cargo_crate_version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as _;

use search::SearchPage;
use utils::ListGrammaticallyExt;

use crate::character::{Character, CharacterPage, SerializeCharacter};
use crate::hotkey::Move;
use crate::hotmouse::{ButtonPress, Pt};
use crate::search::{PLOption, Unwrap};
use crate::settings::{ClosedCharacter, Edit, SettingsPage, SpellEditor};
use crate::style::{SettingsBarStyle, Style};
use crate::tabs::Tab;
use crate::utils::{SpacingExt, Tap, TryRemoveExt};

mod fetch;
mod style;
mod search;
mod tabs;
mod settings;
mod character;
mod hotkey;
mod hotmouse;
mod utils;
mod update;

const JSON: &str = include_str!("../resources/spells.json");

pub static SPELLS: Lazy<Vec<Spell>> = Lazy::new(|| serde_json::from_str(JSON).expect("json error in `data/spells.json`"));

static SAVE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = dirs::data_local_dir().unwrap_or_default()
        .join("dndspells");
    std::fs::create_dir_all(&path).unwrap();
    path
});

fn get_file(name: &str) -> PathBuf {
    let mut path = SAVE_DIR.clone();
    path.push(name);
    std::fs::OpenOptions::new().create(true).append(true).open(&path).unwrap();
    path
}

static CHARACTER_FILE: Lazy<PathBuf> = Lazy::new(|| get_file("characters.json"));
static CLOSED_CHARACTER_FILE: Lazy<PathBuf> = Lazy::new(|| get_file("closed-characters.json"));
static SPELL_FILE: Lazy<PathBuf> = Lazy::new(|| get_file("custom-spells.json"));

// static ICON: Lazy<Icon> = Lazy::new(|| );
fn icon() -> Icon {
    const LOGO: &[u8] = include_bytes!("../resources/logo.png");
    const WIDTH: u32 = 1500;
    const HEIGHT: u32 = 1500;
    let image = image::load_from_memory(LOGO).expect("failed to read logo");

    Icon::from_rgba(image.into_bytes(), WIDTH, HEIGHT).unwrap()
}

const WIDTH: u32 = 1100;

/// want two columns for starting window size with a bit of room to expand
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
const COLUMN_WIDTH: f32 = WIDTH as f32 * 1.1 / 2.0;

fn main() -> iced::Result {
    if let Some("TARGET") = std::env::args().nth(1).as_deref() {
        println!("{}", self_update::get_target());
        return Ok(());
    }

    DndSpells::run(Settings {
        window: iced::window::Settings {
            min_size: Some((1024 / 2, 500)),
            // default: (1024, 768)
            size: (WIDTH, 768),
            icon: Some(icon()),
            ..Default::default()
        },
        // default_font: Some(include_bytes!("../resources/arial.ttf")),
        default_text_size: 18,
        antialiasing: true,
        ..Default::default()
    })
}

#[derive(Debug)]
pub enum UpdateState {
    Checking,
    Ready,
    Downloading(f32),
    UpToDate,
    Downloaded,
    Errored(String),
}

impl UpdateState {
    pub fn view(&self, style: SettingsBarStyle) -> Element<crate::Message> {
        const VER: &str = cargo_crate_version!();
        match self {
            &Self::Downloading(pct) => {
                Row::new()
                    .align_items(Align::Center)
                    .push(Text::new("Downloading").size(10))
                    .push_space(5)
                    .push(ProgressBar::new(0.0..=100.0, pct)
                        .style(style)
                        .height(Length::Units(12)) // bottom bar is 20 pts
                        .width(Length::Units(100)))
                    .into()
            }
            view_as_text => match view_as_text {
                Self::Checking => Text::new("Checking for updates..."),
                Self::Ready => Text::new("Preparing to download..."),
                Self::Downloaded => Text::new("Downloaded new version! Restart program to get new features!"),
                Self::UpToDate => Text::new(format!("Up to date, v{}", VER)),
                Self::Errored(e) => Text::new(format!("Error downloading new version: {}. Running v{}", e, VER)),
                Self::Downloading(_) => unreachable!(),
            }.size(10).into()
        }
    }
}

pub struct DndSpells {
    update_state: UpdateState,
    update_url: String,
    style: Style,
    tab: Tab,
    width: u32,
    height: u32,
    control_pressed: bool,
    search_page: SearchPage,
    characters: Vec<CharacterPage>,
    closed_characters: Vec<ClosedCharacter>,
    settings_page: SettingsPage,
    pub col_scale: f32,
    col_reset: button::State,
    col_slider: slider::State,
    style_button: button::State,
    /// Vec<(characters, closed_characters)>
    save_states: Vec<(Vec<SerializeCharacter>, Vec<SerializeCharacter>)>,
    state: Option<usize>,
    custom_spells: Vec<CustomSpell>,
    num_cols: usize,
    mouse: hotmouse::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Update(update::Message),
    ToggleTheme,
    SetColScale(f32),
    // SwitchTab(Tab),
    Search(search::Message),
    Settings(settings::Message),
    Character(usize, character::Message),
    MoveCharacter(usize, isize),
    CloseCharacter(usize),
    Hotkey(hotkey::Message),
    MouseState(hotmouse::StateMessage),
    ScrollIGuessHopefully(Pt),
    Resize(u32, u32),
    SelectTab(usize),
    CloseTab(usize),
}

impl DndSpells {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn set_num_columns(&mut self) {
        self.num_cols = (self.width as f32 / (COLUMN_WIDTH * self.col_scale)).ceil() as _;
    }

    fn add_character<C: Into<CharacterPage>>(&mut self, character: C) {
        self.characters.push(character.into());
        self.refresh_search();
        self.tab = Tab::Character { index: self.characters.len() - 1 };
        self.save().expect("failed to save");
    }

    fn swap_characters(&mut self, a: usize, b: usize) {
        self.characters.swap(a, b);
        self.refresh_search();
        self.save().expect("blah");
    }

    fn close_character(&mut self, character: usize) {
        let character = self.characters.remove(character);
        self.tab = match self.tab {
            Tab::Character { index } if index >= self.characters.len() => Tab::Character {
                index: self.characters.len().saturating_sub(1)
            },
            tab => tab,
        };
        self.closed_characters.push(character.character.into());
        self.refresh_search();
        self.save().expect("waa haa");
    }

    // todo spells save state, they keybinds should do that when the spell editor is open3
    fn save_state(&mut self) {
        if let Some(idx) = self.state.take() {
            self.save_states.truncate(idx + 1);
        }
        let characters = self.characters.iter()
            .map(|page| page.character.serialize())
            .collect();
        let closed = self.closed_characters.iter()
            .map(|closed| closed.character.serialize())
            .collect();
        self.save_states.push((characters, closed));
    }

    fn load_state(&mut self, idx: usize) {
        let (characters, closed) = self.save_states.get(idx).unwrap();
        let custom = &self.custom_spells;
        self.characters = characters.iter()
            .map(|c| Character::from_serialized(c, custom))
            .map(CharacterPage::from)
            .collect();
        self.closed_characters = closed.iter()
            .map(|c| Character::from_serialized(c, custom))
            .map(ClosedCharacter::from)
            .collect();
    }

    fn read_characters<C: From<Character>>(file: &Path, custom: &[CustomSpell]) -> anyhow::Result<Vec<C>> {
        match File::open(file) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut characters = Vec::new();
                for line in reader.lines() {
                    let line = line.unwrap();
                    let serialized = serde_json::from_str(&line)?;
                    let c = Character::from_serialized(&serialized, custom);
                    characters.push(C::from(c));
                }
                Ok(characters)
            }
            Err(e) if matches!(e.kind(), ErrorKind::NotFound) => {
                File::create(file)?;
                Ok(Vec::default())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn read_spells(file: &Path) -> anyhow::Result<Vec<CustomSpell>> {
        match File::open(file) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut spells = Vec::new();
                for line in reader.lines() {
                    let line = line.unwrap();
                    spells.push(serde_json::from_str(&line)?);
                }
                Ok(spells)
            }
            Err(e) if matches!(e.kind(), ErrorKind::NotFound) => {
                File::create(file)?;
                Ok(Vec::new())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn set_spells_characters(&mut self) {
        self.custom_spells = Self::read_spells(&*SPELL_FILE)
            .unwrap_or_default();
        self.characters = Self::read_characters(&*CHARACTER_FILE, &self.custom_spells)
            .unwrap_or_default();
        self.closed_characters = Self::read_characters(&*CLOSED_CHARACTER_FILE, &self.custom_spells)
            .unwrap_or_default();
        self.settings_page = SettingsPage::new(&self.custom_spells);
    }

    fn open() -> Self {
        let (width, height) = iced::window::Settings::default().size;
        let mut window = Self {
            update_state: UpdateState::Checking,
            update_url: "".to_string(),
            style: Style::default(),
            tab: Tab::Search,
            width,
            height,
            control_pressed: false,
            search_page: Default::default(),
            characters: vec![],
            closed_characters: vec![],
            settings_page: Default::default(),
            col_scale: 1.0,
            col_reset: Default::default(),
            col_slider: Default::default(),
            style_button: Default::default(),
            save_states: Default::default(),
            state: None,
            custom_spells: vec![],
            num_cols: 2,
            mouse: Default::default(),
        };
        window.set_spells_characters();
        window.save_state();
        window
    }

    fn save(&mut self) -> anyhow::Result<()> {
        self.save_state();
        let mut file = File::create(&*CHARACTER_FILE)?;
        for c in &self.characters {
            serde_json::to_writer(&mut file, &c.character.serialize())?;
            file.write_all(b"\n")?;
        }
        let mut file = File::create(&*CLOSED_CHARACTER_FILE)?;
        for c in &self.closed_characters {
            serde_json::to_writer(&mut file, &c.character.serialize())?;
            file.write_all(b"\n")?;
        }
        let mut file = File::create(&*SPELL_FILE)?;
        for spell in &self.custom_spells {
            serde_json::to_writer(&mut file, &spell)?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    fn refresh_search(&mut self) {
        self.search_page.update(search::Message::Refresh, &self.custom_spells, &self.characters);
    }
}

impl Application for DndSpells {
    type Executor = iced_futures::executor::Tokio;
    type Message = Message;
    type Flags = ();

    fn new((): Self::Flags) -> (Self, Command<Message>) {
        let window = Self::open();
        let commands = Command::batch([
            async { Message::Search(search::Message::Refresh) }.into(),
            async {
                // wait briefly to so that loading doesn't take so long
                tokio::time::sleep(Duration::from_millis(500)).await;
                Message::Update(update::Message::CheckForUpdate)
            }.into(),
        ]);
        (window, commands)
    }

    fn title(&self) -> String {
        const SPELLS: &str = "D&D Spells";
        match self.tab {
            Tab::Search | Tab::Settings => SPELLS.into(),
            Tab::Character { index } => format!(
                "{} - {}",
                SPELLS,
                self.characters.get(index)
                    .map_or("Character", |c| &c.character.name)
            )
        }
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut iced::Clipboard) -> Command<Message> {
        match message {
            Message::Update(msg) => {
                if let Err(e) = update::handle(self, msg) {
                    self.update_state = UpdateState::Errored(e.to_string())
                }
                if let UpdateState::Downloaded = &self.update_state {
                    self.set_spells_characters();
                }
            },
            Message::ToggleTheme => self.style = match self.style {
                Style::Light => Style::Dark,
                Style::Dark => Style::Light,
            },
            Message::SetColScale(mult) => {
                println!("mult = {:?}", mult);
                self.col_scale = mult;
                self.set_num_columns();
            }
            Message::Search(msg) => self.search_page.update(msg, &self.custom_spells, &self.characters),
            Message::Settings(message) => {
                use settings::Message;
                match message {
                    Message::CharacterName(name) => {
                        self.settings_page.character_name = name;
                    }
                    Message::SubmitCharacter => {
                        self.settings_page.character_name_state.focus();
                        let name = &mut self.settings_page.character_name;
                        if !name.is_empty() && !self.characters.iter().any(|page| &*page.character.name == name) {
                            let name = Arc::<str>::from(mem::take(name));
                            self.add_character(name);
                        } else {
                            // todo notify in gui somehow
                            println!("{} is already a character", name);
                        }
                    }
                    Message::Open(index) => {
                        let character = self.closed_characters.remove(index);
                        self.add_character(character.character);
                    }
                    Message::Rename(index) => {
                        let rename = match &mut self.closed_characters[index].rename {
                            Either::Left(_) => {
                                Either::Right(Default::default())
                            }
                            Either::Right((_, name, _)) => {
                                if !name.is_empty() {
                                    let name = std::mem::take(name);
                                    self.closed_characters[index].character.name = Arc::from(name);
                                    self.save().expect("ASDSADAS");
                                }
                                Either::Left(Default::default())
                            }
                        };
                        self.closed_characters[index].rename = rename;
                    }
                    Message::RenameString(index, new) => {
                        if let Either::Right((_, name, _)) = &mut self.closed_characters[index].rename {
                            *name = new;
                        }
                    }
                    Message::DeleteCharacter(index) => {
                        self.closed_characters.remove(index);
                        self.save().expect("todoooooo");
                    }
                    Message::SpellName(name) => {
                        let name = {
                            let lower = name.to_lowercase();
                            self.settings_page.spell_name = name;
                            lower
                        };
                        if let Some(spell) = self.custom_spells.iter()
                            .find(|spell| spell.name_lower == name)
                            .cloned() {
                            self.settings_page.spell_editor = SpellEditor::Editing { spell }
                        } else {
                            self.settings_page.spell_editor = SpellEditor::searching(&name, &self.custom_spells);
                        }
                    }
                    Message::SubmitSpell => {
                        let name = mem::take(&mut self.settings_page.spell_name);
                        let spell = CustomSpell::new(name);
                        self.custom_spells.push(spell.clone());
                        self.settings_page.spell_editor = SpellEditor::Editing { spell };
                        self.save().unwrap();
                    }
                    Message::OpenSpell(index) => {
                        if let SpellEditor::Searching { spells } = &mut self.settings_page.spell_editor {
                            if let Some((spell, _, _, _)) = spells.try_remove(index) {
                                self.settings_page.spell_editor = SpellEditor::Editing { spell };
                            }
                        }
                    }
                    Message::DeleteSpell(index) => {
                        if let SpellEditor::Searching { spells } = &mut self.settings_page.spell_editor {
                            let spell = spells.remove(index).0;
                            if let Some(index) = self.custom_spells.iter().position(|cs| *cs == spell) {
                                self.custom_spells.remove(index);
                            }
                            self.save().unwrap();
                        }
                    }
                    Message::EditSpell(edit) => match &mut self.settings_page.spell_editor {
                        SpellEditor::Searching { .. } => unreachable!(),
                        SpellEditor::Editing { spell } => {
                            let nullify = |s: String| s.is_empty().not().then(|| s);
                            match edit {
                                Edit::School(school) => spell.school = school,
                                Edit::Level(level) => spell.level = level,
                                Edit::CastingTime(time) => spell.casting_time = time,
                                Edit::CastingTimeN(new) => {
                                    if let Ok(new) = new.parse() {
                                        match &mut spell.casting_time {
                                            CastingTime::Minute(n) | CastingTime::Hour(n) => *n = new,
                                            _ => {}
                                        }
                                    }
                                }
                                Edit::CastingTimeWhen(new) => if let CastingTime::Reaction(when) = &mut spell.casting_time {
                                    *when = Some(StArc::Arc(Arc::from(new)));
                                },
                                Edit::Range(range) => spell.range = range,
                                Edit::ComponentV(v) => match &mut spell.components {
                                    Some(components) => components.v = v,
                                    none @ None => *none = Some(Components { v: true, s: false, m: None }),
                                },
                                Edit::ComponentS(s) => match &mut spell.components {
                                    Some(components) => components.s = s,
                                    none @ None => *none = Some(Components { v: false, s: true, m: None }),
                                },
                                Edit::ComponentM(m) => match &mut spell.components {
                                    Some(components) => components.m = m.then(String::new),
                                    none @ None => *none = Some(Components { v: false, s: false, m: Some(String::new()) }),
                                },
                                Edit::ComponentMaterial(mat) => match &mut spell.components {
                                    Some(components) => components.m = Some(mat),
                                    None => spell.components = Some(Components { v: false, s: false, m: Some(mat) }),
                                },
                                Edit::Duration(duration) => spell.duration = duration,
                                Edit::Ritual(ritual) => spell.ritual = ritual,
                                Edit::Concentration(conc) => spell.conc = conc,
                                Edit::Description(mut desc) => {
                                    loop {
                                        const NEWLINE: &str = "\\n";
                                        if let Some(idx) = desc.find(NEWLINE) {
                                            desc.replace_range(idx..(idx + NEWLINE.len()), "\n");
                                        } else {
                                            break;
                                        }
                                    }
                                    spell.description = desc;
                                }
                                // Edit::DescEnter => {
                                //     spell.description_state.cursor().
                                //     println!("spell.description = {:?}", spell.description);
                                //     spell.description.push('\n');
                                //     println!("spell.description = {:?}", spell.description);
                                // }
                                Edit::HigherLevels(higher) => spell.higher_levels = nullify(higher),
                                Edit::Class(class) => {
                                    let class = class.unwrap();
                                    if let Some(idx) = spell.classes.iter().position(|&c| c == class) {
                                        spell.classes.remove(idx);
                                    } else {
                                        spell.classes.push(class);
                                    }
                                }
                                // Edit::Source(source) => spell.source = source,
                                // Edit::Page(page) => if page.is_empty() {
                                //     spell.page = None;
                                // } else if let Ok(page) = page.parse() {
                                //     spell.page = Some(page);
                                // },
                            };
                            if let Some(saved_spell) = self.custom_spells.iter_mut().find(|s| s.name == spell.name) {
                                *saved_spell = spell.clone();
                            } else {
                                self.custom_spells.push(spell.clone());
                            }
                            self.refresh_search();
                            self.save().unwrap();
                        }
                    },
                    Message::CloseSpell => {
                        self.settings_page.spell_editor = SpellEditor::searching(
                            &self.settings_page.spell_name.to_lowercase(),
                            &self.custom_spells,
                        );
                    }
                }
            }
            Message::Character(index, msg) => {
                let add = matches!(msg, character::Message::AddSpell(_));
                let num_cols = self.num_cols;
                let custom = &self.custom_spells;
                let must_save = self.characters.get_mut(index)
                    .map(|c| c.update(msg, custom, num_cols));
                // let must_save = self.character_pages.get_mut(&name)
                //     .map(|c| c.update(msg, num_cols));
                if add {
                    self.search_page.search.state.focus();
                    // have to update after adding the spell
                    self.refresh_search();
                }
                if let Some(true) = must_save {
                    self.refresh_search();
                    self.save().expect("todo #2");
                }
            }
            Message::MoveCharacter(idx, delta) => {
                let new_idx = if delta.is_negative() {
                    idx.saturating_sub(delta.abs() as usize)
                } else {
                    min(idx + delta as usize, self.characters.len() - 1)
                };
                self.swap_characters(idx, new_idx);
                self.tab = Tab::Character { index: new_idx };
            }
            Message::CloseCharacter(index) => {
                // todo currently just goes to next tab, is that good?
                self.close_character(index);
            }
            Message::Hotkey(message) => {
                use hotkey::Message;
                match message {
                    Message::ToCharacter(index) => {
                        let index = if index == 0 {
                            // go to last tab
                            self.characters.len() - 1
                        } else {
                            index - 1
                        };
                        self.tab = Tab::Character { index }
                        // if let Some(name) = self.characters.get(idx) {
                        //     // self.tabs.state = Tab::Character(Arc::clone(&name))
                        // }
                    }
                    Message::Find(main_page) => {
                        match (main_page, self.tab) {
                            (true, _) => {
                                self.tab = Tab::Search;
                                self.search_page.update(
                                    crate::search::Message::Search(String::new()),
                                    &self.custom_spells,
                                    &self.characters,
                                );
                            }
                            (false, Tab::Settings | Tab::Search) => {
                                self.tab = Tab::Search;
                                self.refresh_search();
                            }
                            (false, Tab::Character { index }) => {
                                if let Some(page) = self.characters.get_mut(index) {
                                    page.tab = 0;
                                    page.search.state.focus();
                                }
                            }
                        }
                    }
                    // Message::NewCharacter => self.tabs.state = Tab::New,
                    Message::NewCharacter => self.tab = Tab::Character { index: self.characters.len() + 1 },
                    Message::Move(dir, tab_only) => {
                        if tab_only {
                            let new_tab_idx = self.characters.len() + 1;
                            let orig_idx = match &self.tab {
                                Tab::Search => 0,
                                // Tab::Character(name) => self.characters.iter()
                                //     .position(|c| c == name)
                                //     .unwrap() + 1,
                                Tab::Character { index } => index + 1,
                                Tab::Settings => new_tab_idx,
                            };
                            let idx = match dir {
                                Move::Left => min(orig_idx.wrapping_sub(1), new_tab_idx),
                                Move::Right => {
                                    let idx = orig_idx + 1;
                                    if idx > new_tab_idx { 0 } else { idx }
                                }
                            };
                            match idx {
                                0 => self.tab = Tab::Search,
                                new_tab if new_tab == new_tab_idx => self.tab = Tab::Settings,
                                idx => {
                                    // let character = &self.characters[idx - 1];
                                    self.tab = Tab::Character { index: idx - 1 };
                                }
                            }
                        } else {
                            let new_tab_idx = self.characters.len() * character::TABS + 1;
                            let orig_idx = match self.tab {
                                Tab::Search => 0,
                                Tab::Character { index } => {
                                    let character = &self.characters[index];
                                    // let character_tab = self.characters.get(character).unwrap().tab;
                                    1 + character::TABS * index + character.tab
                                }
                                Tab::Settings => new_tab_idx,
                            };
                            let idx = match dir {
                                Move::Left => min(orig_idx.wrapping_sub(1), new_tab_idx),
                                Move::Right => {
                                    let idx = orig_idx + 1;
                                    if idx > new_tab_idx { 0 } else { idx }
                                }
                            };
                            match idx {
                                0 => self.tab = Tab::Search,
                                new_tab if new_tab == new_tab_idx => self.tab = Tab::Settings,
                                idx => {
                                    let character = (idx - 1) / character::TABS;
                                    self.tab = Tab::Character { index: character };
                                    let tab = (idx - 1) % character::TABS;
                                    self.characters.get_mut(character).unwrap().tab = tab;
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
                            self.load_state(idx);
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
                        if let Tab::Character { index } = self.tab {
                            if let Some(page) = self.characters.get_mut(index) {
                                page.tab = tab;
                            }
                        }
                    }
                    Message::AddSpell(idx) => {
                        if let Some(spell) = self.search_page.spells.first().map(|s| s.spell.id()) {
                            if let Some(character) = self.characters.get_mut(idx) {
                                let spell = find_spell(&spell.name, &self.custom_spells).unwrap();
                                character.add_spell(spell);
                                self.refresh_search();
                            }
                        }
                    }
                    Message::CustomSpellNextField(forwards) => {
                        if let Tab::Settings = self.tab {
                            let mut states = vec![
                                &mut self.settings_page.character_name_state,
                                &mut self.settings_page.spell_name_state,
                            ];
                            if let SpellEditor::Editing { spell } = &mut self.settings_page.spell_editor {
                                states.extend([
                                    &mut spell.casting_time_extra_state,
                                    &mut spell.range_state
                                ]);
                                if matches!(&spell.components, Some(Components { m: Some(_), .. })) {
                                    states.extend([&mut spell.material_state]);
                                }
                                states.extend([
                                    // &mut spell.components_state,
                                    &mut spell.duration_state,
                                    &mut spell.description_state,
                                    &mut spell.higher_levels_state,
                                    // &mut spell.source_state,
                                    // &mut spell.page_state
                                ]);
                            }
                            for i in 0..states.len() {
                                if states[i].is_focused() {
                                    if forwards && i != states.len() - 1 {
                                        states[i].unfocus();
                                        states[i + 1].focus();
                                        break;
                                    } else if !forwards && i != 0 {
                                        states[i].unfocus();
                                        states[i - 1].focus();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::Resize(width, height) => {
                self.width = width;
                self.height = height;
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
                                });
                            }
                            ButtonPress::Left(_, _)
                            | ButtonPress::Right(_, _) => {}
                            ButtonPress::None => unreachable!("Pressed a non-existent button?!?"),
                        }
                    }
                    hotmouse::StateMessage::ButtonRelease(button) => {
                        use iced::mouse::Button as Button;
                        if let (Button::Right, ButtonPress::Right(_, pt)) = (button, self.mouse.press) {
                            if let Some(message) = hotmouse::gesture(self.mouse.pt - pt) {
                                return self.update(message, clipboard);
                            }
                        };
                        if self.mouse.press == button {
                            self.mouse.press = hotmouse::ButtonPress::None;
                        }
                    }
                    hotmouse::StateMessage::Scroll(delta) => {
                        if self.control_pressed {
                            let delta = match delta {
                                ScrollDelta::Lines { y, .. }
                                | ScrollDelta::Pixels { y, .. } => y,
                            };
                            println!("delta = {:?}", delta);
                            self.col_scale += delta;
                        }
                    }
                }
            }
            Message::ScrollIGuessHopefully(pt) => {
                println!("matched: {:?}", pt);
            }
            Message::SelectTab(index) => {
                self.tab = match index {
                    0 => Tab::Search,
                    last if last == self.characters.len() + 1 => Tab::Settings,
                    index => Tab::Character { index: index - 1 }
                }
            }
            Message::CloseTab(tab) => {
                println!("close tab = {:?}", tab);
            }
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let listeners = iced_native::subscription::events_with(|event, _status| {
            match event {
                Event::Keyboard(e) => hotkey::handle(e),
                Event::Window(e) => match e {
                    window::Event::Resized { width, height } => Some(Message::Resize(width, height)),
                    _ => None,
                },
                Event::Mouse(e) => hotmouse::handle(e),
                Event::Touch(_) => None,
            }
        });
        match &self.update_state {
            UpdateState::Ready | UpdateState::Downloading(_) => {
                let download = Subscription::from_recipe(update::Download { url: self.update_url.clone() })
                    .map(|p| Message::Update(update::Message::Progress(p)));
                Subscription::batch([
                    listeners,
                    download,
                ])
            }
            _ => listeners
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let style = self.style;
        let num_cols = self.num_cols;
        let num_characters = self.characters.len();

        let height = self.height
            .saturating_sub(26)  // height of tab bar
            .saturating_sub(20); // height of bottom bar

        let tabs = Tabs::new(self.tab.index(num_characters), crate::Message::SelectTab)
            .push(TabLabel::Text("Search".into()), self.search_page.view(style).max_height(height));
        let tabs = self.characters.iter_mut()
            .enumerate()
            .map(|(index, page)| (
                TabLabel::Text(page.character.name.to_string()),
                page.view(index, num_cols, style).max_height(height)
            )).fold(
            tabs,
            |tabs, (label, tab)| tabs.push(label, tab),
        ).push(TabLabel::Text("Settings".into()), self.settings_page.view(&mut self.closed_characters, self.width, style).max_height(height))
            .tab_bar_style(style)
            .icon_size(10)
            .icon_font(ICON_FONT)
            // .on_close(Message::CloseTab)
            ;

        let mut col_slider_reset = Button::new(
            &mut self.col_reset,
            Text::new("Reset")
                .vertical_alignment(VerticalAlignment::Center)
                .size(12),
        ).style(style.settings_bar());
        // set to exactly 1.0 in code so it's fine
        #[allow(clippy::float_cmp)]
        if self.col_scale != 1.0 {
            col_slider_reset = col_slider_reset.on_press(crate::Message::SetColScale(1.0));
        }

        // todo monospace font and pad with spaces
        let slider_text = Text::new(
            format!("{:3.0}%", self.col_scale * 100.0)
        ).size(10)
            .vertical_alignment(VerticalAlignment::Center);

        let col_slider = Slider::new(
            &mut self.col_slider,
            0.5..=4.0,
            self.col_scale,
            crate::Message::SetColScale,
        )
            .width(Length::Units(120))
            .step(0.01)
            .style(style);

        let toggle_style = Button::new(
            &mut self.style_button,
            Text::new(iced_aw::Icon::BrightnessHigh)
                .font(ICON_FONT)
                .size(12),
        ).style(style.settings_bar())
            .on_press(crate::Message::ToggleTheme);

        let toggle_style = Tooltip::new(
            toggle_style,
            format!("Switch to {} theme", !style),
            Position::Top,
        ).size(10);

        let bottom_bar = Container::new(Row::new()
            .spacing(2)
            .push_space(4)
            .push(self.update_state.view(style.settings_bar()))
            .push_space(Length::Fill)
            .push(col_slider_reset)
            .push(col_slider)
            .push(slider_text)
            .push(toggle_style)
            .height(Length::Units(20))
            .align_items(Align::Center)
        ).style(style.settings_bar())
            .align_y(Align::Center);

        let content = Column::new()
            .push(tabs.height(Length::Shrink))
            .push_space(Length::Fill)
            .push(bottom_bar);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .align_y(Align::End)
            .style(style)
            .into()
    }
}

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

    pub const PL_ALL: [PLOption<Self>; 9] = [
        PLOption::Some(Self::Artificer),
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

    // todo use array::map when that's const stable
    pub const PL_ALL: [PLOption<Self>; 8] = [
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

    fn from_static(str: &'static str) -> Result<Self, String> {
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
}

impl Display for CastingTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Special => f.write_str("Special"),
            Self::Action => f.write_str("1 Action"),
            Self::BonusAction => f.write_str("1 Bonus Action"),
            Self::Reaction(when) => if let Some(when) = when {
                write!(f, "1 Reaction, which you take when {}", when)
            } else {
                f.write_str("1 Reaction")
            }
            &Self::Minute(n) => write!(f, "{} Minute{}", n, if n == 1 { "" } else { "s" }),
            &Self::Hour(n) => write!(f, "{} Hour{}", n, if n == 1 { "" } else { "s" }),
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
    v: bool,
    s: bool,
    m: Option<String>,
}

impl Display for Components {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    Custom,
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(Self::STRINGS[*self as usize])
    }
}

impl Source {
    const ALL: [Self; 4] = [
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(try_from = "DeserializeSpell")]
pub struct Spell {
    name: &'static str,
    #[serde(skip_serializing)]
    name_lower: &'static str,
    level: usize,
    casting_time: CastingTime,
    range: &'static str,
    duration: &'static str,
    components: Components,
    school: School,
    ritual: bool,
    conc: bool,
    description: &'static str,
    #[serde(skip_serializing)]
    desc_lower: &'static str,
    higher_levels: Option<&'static str>,
    #[serde(skip_serializing)]
    higher_levels_lower: Option<&'static str>,
    classes: &'static [Class],
    source: Source,
    page: u32,
}

#[derive(Deserialize)]
struct DeserializeSpell {
    name: &'static str,
    level: usize,
    casting_time: &'static str,
    range: &'static str,
    duration: &'static str,
    components: Components,
    school: School,
    ritual: bool,
    conc: bool,
    description: String,
    higher_levels: Option<String>,
    classes: Vec<Class>,
    source: Source,
    page: u32,
}

impl TryFrom<DeserializeSpell> for Spell {
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

// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
// pub struct PickSpellLevel(pub usize);
//
// impl PLNone for PickSpellLevel {
//     fn title() -> &'static str {
//         "Pick a level"
//     }
// }
//
// impl Debug for PickSpellLevel {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         <usize as Debug>::fmt(&self.0, f)
//     }
// }
//
// impl Display for PickSpellLevel {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         <usize as Display>::fmt(&self.0, f)
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomSpell {
    name: Arc<str>,
    name_lower: String,
    // todo this was for allowing spells to be renamed
    #[serde(skip)]
    name_state: text_input::State,
    level: usize,
    #[serde(skip)]
    level_state: pick_list::State<usize>,
    casting_time: CastingTime,
    #[serde(skip)]
    casting_time_state: pick_list::State<CastingTime>,
    #[serde(skip)]
    pub casting_time_extra_state: text_input::State,
    // #[serde(skip)]
    // casting_time_state: text_input::State,
    range: String,
    #[serde(skip)]
    pub range_state: text_input::State,
    pub components: Option<Components>,
    #[serde(skip)]
    pub material_state: text_input::State,
    duration: String,
    #[serde(skip)]
    pub duration_state: text_input::State,
    school: School,
    #[serde(skip)]
    school_state: pick_list::State<School>,
    #[serde(default)]
    ritual: bool,
    #[serde(default)]
    conc: bool,
    description: String,
    desc_lower: String,
    #[serde(skip)]
    pub description_state: text_input::State,
    higher_levels: Option<String>,
    higher_levels_lower: Option<String>,
    #[serde(skip)]
    pub higher_levels_state: text_input::State,
    classes: Vec<Class>,
    #[serde(skip)]
    pub classes_state: pick_list::State<PLOption<Class>>,
    page: Option<u32>,
    #[serde(skip)]
    pub page_state: text_input::State,
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
            name_state: Default::default(),
            level: 0,
            level_state: Default::default(),
            casting_time: CastingTime::Action,
            casting_time_state: Default::default(),
            casting_time_extra_state: Default::default(),
            range: String::new(),
            range_state: Default::default(),
            duration: String::new(),
            duration_state: Default::default(),
            components: None,
            material_state: Default::default(),
            school: School::Abjuration,
            school_state: Default::default(),
            ritual: false,
            conc: false,
            description: String::new(),
            desc_lower: String::new(),
            description_state: Default::default(),
            higher_levels: None,
            higher_levels_lower: None,
            higher_levels_state: Default::default(),
            classes: Vec::new(),
            classes_state: Default::default(),
            page: None,
            page_state: Default::default(),
        }
    }

    #[must_use]
    pub fn id(&self) -> SpellId {
        SpellId {
            name: self.name.clone().into(),
            level: self.level,
        }
    }

    pub fn view<'a, B: SpellButtons<'a>>(
        &'a self,
        button: B,
        data: B::Data,
        collapse: bool,
        style: Style,
    ) -> Container<'a, Message> {
        fn text<T: Into<String>>(label: T) -> Row<'static, Message> {
            Row::new()
                .push_space(Length::Fill)
                .push(Text::new(label).size(16).width(Length::FillPortion(18)))
                .push_space(Length::Fill)
        }

        let (buttons, title) = button.view(self.id(), data, style);
        let title = Row::new()
            .push_space(Length::Fill)
            .push(title)
            .push_space(Length::Fill);

        let buttons = Row::new()
            .push_space(Length::Fill)
            .push(buttons.width(Length::FillPortion(18)))
            .push_space(Length::Fill);

        let mut column = Column::new()
            .push(title)
            .push(buttons);
        if !collapse {
            let classes = self.classes.iter().list_grammatically();

            #[allow(clippy::if_not_else)]
                let about = format!(
                // "A {}{}spell{}{}{}{}{}",
                "A {}{}spell",
                classes,
                if !classes.is_empty() { " " } else { "" },
                // if !classes.is_empty() && (!self.source.is_empty() || self.page.is_some()) { "," } else { "" },
                // self.source,
                // if !self.source.is_empty() || self.page.is_some() { " from " } else { "" },
                // if !self.source.is_empty() { " " } else { "" },
                // self.page.map(|p| p.to_string()).as_deref().unwrap_or("")
            );

            column = column
                .push(Rule::horizontal(8))
                .push(text(self.school))
                .push_space(4)
                .push(text(format!("Level: {}", self.level)))
                .push(text(format!("Casting Time: {}", self.casting_time)))
                .push(text(format!("Range: {}", self.range)))
                .push(text(format!("Components: {}", self.components.as_ref().map_or_else(String::new, |c| c.to_string()))))
                .push(text(format!("Duration: {}", self.duration)))
                .push(text(format!("Ritual: {}", if self.ritual { "Yes" } else { "No" })))
                .push(Rule::horizontal(10))
                .push(text(&self.description))
                .tap_if_some(self.higher_levels.as_ref(), |col, higher| col
                    .push(Rule::horizontal(8))
                    .push(Row::new()
                        .push_space(Length::Fill)
                        .push(Text::new("At higher levels").size(20).width(Length::FillPortion(18)))
                        .push_space(Length::Fill))
                    .push_space(3)
                    .push(text(higher)))
                .tap_if(about != "A spell", |col| col
                    .push(Rule::horizontal(8))
                    .push(text(about)))
            ;
        }
        Container::new(column)
    }
}

pub trait SpellButtons<'a> {
    type Data;

    fn view(self, id: SpellId, data: Self::Data, style: Style) -> (Row<'a, Message>, Element<'a, Message>);
}

impl Spell {
    #[must_use]
    pub fn id(&self) -> SpellId {
        SpellId {
            name: self.name.into(),
            level: self.level,
        }
    }

    fn view<'a, B: SpellButtons<'a> + 'a>(
        &'a self,
        button: B,
        data: B::Data,
        collapse: bool,
        style: Style,
    ) -> Container<'a, Message> {
        fn text<T: Into<String>>(label: T) -> Row<'static, Message> {
            Row::new()
                .push_space(Length::Fill)
                .push(Text::new(label).size(16).width(Length::FillPortion(18)))
                .push_space(Length::Fill)
        }

        let (buttons, title) = button.view(self.id(), data, style);
        let title = Row::new()
            .push_space(Length::Fill)
            .push(title)
            .push_space(Length::Fill);

        let buttons = Row::new()
            .push_space(Length::Fill)
            .push(buttons.width(Length::FillPortion(18)))
            .push_space(Length::Fill);

        let mut column = Column::new()
            .push(title)
            .push(buttons);
        if !collapse {
            column = column
                .push(Rule::horizontal(8))
                .push(text(self.school))
                .push_space(4)
                .push(text(format!("Level: {}", self.level)))
                .push(text(format!("Casting time: {}", self.casting_time)))
                .push(text(format!("Range: {}", self.range)))
                .push(text(format!("Components: {}", self.components)))
                .push(text(format!("Duration: {}", self.duration)))
                .push(text(format!("Ritual: {}", if self.ritual { "Yes" } else { "No" })))
                .push(Rule::horizontal(10))
                .push(text(self.description));
            if let Some(higher) = self.higher_levels {
                column = column
                    .push(Rule::horizontal(8))
                    .push(Row::new()
                        .push_space(Length::Fill)
                        .push(Text::new("At higher levels").size(20).width(Length::FillPortion(18)))
                        .push_space(Length::Fill))
                    .push_space(3)
                    .push(text(higher));
            }
            let classes = self.classes.iter().list_grammatically();
            let an_grammar = classes.chars().next()
                .filter(|c| *c == 'A')
                .map_or('\0', |_| 'n');
            column = column
                .push(Rule::horizontal(8))
                .push(text(format!("A{} {} spell, from {} page {}", an_grammar, classes, self.source, self.page)));
        }
        Container::new(column)
    }
}

#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
pub enum StArc<T: ?Sized + 'static> {
    Static(&'static T),
    Arc(Arc<T>),
}

impl<T: ?Sized> Deref for StArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(t) => t,
            Self::Arc(t) => t,
        }
    }
}

impl<T: ?Sized> Clone for StArc<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(t) => Self::Static(*t),
            Self::Arc(t) => Self::Arc(t.clone()),
        }
    }
}

impl<T: ?Sized> From<&'static T> for StArc<T> {
    fn from(t: &'static T) -> Self {
        Self::Static(t)
    }
}

impl<T: ?Sized> From<Arc<T>> for StArc<T> {
    fn from(t: Arc<T>) -> Self {
        Self::Arc(t)
    }
}

impl<'a, T: ?Sized> From<&'a Arc<T>> for StArc<T> {
    fn from(t: &'a Arc<T>) -> Self {
        Self::Arc(Arc::clone(t))
    }
}

impl<'de, T: ?Sized> Deserialize<'de> for StArc<T>
    where Arc<T>: Deserialize<'de> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::Arc(<Arc<T>>::deserialize(d)?))
    }
}

impl<T: ?Sized + Serialize> Serialize for StArc<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Static(t) => t.serialize(s),
            Self::Arc(t) => t.serialize(s),
        }
    }
}

impl<'a, T: ?Sized + PartialEq> PartialEq<&'a T> for StArc<T> {
    fn eq(&self, other: &&'a T) -> bool {
        **self == **other
    }
}

// impl<'a, T: ?Sized + PartialEq> PartialEq<StArc<T>> for &'a T {
//     fn eq(&self, other: &StArc<T>) -> bool {
//         other == self
//     }
// }

impl<T: ?Sized> Display for StArc<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StArc::Static(t) => t.fmt(f),
            StArc::Arc(t) => (&**t).fmt(f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SpellId {
    name: StArc<str>,
    level: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub enum StaticCustomSpell {
    Static(&'static Spell),
    Custom(CustomSpell),
}

macro_rules! delegate {
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

impl StaticCustomSpell {
    #[must_use]
    pub fn id(&self) -> SpellId {
        delegate!(self, id())
    }

    // todo level should really be an enum
    #[must_use]
    pub fn level(&self) -> usize {
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
        match self {
            Self::Static(spell) => spell.name.into(),
            Self::Custom(spell) => (&spell.name).into(),
        }
    }

    #[must_use]
    pub fn name_lower(&self) -> &str {
        delegate!(self, ref name_lower)
    }

    #[must_use]
    pub fn desc_lower(&self) -> &str {
        delegate!(self, ref desc_lower)
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
        match self {
            Self::Static(spell) => &spell.casting_time,
            Self::Custom(spell) => &spell.casting_time,
        }
    }

    #[must_use]
    pub fn source(&self) -> Source {
        match self {
            Self::Static(spell) => spell.source,
            Self::Custom(_) => Source::Custom,
        }
    }

    fn view<'a, B: SpellButtons<'a> + 'a>(&'a self, button: B, data: B::Data, collapse: bool, style: Style) -> Container<'a, Message> {
        match self {
            Self::Static(spell) => spell.view(button, data, collapse, style),
            Self::Custom(spell) => spell.view(button, data, collapse, style),
        }
    }
}

#[must_use]
pub fn find_spell(spell_name: &str, custom: &[CustomSpell]) -> Option<StaticCustomSpell> {
    // TODO remove this after its been enough time that everyone probably updated it
    fn fix_name_changes(spell_name: &str, spell: &Spell) -> bool {
        match spell_name {
            // Feb 21, 2022
            "Enemies abound" => spell.name == "Enemies Abound",
            _ => false
        }
    }

    SPELLS.iter()
        .find(|s| &*s.name == spell_name || fix_name_changes(spell_name, s))
        .map(StaticCustomSpell::Static)
        .or_else(|| custom.iter()
            .find(|s| &*s.name == spell_name)
            .cloned()
            .map(StaticCustomSpell::Custom))
}