#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum TextObject {
    #[default]
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
    Around,
}
