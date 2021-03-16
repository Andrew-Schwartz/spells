use iced::keyboard::{self, KeyCode};

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
}

pub fn handle(event: keyboard::Event) -> Option<crate::Message> {
    match event {
        keyboard::Event::KeyPressed { key_code, modifiers } => {
            let message = match (modifiers.control, modifiers.alt, modifiers.shift) {
                // ctrl
                (true, false, false) => match key_code {
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
                // alt
                (false, true, false) => match key_code {
                    KeyCode::Left => Some(Message::Move(Move::Left, false)),
                    KeyCode::Right => Some(Message::Move(Move::Right, false)),
                    _ => None,
                }
                // ctrl + shift
                (true, false, true) => match key_code {
                    KeyCode::Tab => Some(Message::Move(Move::Left, true)),
                    KeyCode::F | KeyCode::S => Some(Message::Find(true)),
                    _ => None,
                }
                // none
                (false, false, false) => match key_code {
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
                    _ => None,
                }
                _ => None
            };
            message.map(crate::Message::Hotkey)
        }
        _ => None
    }
}