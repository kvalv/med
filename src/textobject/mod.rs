pub enum TextObject {
    Block,
    Word,
    // Paren, // ()
    // Brack, // <>
}

pub enum Boundary {
    Current, // From current location. I would like to use 'None', but that would be bad.
    Inner,
    Around,
}
