#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum TextObject {
    #[default]
    Paren, // ()
    Block,
    Word,
    End, // e
         // Paren, // ()
         // Brack, // <>
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum Boundary {
    #[default]
    Current, // From current location. I would like to use 'None', but that would be bad.
    Inner,
    // includes whitespace . Prefers whitespace ahead. If not possible (eg due to different
    // word type) then uses the other side.
    // Not really clear to me the logic of how this is in vim...
    Around,
}
