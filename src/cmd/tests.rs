use super::*;

#[test]
fn test_pattern_write() {
    let x = Pattern::try_from("w[rite]").expect("Failed to parse pattern");
    assert_eq!(true, x.matches("w"));
    assert_eq!(true, x.matches("wr"));
    assert_eq!(true, x.matches("wri"));
    assert_eq!(true, x.matches("writ"));
    assert_eq!(true, x.matches("write"));
    assert_eq!(false, x.matches("writex"));
}
