use iced::keyboard::{self, KeyCode, Modifiers};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Move {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    ToCharacter(usize),
    ///  force
    Find(bool),
    NewCharacter,
    ///  move, tab_only
    Move(Move, bool),
    Undo,
    Redo,
    CharacterTab(usize),
    AddSpell(usize),
    /// true -> forwards, false -> backwards
    CustomSpellNextField(bool),
}

pub fn handle(event: keyboard::Event) -> Option<crate::Message> {
    const CTRL_ALT: Modifiers = Modifiers::CTRL.union(Modifiers::ALT);
    const CTRL_SHIFT: Modifiers = Modifiers::CTRL.union(Modifiers::SHIFT);
    const NONE: Modifiers = Modifiers::empty();

    match event {
        keyboard::Event::KeyPressed { key_code, modifiers } => {
            let message = match modifiers {
                #[allow(clippy::match_same_arms)]
                Modifiers::CTRL => match key_code {
                    KeyCode::Grave => Some(Message::Find(true)),
                    KeyCode::Key1 => Some(Message::ToCharacter(1)),
                    KeyCode::Key2 => Some(Message::ToCharacter(2)),
                    KeyCode::Key3 => Some(Message::ToCharacter(3)),
                    KeyCode::Key4 => Some(Message::ToCharacter(4)),
                    KeyCode::Key5 => Some(Message::ToCharacter(5)),
                    KeyCode::Key6 => Some(Message::ToCharacter(6)),
                    KeyCode::Key7 => Some(Message::ToCharacter(7)),
                    KeyCode::Key8 => Some(Message::ToCharacter(8)),
                    KeyCode::Key9 => Some(Message::ToCharacter(9)),
                    KeyCode::Key0 => Some(Message::ToCharacter(0)),
                    KeyCode::F => Some(Message::Find(false)),
                    KeyCode::S => Some(Message::Find(true)),
                    KeyCode::Tab => Some(Message::Move(Move::Right, true)),
                    KeyCode::Insert | KeyCode::N => Some(Message::NewCharacter),
                    KeyCode::Z => Some(Message::Undo),
                    KeyCode::Y => Some(Message::Redo),
                    _ => None,
                }
                CTRL_ALT => match key_code {
                    KeyCode::Key1 => Some(Message::AddSpell(0)),
                    KeyCode::Key2 => Some(Message::AddSpell(1)),
                    KeyCode::Key3 => Some(Message::AddSpell(2)),
                    KeyCode::Key4 => Some(Message::AddSpell(3)),
                    KeyCode::Key5 => Some(Message::AddSpell(4)),
                    KeyCode::Key6 => Some(Message::AddSpell(5)),
                    KeyCode::Key7 => Some(Message::AddSpell(6)),
                    KeyCode::Left => Some(Message::Move(Move::Left, false)),
                    KeyCode::Right => Some(Message::Move(Move::Right, false)),
                    _ => None,
                }
                CTRL_SHIFT => match key_code {
                    KeyCode::Tab => Some(Message::Move(Move::Left, true)),
                    KeyCode::F | KeyCode::S => Some(Message::Find(true)),
                    _ => None,
                }
                Modifiers::SHIFT => match key_code {
                    KeyCode::Tab | KeyCode::Enter | KeyCode::NumpadEnter => Some(Message::CustomSpellNextField(false)),
                    _ => None,
                }
                NONE => match key_code {
                    KeyCode::Grave | KeyCode::Key0 => Some(Message::CharacterTab(1)),
                    KeyCode::Key1 => Some(Message::CharacterTab(2)),
                    KeyCode::Key2 => Some(Message::CharacterTab(3)),
                    KeyCode::Key3 => Some(Message::CharacterTab(4)),
                    KeyCode::Key4 => Some(Message::CharacterTab(5)),
                    KeyCode::Key5 => Some(Message::CharacterTab(6)),
                    KeyCode::Key6 => Some(Message::CharacterTab(7)),
                    KeyCode::Key7 => Some(Message::CharacterTab(8)),
                    KeyCode::Key8 => Some(Message::CharacterTab(9)),
                    KeyCode::Key9 => Some(Message::CharacterTab(10)),
                    KeyCode::A => Some(Message::CharacterTab(0)),
                    KeyCode::Tab | KeyCode::Enter | KeyCode::NumpadEnter => Some(Message::CustomSpellNextField(true)),
                    _ => None,
                }
                _ => None
            };
            message.map(crate::Message::Hotkey)
        }
        _ => None
    }
}