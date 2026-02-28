use crate::span::Span;

use super::*;

use Boundary::*;
use TextObject::*;

#[test]
fn test_span() {
    let got = span("the cat sat", Location::from((0, 5)), Inner, Word);
    assert_eq!(got, Some(Span::from((0, 4, 0, 7))));
}
