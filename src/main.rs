#![feature(array_methods)]
#![feature(mixed_integer_ops)]
#![feature(const_option_ext)]

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
#![warn(elided_lifetimes_in_paths)]

use std::{fs::{self, File}, mem};
use std::cmp::min;
use std::convert::{From, Into};
use std::default::Default;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, ErrorKind, Write as _};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::{
    Alignment,
    alignment::Vertical,
    Command,
    Length,
    mouse::ScrollDelta,
    pure::{
        Application,
        button,
        column,
        container,
        Element,
        horizontal_rule,
        progress_bar,
        row,
        slider,
        text,
    },
    pure::widget::{
        Button,
        Container,
        Row,
    },
    Settings,
    tooltip::Position,
    window::Icon,
};
use iced_aw::{ICON_FONT, TabLabel};
use iced_native::{Event, Subscription, window};
use itertools::{Either, Itertools};
use once_cell::sync::Lazy;
use self_update::cargo_crate_version;
use serde::Deserialize;

use search::SearchPage;
use utils::ListGrammaticallyExt;

use crate::character::{Character, CharacterPage, SerializeCharacter};
use crate::hotkey::Move;
use crate::hotmouse::{ButtonPress, Pt};
use crate::settings::{ClosedCharacter, Edit, SettingsPage, SpellEditor};
use crate::spells::data::GetLevel;
use crate::spells::spell::{find_spell, SpellId};
use crate::style::{SettingsBarStyle, Style};
use crate::tabs::Tab;
use crate::utils::{SpacingExt, Tap, TooltipExt, TryRemoveExt};

use self::spells::data::{CastingTime, Class, Components, Level, School, Source};
use self::spells::spell::{CustomSpell, StaticSpell};
use self::spells::static_arc::StArc;

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
mod slots_widget;
mod spells;
mod error;

const JSON: &str = include_str!("../resources/spells.json");

pub static SPELLS: Lazy<Vec<StaticSpell>> = Lazy::new(|| serde_json::from_str(JSON).expect("json error in `data/spells.json`"));

static SAVE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = dirs::data_local_dir().unwrap_or_default()
        .join("dndspells");
    fs::create_dir_all(&path).unwrap();
    path
});
static CHARACTER_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = SAVE_DIR.clone();
    path.push("characters.json");
    fs::OpenOptions::new().create(true).append(true).open(&path).unwrap();
    path
});
static CLOSED_CHARACTER_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = SAVE_DIR.clone();
    path.push("closed-characters.json");
    fs::OpenOptions::new().create(true).append(true).open(&path).unwrap();
    path
});
static SPELL_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = SAVE_DIR.clone();
    path.push("custom-spells.json");
    fs::OpenOptions::new().create(true).append(true).open(&path).unwrap();
    path
});

// static ICON: Lazy<Icon> = Lazy::new(|| );
fn icon() -> Icon {
    const LOGO: &[u8] = include_bytes!("../resources/logo.png");
    const WIDTH: u32 = 1500;
    const HEIGHT: u32 = 1500;
    let image = image::load_from_memory(LOGO).expect("failed to read logo");

    Icon::from_rgba(image.into_bytes(), WIDTH, HEIGHT).unwrap()
}

const WIDTH: u32 = 1100;

// const CONSOLAS: Font = Font::External {
//     name: "consolas",
//     bytes: include_bytes!("../resources/consola.ttf"),
// };

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
    #[must_use]
    pub fn view<'s, 'c: 's>(&'s self, style: SettingsBarStyle) -> Container<'c, Message> {
        const VER: &str = cargo_crate_version!();
        match self {
            &Self::Downloading(pct) => {
                container(row()
                    .align_items(Alignment::Center)
                    .push(text("Downloading").size(10))
                    .push_space(5)
                    .push(progress_bar(0.0..=100.0, pct)
                        .style(style)
                        .height(Length::Units(12)) // bottom bar is 20 pts
                        .width(Length::Units(100)))
                )
            }
            view_as_text => match view_as_text {
                Self::Checking => text("Checking for updates..."),
                Self::Ready => text("Preparing to download..."),
                Self::Downloaded => text("Downloaded new version! Restart program to get new features!"),
                Self::UpToDate => text(format!("Spells v{}", VER)),
                Self::Errored(e) => text(format!("Error downloading new version: {}. Running v{}", e, VER)),
                Self::Downloading(_) => unreachable!(),
            }.size(10).tap(container)
        }.style(style)
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

    fn read_characters<C: From<Character>>(file: &Path, custom: &[CustomSpell]) -> error::Result<Vec<C>> {
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

    fn read_spells(file: &Path) -> error::Result<Vec<CustomSpell>> {
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

    fn open() -> error::Result<Self> {
        let custom_spells = Self::read_spells(&SPELL_FILE)?;
        let characters = Self::read_characters(&CHARACTER_FILE, &custom_spells)?;
        let closed_characters = Self::read_characters(&CLOSED_CHARACTER_FILE, &custom_spells)?;
        let (width, height) = iced::window::Settings::default().size;
        let mut window = Self {
            update_state: UpdateState::Checking,
            // update_status: format!("Running {}", cargo_crate_version!()),
            update_url: "".to_string(),
            style: Style::default(),
            tab: Tab::Search,
            width,
            height,
            control_pressed: false,
            search_page: SearchPage::new(&custom_spells, &characters),
            characters,
            closed_characters,
            settings_page: SettingsPage::new(&custom_spells),
            col_scale: 1.0,
            save_states: Default::default(),
            state: None,
            custom_spells,
            num_cols: 2,
            mouse: Default::default(),
        };
        window.save_state();
        Ok(window)
    }

    fn save(&mut self) -> error::Result<()> {
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
    type Executor = iced_futures::backend::default::Executor;
    type Message = Message;
    type Flags = ();

    fn new((): Self::Flags) -> (Self, Command<Message>) {
        let window = Self::open().expect("failed to start");
        // let commands = Command::batch([
        //     async { Message::Search(search::Message::Refresh) }.into(),
        //     async {
        //         // wait briefly to so that loading doesn't take so long
        //         tokio::time::sleep(Duration::from_millis(500)).await;
        //         Message::Update(update::Message::CheckForUpdate)
        //     }.into(),
        // ]);
        let commands = Command::perform(
            tokio::time::sleep(Duration::from_millis(500)),
            |()| Message::Update(update::Message::CheckForUpdate),
        );
        (window, commands)
    }

    fn title(&self) -> String {
        const SPELLS: &str = "D&D Spells";
        match self.tab {
            Tab::Search | Tab::Settings => SPELLS.into(),
            Tab::Character { index } => format!(
                "{SPELLS} - {}",
                self.characters.get(index)
                    .map_or("Character", |c| &c.character.name)
            )
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Update(msg) => if let Err(e) = update::handle(self, msg) {
                self.update_state = UpdateState::Errored(e.to_string());
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
                        self.settings_page.name = name;
                    }
                    Message::SubmitCharacter => {
                        // todo focus
                        // self.settings_page.character_name_state.focus();
                        let name = &mut self.settings_page.name;
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
                        self.save().expect("todoooooo");
                    }
                    Message::Rename(index) => {
                        let rename = match &mut self.closed_characters[index].rename {
                            Either::Left(_) => {
                                Either::Right(Default::default())
                            }
                            Either::Right(name) => {
                                if !name.is_empty() {
                                    let name = mem::take(name);
                                    self.closed_characters[index].character.name = Arc::from(name);
                                    self.save().expect("ASDSADAS");
                                }
                                Either::Left(())
                            }
                        };
                        self.closed_characters[index].rename = rename;
                    }
                    Message::RenameString(index, new) => {
                        if let Either::Right(name) = &mut self.closed_characters[index].rename {
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
                            .cloned()
                            .map(Box::new) {
                            self.settings_page.spell_editor = SpellEditor::Editing { spell }
                        } else {
                            self.settings_page.spell_editor = SpellEditor::searching(&name, &self.custom_spells);
                        }
                    }
                    Message::SubmitSpell => {
                        let name = mem::take(&mut self.settings_page.spell_name);
                        let spell = CustomSpell::new(name);
                        self.custom_spells.push(spell.clone());
                        self.settings_page.spell_editor = SpellEditor::Editing { spell: Box::new(spell) };
                        self.save().unwrap();
                    }
                    Message::OpenSpell(index) => {
                        if let SpellEditor::Searching { spells } = &mut self.settings_page.spell_editor {
                            if let Some(spell) = spells.try_remove(index).map(Box::new) {
                                self.settings_page.spell_editor = SpellEditor::Editing { spell };
                            }
                        }
                    }
                    Message::DeleteSpell(index) => {
                        if let SpellEditor::Searching { spells } = &mut self.settings_page.spell_editor {
                            let spell = spells.remove(index);
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
                                Edit::Range(range) => spell.range = (!range.is_empty()).then_some(range),
                                Edit::ComponentV(v) => match &mut spell.components {
                                    Some(components) => components.v = v,
                                    none => *none = Some(Components { v: true, s: false, m: None }),
                                },
                                Edit::ComponentS(s) => match &mut spell.components {
                                    Some(components) => components.s = s,
                                    none => *none = Some(Components { v: false, s: true, m: None }),
                                },
                                Edit::ComponentM(m) => match &mut spell.components {
                                    Some(components) => components.m = m.then(String::new),
                                    none => *none = Some(Components { v: false, s: false, m: Some(String::new()) }),
                                },
                                Edit::ComponentMaterial(mat) => match &mut spell.components {
                                    Some(components) => components.m = Some(mat),
                                    None => spell.components = Some(Components { v: false, s: false, m: Some(mat) }),
                                },
                                Edit::Duration(duration) => spell.duration = (!duration.is_empty()).then_some(duration),
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
                                saved_spell.clone_from(spell);
                                // *saved_spell = *spell.clone();
                            } else {
                                self.custom_spells.push(*spell.clone());
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
                    // todo
                    // self.search_page.search.state.focus();
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
                    idx.saturating_sub(delta.unsigned_abs())
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
                                    search::Message::Search(String::new()),
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
                                    page.tab = None;
                                    // todo
                                    // page.search.state.focus();
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
                            let tabs_by_character = self.characters.iter()
                                .map(|page| page.character.spells.iter()
                                    .filter(|spells| !spells.is_empty())
                                    .count() + 1)
                                .collect_vec();
                            // add 1 bc empty is [search, settings] which has max idx 1
                            let max_tab_idx = tabs_by_character.iter().sum::<usize>() + 1;
                            let orig_idx = match self.tab {
                                Tab::Search => 0,
                                Tab::Character { index } => {
                                    let character = &self.characters[index];
                                    1 // search
                                        + tabs_by_character[..index].iter().sum::<usize>() // previous characters
                                        + character.tab_index() // index in this character
                                }
                                Tab::Settings => max_tab_idx,
                            };
                            let idx = match dir {
                                Move::Left => {
                                    let idx = orig_idx.wrapping_sub(1);
                                    min(idx, max_tab_idx)
                                },
                                Move::Right => {
                                    let idx = orig_idx + 1;
                                    if idx > max_tab_idx { 0 } else { idx }
                                }
                            };
                            match idx {
                                0 => self.tab = Tab::Search,
                                settings_tab if settings_tab == max_tab_idx => self.tab = Tab::Settings,
                                mut tab => {
                                    // search tab
                                    tab -= 1;
                                    let mut character = 0;
                                    while tab >= tabs_by_character[character] {
                                        tab -= tabs_by_character[character];
                                        character += 1;
                                    }
                                    self.tab = Tab::Character { index: character };
                                    self.characters.get_mut(character).unwrap().tab = if tab == 0 {
                                        None
                                    } else {
                                        #[allow(clippy::cast_possible_truncation)]
                                        self.characters[character].character.spells.iter()
                                            .enumerate()
                                            .map(|(index, s)| (Level::from_u8(index as _).unwrap(), s))
                                            .filter(|(_, s)| !s.is_empty())
                                            .nth(tab - 1)
                                            .unwrap()
                                            .0
                                            .tap(Some)
                                    };
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
                    // todo
                    Message::CustomSpellNextField(_forwards) => {
                        // if let Tab::Settings = self.tab {
                        //     let mut states = vec![
                        //         &mut self.settings_page.character_name_state,
                        //         &mut self.settings_page.spell_name_state,
                        //     ];
                        //     if let SpellEditor::Editing { spell } = &mut self.settings_page.spell_editor {
                        //         states.extend([
                        //             &mut spell.casting_time_extra_state,
                        //             &mut spell.range_state
                        //         ]);
                        //         if matches!(&spell.components, Some(Components { m: Some(_), .. })) {
                        //             states.extend([&mut spell.material_state]);
                        //         }
                        //         // todo
                        //         states.extend([
                        //             // &mut spell.components_state,
                        //             // &mut spell.duration_state,
                        //             // &mut spell.description_state,
                        //             // &mut spell.higher_levels_state,
                        //             // &mut spell.source_state,
                        //             // &mut spell.page_state
                        //         ]);
                        //     }
                        //     for i in 0..states.len() {
                        //         if states[i].is_focused() {
                        //             if forwards && i != states.len() - 1 {
                        //                 states[i].unfocus();
                        //                 states[i + 1].focus();
                        //                 break;
                        //             } else if !forwards && i != 0 {
                        //                 states[i].unfocus();
                        //                 states[i - 1].focus();
                        //                 break;
                        //             }
                        //         }
                        //     }
                        // }
                    }
                    Message::CharacterSpellUpDown(delta) => {
                        if let Tab::Character { index } = self.tab {
                            if let Some(page) = self.characters.get_mut(index) {
                                // all tab
                                if page.tab.is_none() {
                                    if let Some(curr_view) = &mut page.view_spell {
                                        let spells = &page.character.spells;
                                        if let Some(pos) = spells[curr_view.level]
                                            .iter()
                                            .position(|(s, _)| s.name() == curr_view.name) {
                                            let first_spell = spells.iter()
                                                .flatten()
                                                .next()
                                                .expect("Not empty, since have `view_spell`")
                                                .0.id();
                                            let idx = if first_spell == *curr_view {
                                                pos.saturating_add_signed(delta)
                                            } else {
                                                pos.wrapping_add_signed(delta)
                                            };
                                            let new_view = spells[curr_view.level].get(idx)
                                                .or_else(|| {
                                                    curr_view.level.add_checked(delta)
                                                        .and_then(|level_added| spells.get_lvl(level_added))
                                                        .and_then(|other_level| match delta {
                                                            1 => other_level.first(),
                                                            -1 => other_level.last(),
                                                            _ => unreachable!(),
                                                        })
                                                })
                                                .map(|(s, _)| s.id());
                                            if let Some(new_view) = new_view {
                                                *curr_view = new_view;
                                            }
                                        }
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
                                return Command::perform(async move { pt }, Message::ScrollIGuessHopefully);
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
                                return self.update(message);
                            }
                        };
                        if self.mouse.press == button {
                            self.mouse.press = ButtonPress::None;
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

    fn view(&self) -> Element<'_, Self::Message> {
        let style = self.style;
        let num_cols = self.num_cols;
        let num_characters = self.characters.len();

        let height = self.height
            .saturating_sub(26)  // height of tab bar
            .saturating_sub(20); // height of bottom bar

        let tabs = iced_aw::pure::Tabs::new(self.tab.index(num_characters), Message::SelectTab)
            .push(TabLabel::Text("Search".into()), self.search_page.view(style).max_height(height));
        let tabs = self.characters.iter()
            .enumerate()
            .map(|(index, page)| (
                TabLabel::Text(page.character.name.to_string()),
                page.view(index, num_cols, style).max_height(height)
            )).fold(
            tabs,
            |tabs, (label, tab)| tabs.push(label, tab),
        ).push(TabLabel::Text("Settings".into()), self.settings_page.view(&self.closed_characters, self.width, style).max_height(height))
            .tab_bar_style(style)
            .icon_size(10)
            .icon_font(ICON_FONT)
            // .on_close(Message::CloseTab)
            ;

        #[allow(clippy::float_cmp)]
        let col_slider_reset: Button<'_, Message> = button(
            text("Reset")
                .vertical_alignment(Vertical::Center)
                .size(12),
        ).style(style.settings_bar())
            .tap_if(self.col_scale != 1.0, |reset| reset.on_press(Message::SetColScale(1.0)));

        // todo monospace font and pad with spaces
        let slider_text = text(
            format!("{:3.0}%", self.col_scale * 100.0)
        ).size(10)
            .vertical_alignment(Vertical::Center);

        let col_slider = slider(
            0.5..=4.0,
            self.col_scale,
            Message::SetColScale,
        )
            .width(Length::Units(120))
            .step(0.01)
            .style(style);

        let toggle_style = button(
            text(iced_aw::Icon::BrightnessHigh)
                .font(ICON_FONT)
                .size(12),
        ).style(style.settings_bar())
            .on_press(Message::ToggleTheme)
            .tooltip_at(&format!("Switch to {} theme", !style), Position::Top)
            .size(10);

        let bottom_bar = container(row()
            .spacing(2)
            .push_space(4)
            .push(self.update_state.view(style.settings_bar()))
            .push_space(Length::Fill)
            .push(col_slider_reset)
            .push(col_slider)
            .push(slider_text)
            .push(toggle_style)
            .height(Length::Units(20))
            .align_items(Alignment::Center)
        ).style(style.settings_bar())
            .align_y(Vertical::Center);

        let main_content = container(tabs)
            .height(Length::Fill)
            .width(Length::FillPortion(18));

        let content = column()
            .push(main_content)
            .push(bottom_bar);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .align_y(Vertical::Top)
            .style(style)
            .into()
    }
}

impl From<School> for String {
    fn from(school: School) -> Self {
        school.to_string()
    }
}

#[derive(Deserialize)]
struct DeserializeSpell {
    name: &'static str,
    level: Level,
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

pub trait SpellButtons {
    type Data;

    fn view<'c>(self, id: SpellId, data: Self::Data, style: Style) -> (Row<'c, Message>, Element<'c, Message>);
}