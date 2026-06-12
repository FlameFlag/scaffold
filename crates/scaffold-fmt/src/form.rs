#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Form {
    Atom(String),
    String(String),
    Comment(String),
    Quote(char, Box<Form>),
    List {
        open: char,
        close: char,
        items: Vec<Form>,
    },
}

impl Form {
    pub(super) const fn is_block_comment(&self) -> bool {
        matches!(self, Self::Comment(_))
    }
}
