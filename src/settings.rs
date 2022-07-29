use iced::{Alignment, Length, pure::{*, widget::*}};
use itertools::{Either, Itertools};

use crate::{CastingTime, Class, Components, CustomSpell, School};
use crate::character::Character;
use crate::search::PLOption;
use crate::style::Style;
use crate::utils::{ListGrammaticallyExt, SpacingExt, Tap};

#[derive(Debug, Clone)]
pub enum Message {
    CharacterName(String),
    SubmitCharacter,
    Open(usize),
    Rename(usize),
    RenameString(usize, String),
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
    ComponentV(bool),
    ComponentS(bool),
    ComponentM(bool),
    ComponentMaterial(String),
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
    pub rename: Either<(), String>,
}

impl From<Character> for ClosedCharacter {
    fn from(character: Character) -> Self {
        Self {
            character,
            rename: Either::Left(Default::default()),
        }
    }
}

pub struct SettingsPage {
    pub name: String,
    pub spell_name: String,
    pub spell_editor: SpellEditor,
}

impl SettingsPage {
    pub fn new(custom_spells: &[CustomSpell]) -> Self {
        Self {
            name: Default::default(),
            spell_name: Default::default(),
            spell_editor: SpellEditor::searching("", custom_spells),
        }
    }
}

pub enum SpellEditor {
    Searching {
        /// Vec<(spell, open, delete)>
        spells: Vec<CustomSpell>,
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
            .cloned()
            .collect();
        Self::Searching { spells }
    }
}

impl SettingsPage {
    pub fn view<'s, 'c: 's>(
        &'s self,
        closed_characters: &[ClosedCharacter],
        width: u32,
        style: Style,
    ) -> Container<'c, crate::Message> {
        const PADDING: u16 = 12;
        const RULE_SPACING: u16 = 24;
        const NAME_PADDING: u16 = 3;
        const SPACING: u16 = 5;

        let character_label = row()
            .push_space(Length::Fill)
            .push(text("Characters").size(30))
            .push_space(Length::Fill);

        let character_name_input = text_input(
            "Character Name",
            &self.name,
            |n| crate::Message::Settings(Message::CharacterName(n)),
        ).style(style)
            .on_submit(crate::Message::Settings(Message::SubmitCharacter));
        let create_character_button = button(
            text("Create").size(16),
        ).style(style)
            .on_press(crate::Message::Settings(Message::SubmitCharacter));
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_lossless)]
            let text_width = (width as f32 / 2.0
            - PADDING as f32
            - RULE_SPACING as f32
            - NAME_PADDING as f32
            - 45.0 // open button
            - (2 * SPACING) as f32
            - 51.0 // delete button
        ) as u32;
        let closed_character_buttons = closed_characters.iter()
            .enumerate()
            .fold(column(), |col, (index, closed)| {
                let style = style.alternating(index);
                let name = button(
                    text(&*closed.character.name).size(19),
                ).style(style.no_highlight())
                    .on_press(crate::Message::Settings(Message::Open(index)));
                let name = container(name)
                    .max_width(text_width)
                    .style(style);
                let open = button(
                    text("Open").size(15),
                ).style(style)
                    .on_press(crate::Message::Settings(Message::Open(index)));
                let rename = match &closed.rename {
                    Either::Left(()) => {
                        let button = button(
                            text("Rename").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::Rename(index)));
                        container(button).style(style)
                    }
                    Either::Right(name) => {
                        let cancel_input = text_input(
                            "Submit now to cancel",
                            &*name,
                            move |s| crate::Message::Settings(Message::RenameString(index, s)),
                        ).style(style)
                            .width(Length::Units(140))
                            .on_submit(crate::Message::Settings(Message::Rename(index)));
                        let button = button(
                            text("Submit").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::Rename(index)));
                        let row = row()
                            .align_items(Alignment::Center)
                            .push(cancel_input)
                            .push_space(3)
                            .push(button);
                        container(row).style(style)
                    }
                };
                let delete = button(
                    text("Delete").size(15),
                ).style(style)
                    .on_press(crate::Message::Settings(Message::DeleteCharacter(index)));
                col.push(container(
                    row()
                        .spacing(SPACING)
                        .push_space(NAME_PADDING)
                        .push(name)
                        .push_space(Length::Fill)
                        .push(open)
                        .push(rename)
                        .push(delete)
                        .align_items(Alignment::Center)
                ).style(style))
            });

        let character_col = column()
            .spacing(4)
            .push(row()
                .align_items(Alignment::Center)
                .push(character_name_input)
                .push_space(4)
                .push(create_character_button))
            .push_space(14)
            .push(closed_character_buttons)
            ;

        let spells_label = row()
            .push_space(Length::Fill)
            .push(text("Spell Editor").size(30))
            .push_space(Length::Fill);

        let spell_name = text_input(
            "Spell Name",
            &self.spell_name,
            |n| crate::Message::Settings(Message::SpellName(n)),
        ).style(style)
            .on_submit(crate::Message::Settings(Message::SubmitSpell));
        let create_spell_button = button(
            text("Create").size(16),
        ).style(style)
            .on_press(crate::Message::Settings(Message::SubmitSpell));

        let spells_col = column()
            .spacing(4)
            .push(row()
                .align_items(Alignment::Center)
                .push(spell_name)
                .push_space(4)
                .push(create_spell_button))
            .push_space(10);

        let spells_col = match &self.spell_editor {
            SpellEditor::Searching { spells } => {
                let col = spells.iter()
                    .enumerate()
                    .fold(column().spacing(4), |spells_col, (index, spell)| {
                        let style = style.alternating(index);
                        let name = button(
                            text(&*spell.name).size(19),
                        ).style(style.no_highlight())
                            .on_press(crate::Message::Settings(Message::OpenSpell(index)));
                        let edit = button(
                            text("Edit").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::OpenSpell(index)));
                        let delete = button(
                            text("Delete").size(15),
                        ).style(style)
                            .on_press(crate::Message::Settings(Message::DeleteSpell(index)));
                        spells_col.push(container(
                            row()
                                .spacing(SPACING)
                                .push_space(NAME_PADDING)
                                .push(name)
                                .push_space(Length::Fill)
                                .push(edit)
                                .push(delete)
                                .align_items(Alignment::Center)
                        ).style(style))
                    });
                spells_col.push(scrollable(col))
            }
            SpellEditor::Editing { spell } => {
                fn make_row<'a, T: Into<Element<'a, crate::Message>>, L: Into<String>>(
                    label: L,
                    content: T,
                ) -> Row<'a, crate::Message> {
                    let label = label.into();
                    let labeled = !label.is_empty();
                    let mut ret = row()
                        .push(text(label).size(16));
                    if labeled {
                        ret = ret.push_space(Length::Fill);
                        // row = row.push_space(Length::Units(16))
                    }
                    let ret = ret
                        .push(content)
                        .align_items(Alignment::Center);
                    row()
                        .push_space(Length::Fill)
                        .push(
                            container(ret).width(Length::FillPortion(18))
                        )
                        .push_space(Length::Fill)
                }
                fn edit_message<T: 'static>(edit_ctor: fn(T) -> Edit) -> impl Fn(T) -> crate::Message {
                    move |t: T| crate::Message::Settings(Message::EditSpell(edit_ctor(t)))
                }

                let title = text(&*spell.name).size(36);
                let close_button = button(
                    "Close",
                ).style(style)
                    .on_press(crate::Message::Settings(Message::CloseSpell));
                let title = row()
                    .push_space(Length::Fill)
                    .push(title)
                    .push(container(row()
                        .push_space(Length::Fill)
                        .push(close_button)
                    ).width(Length::Fill))
                    .align_items(Alignment::Center);

                let school = pick_list(
                    &School::ALL[..],
                    Some(spell.school),
                    edit_message(Edit::School),
                ).style(style);

                let level = pick_list(
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..],
                    Some(spell.level),
                    edit_message(Edit::Level),
                ).style(style).text_size(14);

                const CASTING_TIMES: &'static [CastingTime] = &CastingTime::ALL;
                let casting_time = pick_list(
                    CASTING_TIMES,
                    Some(match &spell.casting_time {
                        CastingTime::Reaction(_) => CastingTime::Reaction(None),
                        other => other.clone(),
                    }),
                    edit_message(Edit::CastingTime),
                ).style(style);

                let casting_time_extra = match &spell.casting_time {
                    CastingTime::Special | CastingTime::Action | CastingTime::BonusAction => None,
                    CastingTime::Reaction(when) => Some(make_row(
                        "Which you take when:",
                        text_input(
                            "",
                            when.as_deref().unwrap_or(""),
                            edit_message(Edit::CastingTimeWhen),
                        ).style(style),
                    )),
                    &(CastingTime::Minute(n) | CastingTime::Hour(n)) => Some(make_row(
                        if matches!(&spell.casting_time, CastingTime::Minute(_)) { "Minutes:" } else { "Hours:" },
                        text_input(
                            "",
                            &n.to_string(),
                            edit_message(Edit::CastingTimeN),
                        ).style(style),
                    )),
                };

                let range = text_input(
                    "",
                    &spell.range,
                    edit_message(Edit::Range),
                ).style(style);

                let Components { v, s, m } = spell.components.clone().unwrap_or_default();
                let v = checkbox(
                    "V",
                    v,
                    edit_message(Edit::ComponentV),
                ).style(style);
                let s = checkbox(
                    "S",
                    s,
                    edit_message(Edit::ComponentS),
                ).style(style);
                let mat = checkbox(
                    "M",
                    m.is_some(),
                    edit_message(Edit::ComponentM),
                ).style(style);
                let components = row()
                    .push_space(Length::Fill)
                    .push(v)
                    .push_space(Length::Fill)
                    .push(s)
                    .push_space(Length::Fill)
                    .push(mat);
                let material_component = if let Some(mat) = m {
                    Some(text_input(
                        "material",
                        &mat,
                        edit_message(Edit::ComponentMaterial),
                    ).style(style))
                } else {
                    None
                };

                let duration = text_input(
                    "",
                    &spell.duration,
                    edit_message(Edit::Duration),
                ).style(style);

                let ritual = checkbox(
                    "",
                    spell.ritual,
                    edit_message(Edit::Ritual),
                ).style(style);

                let conc = checkbox(
                    "",
                    spell.conc,
                    edit_message(Edit::Concentration),
                ).style(style);

                let description = text_input(
                    "Describe the spell's effects...",
                    &spell.description,
                    edit_message(Edit::Description),
                ).style(style)
                    // .on_submit(crate::Message::Settings(Message::EditSpell(Edit::DescEnter)))
                    ;

                let higher_levels = text_input(
                    "Higher level effects...",
                    spell.higher_levels.as_deref().unwrap_or(""),
                    edit_message(Edit::HigherLevels),
                ).style(style);

                let classes = pick_list(
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

                let column = column()
                    .spacing(3)
                    .push(make_row("", title))
                    .push(horizontal_rule(8))
                    .push(make_row("", school))
                    .push_space(2)
                    .push(make_row("Level:", level))
                    .push(make_row("Casting Time:", casting_time))
                    .tap_if_some(casting_time_extra, |col, cte| col.push(cte))
                    .push(make_row("Range:", range))
                    .push(make_row("Components:", components))
                    .tap_if_some(material_component, |col, mat| col.push(make_row("Material:", mat)))
                    .push(make_row("Duration:", duration))
                    .push(make_row("Ritual?", ritual))
                    .push(make_row("Concentration?", conc))
                    .push(horizontal_rule(8))
                    .push(make_row("", description))
                    .push(horizontal_rule(8))
                    .push(make_row("At Higher Levels:", higher_levels))
                    .push(horizontal_rule(8))
                    .push(make_row("Classes:", classes))
                    .push(make_row("", text(spell.classes.iter().list_grammatically()).size(16)))
                    // .push(Rule::horizontal(8))
                    // .push(row("Source:", source))
                    // .push(row("Page:", page))
                    ;
                spells_col.push(column)
            }
        };

        let row = row()
            .padding(PADDING)
            .push(column()
                .width(Length::Fill)
                .push(character_label.height(Length::Fill))
                .push_space(1)
                .push(character_col.height(Length::FillPortion(18)))
            )
            .push(vertical_rule(RULE_SPACING))
            .push(column()
                .width(Length::Fill)
                .push(spells_label.height(Length::Fill))
                .push_space(1)
                .push(spells_col.height(Length::FillPortion(18)))
            );

        container(row.height(Length::Shrink))
    }
}