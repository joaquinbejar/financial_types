//! Integration tests for the `side` module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::Side;

#[test]
fn test_default() {
    assert_eq!(Side::default(), Side::Long);
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", Side::Long), "Long");
    assert_eq!(format!("{}", Side::Short), "Short");
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", Side::Long), "Side::Long");
    assert_eq!(format!("{:?}", Side::Short), "Side::Short");
}

#[test]
fn test_is_helpers() {
    assert!(Side::Long.is_long());
    assert!(!Side::Long.is_short());
    assert!(Side::Short.is_short());
    assert!(!Side::Short.is_long());
}

#[test]
fn test_opposite() {
    assert_eq!(Side::Long.opposite(), Side::Short);
    assert_eq!(Side::Short.opposite(), Side::Long);
}

#[test]
fn test_copy() {
    let side = Side::Long;
    let copied = side;
    assert_eq!(side, copied);
}

#[test]
fn test_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Side::Long);
    set.insert(Side::Short);
    set.insert(Side::Long); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_serialization_roundtrip() {
    let side = Side::Short;
    let json = serde_json::to_string(&side).unwrap();
    let deserialized: Side = serde_json::from_str(&json).unwrap();
    assert_eq!(side, deserialized);
}

#[test]
fn test_repr_u8_size() {
    assert_eq!(
        std::mem::size_of::<Side>(),
        1,
        "Side should be 1 byte with #[repr(u8)]"
    );
}

#[test]
fn test_from_str_valid() {
    assert_eq!("Long".parse::<Side>().unwrap(), Side::Long);
    assert_eq!("short".parse::<Side>().unwrap(), Side::Short);
    assert_eq!("LONG".parse::<Side>().unwrap(), Side::Long);
}

#[test]
fn test_from_str_invalid() {
    let err = "sideways".parse::<Side>().unwrap_err();
    assert_eq!(err.kind(), "Side");
    assert_eq!(err.input(), "sideways");
}

#[test]
fn test_try_from_u8() {
    assert_eq!(Side::try_from(0u8).unwrap(), Side::Long);
    assert_eq!(Side::try_from(1u8).unwrap(), Side::Short);
    assert!(Side::try_from(7u8).is_err());
}

#[test]
fn test_try_from_str() {
    // Asserts non-default variant — kills mutants that replace the
    // body with `Ok(Default::default())` (which would yield Long).
    let short: Side = "Short".try_into().unwrap();
    assert_eq!(short, Side::Short);
    let err = <Side as TryFrom<&str>>::try_from("nope").unwrap_err();
    assert_eq!(err.kind(), "Side");
}

#[test]
fn test_display_parse_roundtrip() {
    for variant in [Side::Long, Side::Short] {
        let parsed: Side = format!("{variant}").parse().unwrap();
        assert_eq!(variant, parsed);
    }
}

#[test]
fn test_as_str_matches_display() {
    for variant in [Side::Long, Side::Short] {
        assert_eq!(variant.as_str(), format!("{variant}"));
    }
}

#[test]
fn test_as_str_is_const() {
    const LONG: &str = Side::Long.as_str();
    assert_eq!(LONG, "Long");
}

#[test]
fn test_all_variants_ordered() {
    assert_eq!(Side::ALL, &[Side::Long, Side::Short]);
}

#[test]
fn test_ord_matches_discriminants() {
    assert!(Side::Long < Side::Short);
}
