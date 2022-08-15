#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tab {
    Search,
    Character { index: usize },
    Settings,
}

impl Tab {
    pub fn index(self, num_characters: usize) -> usize {
        match self {
            Tab::Search => 0,
            Tab::Character { index } => index + 1,
            Tab::Settings => num_characters + 1,
        }
    }
}