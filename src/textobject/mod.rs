#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum TextObject {
    #[default]
    Paren, // ()
    CurlyBracket, // {}
    Word,
    End, // e
         // Paren, // ()
         // Brack, // <>
}

impl TextObject {
    pub fn open_symbol(&self) -> Option<char> {
        match self {
            TextObject::Paren => Some('('),
            TextObject::CurlyBracket => Some('{'),
            _ => None,
        }
    }
    pub fn close_symbol(&self) -> Option<char> {
        match self {
            TextObject::Paren => Some(')'),
            TextObject::CurlyBracket => Some('}'),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Boundary {
    #[default]
    Current, // From current location. I would like to use 'None', but that would be bad.
    Inner,
    // includes whitespace . Prefers whitespace ahead. If not possible (eg due to different
    // word type) then uses the other side.
    // Not really clear to me the logic of how this is in vim...
    Around,
}
