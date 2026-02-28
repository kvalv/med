use crate::span::{Location, Span};

enum TextObject {
    Block,
    Word,
    // Paren, // ()
    // Brack, // <>
}

pub enum Boundary {
    Inner,
    Around,
}

/// For given text and location of cursor, figure out what characters are included in the
/// obj+boundary
pub fn span(text: &str, cursor: Location, boundary: Boundary, obj: TextObject) -> Option<Span> {
    None
}

#[cfg(test)]
mod tests;
