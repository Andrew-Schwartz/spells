use iced::{Align, button, Button, Checkbox, Column, Container, Element, Length, PickList, Row, Rule, Text, text_input, TextInput};
use itertools::Itertools;

use crate::{CastingTime, Class, CustomSpell, School};
use crate::character::Character;
use crate::search::PLOption;
use crate::settings::Message::SubmitSpell;
use crate::style::Style;
use crate::utils::{ListGrammaticallyExt, SpacingExt, Tap};

#[derive(Debug, Clone)]
pub enum Message {
    CharacterName(String),
    SubmitCharacter,
    Open(usize),
    DeleteCharacter(usize),
    SpellName(String),
    OpenSpell(usize),
    SubmitSpell,
    DeleteSpell(usize),
    EditSpell(Edit),
    CloseSpell,
}

#[derive(Debug, Clone)]
pub enum Edit {
    School(School),
    Level(usize),
    CastingTime(CastingTime),
    CastingTimeN(String),
    CastingTimeWhen(String),
    Range(String),
    Components(String),
    Duration(String),
    Ritual(bool),
    Concentration(bool),
    Description(String),
    // DescEnter,
    HigherLevels(String),
    Class(PLOption<Class>),
    // Source(String),
    // Page(String),
}

pub struct ClosedCharacter {
    pub character: Character,
    name_button: button::State,
    open_button: button::State,
    delete_button: button::State,
}

impl From<Character> for ClosedCharacter {
    fn from(character: Character) -> Self {
        Self {
            character,
            name_button: Default::default(),
            open_button: Default::default(),
            delete_button: Default::default(),
        }
    }
}

pub struct SettingsPage {
    pub name: String,
    pub character_name_state: text_input::State,
    create_character: button::State,
    pub spell_name: String,
    pub spell_name_state: text_input::State,
    create_spell: button::State,
    pub spell_editor: SpellEditor,
    close_spell_state: button::State,
}

impl SettingsPage {
    pub fn new(custom_spells: &[CustomSpell]) -> Self {
        Self {
            name: Default::default(),
            character_name_state: Default::default(),
            create_character: Default::default(),
            spell_name: Default::default(),
            spell_name_state: Default::default(),
            create_spell: Default::default(),
            spell_editor: SpellEditor::searching("", custom_spells),
            close_spell_state: Default::default(),
        }
    }
}

pub enum SpellEditor {
    Searching {
        /// Vec<(spell, open, delete)>
        spells: Vec<(CustomSpell, button::State, button::State, button::State)>,
    },
    Editing {
        spell: CustomSpell,
    },
}

impl SpellEditor {
    pub fn searching(needle: &str, spells: &[CustomSpell]) -> Self {
        let spells = spells.iter()
            .map(|spell| (&spell.name_lower, spell))
            .filter(|(name, _)| name.contains(&needle))
            .sorted_unstable_by_key(|&(name, _)| name)
            // .sorted_unstable_by_key(|(name, _)| levenshtein(name, needle))
            .map(|(_, spell)| spell)
            .take(20)
            .map(|spell| (spell.clone(), Default::default(), Default::default(), Default::default()))
            .collect();
        Self::Searching { spells }
    }
}

impl SettingsPage {
    pub fn view<'a>(
        &'a mut self,
        closed_characters: &'a mut [ClosedCharacter],
        width: u32,
        style: Style,
    ) -> Container<'a, crate::Message> {
        const PADDING: u16 = 12;
        const RULE_SPACING: u16 = 24;
        const NAME_PADDING: u16 = 3;
        const SPACING: u16 = 5;

        let character_label = Row::new()
            .push_space(Length::Fill)
            .push(Text::new("Characters").size(30))
            .push_space(Length::Fill);

        let character_name_input = TextInput::new(
            &mut self.character_name_state,
            "Character Name",
            &self.name,
            |n| crate::Message::Settings(Message::CharacterName(n)),
        ).style(style)
            .on_submit(crate::Message::Settings(Message::SubmitCharacter));
        let create_character_button = Button::new(
            &mut self.create_character,
            Text::new("Create").size(16),
        ).style(style)
            .on_press(crate::Message::Settings(Message::SubmitCharacter));
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_lossless)]
        let text_width = (width as f32 / 2.0
            - PADDING as f32
            - RULE_SPACING as f32
            - NAME_PADDING as f32
            - 45.0 // open button
            - SPACING as f32
            - 51.0 // delete button
        ) as u32;
        let closed_character_buttons = closed_characters.iter_mut()
            .enumerate()
            .fold(Column::new(), |col, (index, closed)| {
                let style = style.alternating(index);
                let name = Button::new(
                    &mut closed.name_button,
                    Text::new(&*closed.character.name).size(19),
                ).style(style.no_highlight())
                    .on_press(crate::Message::Settings(Message::Open(index)));
                let name = Container::new(name)
                    .max_width(text_width)
                    .style(style);
                let open = Button::new(
                    &mut closed.open_button,
                    Text::new("Open").size(15),
                ).style(style)
                    .on_press(crate::Message::Settings(Message::Open(index)));
                let delete = Button::new(
                    &mut closed.delete_button,
                    Text::new("Delete").size(15),
                ).style(style)
                    .on_press(crate::Message::Settings(Message::DeleteCharacter(index)));
                col.push(Container::new(
                    Row::new()
                        .spacing(SPACING)
                        .push_space(NAME_PADDING)
                        .push(name)
                        .push_space(Length::Fill)
                        .push(open)
                        .push(delete)
                        .align_items(Align::Center)
                ).style(style))
            });

        let character_col = Column::new()
            .spacing(4)
            .push(Row::new()
                .align_items(Align::Center)
                .push(character_name_input)
                .push_space(4)
                .push(create_character_button))
            .push_space(14)
            .push(closed_character_buttons)
            ;

        let spells_label = Row::new()
            .push_space(Length::Fill)
            .push(Text::new("Spell Editor").size(30))
            .push_space(Length::Fill);

        let spell_name = TextInput::new(
            &mut self.spell_name_state,
            "Spell Name",
            &self.spell_name,
            |n| crate::Message::Settings(Message::SpellName(n)),
        ).style(style)
            .on_submit(crate::Message::Settings(Message::SubmitSpell));
        let create_spell_button = Button::new(
            &mut self.create_spell,
            Text::new("Create").size(16),
        ).style(style)
            .on_press(crate::Message::Settings(SubmitSpell));

        let spells_col = Column::new()
            .spacing(4)
            .push(Row::new()
                .align_items(Align::Center)
                .push(spell_name)
                .push_space(4)
                .push(create_spell_button))
            .push_space(10);

        let spells_col = match &mut self.spell_editor {
            SpellEditor::Searching { spells } => {
                spells.iter_mut()
                    .enumerate()
                    .fold(spells_col, |spells_col, (index, (spell, edit1, edit2, delete))| {
                        let style = style.alternating(index);
                        let name = Button::new(
                            edit1,
                            Text::new(&*spell.name).size(19),
                        ).style(style.no_highlight())
                            .on_press(crate::Message::Settings(Message::OpenSpell(index)));
                        let edit = Button::new(
                            edit2,
                            Text::new("Edit").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::OpenSpell(index)));
                        let delete = Button::new(
                            delete,
                            Text::new("Delete").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::DeleteSpell(index)));
                        spells_col.push(Container::new(
                            Row::new()
                                .spacing(SPACING)
                                .push_space(NAME_PADDING)
                                .push(name)
                                .push_space(Length::Fill)
                                .push(edit)
                                .push(delete)
                                .align_items(Align::Center)
                        ).style(style))
                    })
            }
            SpellEditor::Editing { spell } => {
                fn row<'a, T: Into<Element<'a, crate::Message>>, L: Into<String>>(
                    label: L,
                    content: T,
                ) -> Row<'a, crate::Message> {
                    let label = label.into();
                    let labeled = !label.is_empty();
                    let mut row = Row::new()
                        .push(Text::new(label).size(16));
                    if labeled {
                        row = row.push_space(Length::Fill);
                        // row = row.push_space(Length::Units(16))
                    }
                    let row = row
                        .push(content)
                        .align_items(Align::Center);
                    Row::new()
                        .push_space(Length::Fill)
                        .push(
                            Container::new(row).width(Length::FillPortion(18))
                        )
                        .push_space(Length::Fill)
                }
                fn edit_message<T: 'static>(edit_ctor: fn(T) -> Edit) -> impl Fn(T) -> crate::Message {
                    move |t: T| crate::Message::Settings(Message::EditSpell(
                        edit_ctor(t)
                    ))
                }

                let title = Text::new(&*spell.name).size(36);
                let close_button = Button::new(
                    &mut self.close_spell_state,
                    Text::new("Close"),
                ).style(style)
                    .on_press(crate::Message::Settings(Message::CloseSpell));
                let title = Row::new()
                    .push_space(Length::Fill)
                    .push(title)
                    .push(Container::new(Row::new()
                        .push_space(Length::Fill)
                        .push(close_button)
                    ).width(Length::Fill))
                    .align_items(Align::Center);

                let school = PickList::new(
                    &mut spell.school_state,
                    &School::ALL[..],
                    Some(spell.school),
                    edit_message(Edit::School),
                ).style(style);

                let level = PickList::new(
                    &mut spell.level_state,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..],
                    Some(spell.level),
                    edit_message(Edit::Level),
                ).style(style).text_size(14);

                // let casting_time = TextInput::new(
                //     &mut spell.casting_time_state,
                //     "",
                //     &spell.casting_time.to_string(),
                //     edit_message(Edit::CastingTime),
                // ).style(style)
                //     .on_submit(crate::Message::Settings(Message::EditSpell(Edit::CastingTimeSubmit)));

                const CASTING_TIMES: &'static [CastingTime] = &CastingTime::ALL;
                let casting_time = PickList::new(
                    &mut spell.casting_time_state,
                    CASTING_TIMES,
                    Some(match &spell.casting_time {
                        CastingTime::Reaction(_) => CastingTime::Reaction(None),
                        other => other.clone(),
                    }),
                    edit_message(Edit::CastingTime),
                ).style(style);

                let casting_time_extra = match &spell.casting_time {
                    CastingTime::Special | CastingTime::Action | CastingTime::BonusAction => None,
                    CastingTime::Reaction(when) => Some(row(
                        "Which you take when:",
                        TextInput::new(
                            &mut spell.casting_time_extra_state,
                            "",
                            when.as_deref().unwrap_or(""),
                            edit_message(Edit::CastingTimeWhen),
                        ).style(style),
                    )),
                    &(CastingTime::Minute(n) | CastingTime::Hour(n)) => Some(row(
                        if matches!(&spell.casting_time, CastingTime::Minute(_)) { "Minutes:" } else { "Hours:" },
                        TextInput::new(
                            &mut spell.casting_time_extra_state,
                            "",
                            &n.to_string(),
                            edit_message(Edit::CastingTimeN),
                        ).style(style),
                    )),
                };

                let range = TextInput::new(
                    &mut spell.range_state,
                    "",
                    &spell.range,
                    edit_message(Edit::Range),
                ).style(style);

                let components = TextInput::new(
                    &mut spell.components_state,
                    "",
                    &spell.components,
                    edit_message(Edit::Components),
                ).style(style);

                let duration = TextInput::new(
                    &mut spell.duration_state,
                    "",
                    &spell.duration,
                    edit_message(Edit::Duration),
                ).style(style);

                let ritual = Checkbox::new(
                    spell.ritual,
                    "",
                    edit_message(Edit::Ritual),
                ).style(style);

                let conc = Checkbox::new(
                    spell.conc,
                    "",
                    edit_message(Edit::Concentration),
                ).style(style);

                let description = TextInput::new(
                    &mut spell.description_state,
                    "Describe the spell's effects...",
                    &spell.description,
                    edit_message(Edit::Description),
                ).style(style)
                    // .on_submit(crate::Message::Settings(Message::EditSpell(Edit::DescEnter)))
                    ;

                let higher_levels = TextInput::new(
                    &mut spell.higher_levels_state,
                    "Higher level effects...",
                    spell.higher_levels.as_deref().unwrap_or(""),
                    edit_message(Edit::HigherLevels),
                ).style(style);

                let classes = PickList::new(
                    &mut spell.classes_state,
                    &Class::PL_ALL[..],
                    Some(PLOption::None),
                    edit_message(Edit::Class),
                ).style(style);

                // let page = TextInput::new(
                //     &mut spell.page_state,
                //     "278",
                //     &spell.page.map_or_else(String::new, |p| p.to_string()),
                //     edit_message(Edit::Page),
                // ).style(style);

                let column = Column::new()
                    .spacing(3)
                    .push(row("", title))
                    .push(Rule::horizontal(8))
                    .push(row("", school))
                    .push_space(2)
                    .push(row("Level:", level))
                    .push(row("Casting Time:", casting_time))
                    .tap(|col| match casting_time_extra {
                        Some(casting_time_extra) => col.push(casting_time_extra),
                        None => col,
                    }).push(row("Range:", range))
                    .push(row("Components:", components))
                    .push(row("Duration:", duration))
                    .push(row("Ritual?", ritual))
                    .push(row("Concentration?", conc))
                    .push(Rule::horizontal(8))
                    .push(row("", description))
                    .push(Rule::horizontal(8))
                    .push(row("At Higher Levels:", higher_levels))
                    .push(Rule::horizontal(8))
                    .push(row("Classes:", classes))
                    .push(row("", Text::new(spell.classes.iter().list_grammatically()).size(16)))
                    // .push(Rule::horizontal(8))
                    // .push(row("Source:", source))
                    // .push(row("Page:", page))
                    ;
                spells_col.push(column)
            }
        };

        let row = Row::new()
            .padding(PADDING)
            .push(Column::new()
                .width(Length::Fill)
                .push(character_label.height(Length::Fill))
                .push_space(1)
                .push(character_col.height(Length::FillPortion(18)))
            )
            .push(Rule::vertical(RULE_SPACING))
            .push(Column::new()
                .width(Length::Fill)
                .push(spells_label.height(Length::Fill))
                .push_space(1)
                .push(spells_col.height(Length::FillPortion(18)))
            );

        Container::new(row.height(Length::Shrink))
    }
}