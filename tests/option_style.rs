//! Integration tests for the `option_style` module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::OptionStyle;

#[test]
fn test_default() {
    assert_eq!(OptionStyle::default(), OptionStyle::Call);
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", OptionStyle::Call), "Call");
    assert_eq!(format!("{}", OptionStyle::Put), "Put");
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", OptionStyle::Call), "OptionStyle::Call");
    assert_eq!(format!("{:?}", OptionStyle::Put), "OptionStyle::Put");
}

#[test]
fn test_is_helpers() {
    assert!(OptionStyle::Call.is_call());
    assert!(!OptionStyle::Call.is_put());
    assert!(OptionStyle::Put.is_put());
    assert!(!OptionStyle::Put.is_call());
}

#[test]
fn test_opposite() {
    assert_eq!(OptionStyle::Call.opposite(), OptionStyle::Put);
    assert_eq!(OptionStyle::Put.opposite(), OptionStyle::Call);
}

#[test]
fn test_ordering() {
    assert!(OptionStyle::Call < OptionStyle::Put);
}

#[test]
fn test_copy() {
    let style = OptionStyle::Call;
    let copied = style;
    assert_eq!(style, copied);
}

#[test]
fn test_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(OptionStyle::Call);
    set.insert(OptionStyle::Put);
    set.insert(OptionStyle::Call); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_serialization_roundtrip() {
    let style = OptionStyle::Put;
    let json = serde_json::to_string(&style).unwrap();
    let deserialized: OptionStyle = serde_json::from_str(&json).unwrap();
    assert_eq!(style, deserialized);
}

#[test]
fn test_repr_u8_size() {
    assert_eq!(
        std::mem::size_of::<OptionStyle>(),
        1,
        "OptionStyle should be 1 byte with #[repr(u8)]"
    );
}

#[test]
fn test_from_str_valid() {
    assert_eq!("Call".parse::<OptionStyle>().unwrap(), OptionStyle::Call);
    assert_eq!("put".parse::<OptionStyle>().unwrap(), OptionStyle::Put);
    assert_eq!("CALL".parse::<OptionStyle>().unwrap(), OptionStyle::Call);
}

#[test]
fn test_from_str_invalid() {
    let err = "straddle".parse::<OptionStyle>().unwrap_err();
    assert_eq!(err.kind(), "OptionStyle");
}

#[test]
fn test_try_from_u8() {
    assert_eq!(OptionStyle::try_from(0u8).unwrap(), OptionStyle::Call);
    assert_eq!(OptionStyle::try_from(1u8).unwrap(), OptionStyle::Put);
    assert!(OptionStyle::try_from(42u8).is_err());
}

#[test]
fn test_try_from_str() {
    // Asserts non-default variant — kills mutants that replace the
    // body with `Ok(Default::default())` (which would yield Call).
    let put: OptionStyle = "Put".try_into().unwrap();
    assert_eq!(put, OptionStyle::Put);
    let err = <OptionStyle as TryFrom<&str>>::try_from("nope").unwrap_err();
    assert_eq!(err.kind(), "OptionStyle");
}

#[test]
fn test_display_parse_roundtrip() {
    for variant in [OptionStyle::Call, OptionStyle::Put] {
        let parsed: OptionStyle = format!("{variant}").parse().unwrap();
        assert_eq!(variant, parsed);
    }
}

#[test]
fn test_as_str_matches_display() {
    for variant in [OptionStyle::Call, OptionStyle::Put] {
        assert_eq!(variant.as_str(), format!("{variant}"));
    }
}

#[test]
fn test_as_str_is_const() {
    const CALL: &str = OptionStyle::Call.as_str();
    assert_eq!(CALL, "Call");
}

#[test]
fn test_all_variants_ordered() {
    assert_eq!(OptionStyle::ALL, &[OptionStyle::Call, OptionStyle::Put]);
}
