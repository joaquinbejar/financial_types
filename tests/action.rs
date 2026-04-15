//! Integration tests for the `action` module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::Action;

#[test]
fn test_default() {
    assert_eq!(Action::default(), Action::Buy);
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", Action::Buy), "Buy");
    assert_eq!(format!("{}", Action::Sell), "Sell");
    assert_eq!(format!("{}", Action::Other), "Other");
}

#[test]
fn test_is_helpers() {
    assert!(Action::Buy.is_buy());
    assert!(!Action::Buy.is_sell());
    assert!(Action::Sell.is_sell());
    assert!(!Action::Sell.is_buy());
    assert!(!Action::Other.is_buy());
    assert!(!Action::Other.is_sell());
}

#[test]
fn test_copy() {
    let action = Action::Buy;
    let copied = action;
    assert_eq!(action, copied);
}

#[test]
fn test_serialization_roundtrip() {
    let action = Action::Sell;
    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn test_repr_u8_size() {
    assert_eq!(
        std::mem::size_of::<Action>(),
        1,
        "Action should be 1 byte with #[repr(u8)]"
    );
}

#[test]
fn test_from_str_valid() {
    assert_eq!("Buy".parse::<Action>().unwrap(), Action::Buy);
    assert_eq!("sell".parse::<Action>().unwrap(), Action::Sell);
    assert_eq!("OTHER".parse::<Action>().unwrap(), Action::Other);
}

#[test]
fn test_from_str_invalid() {
    let err = "hold".parse::<Action>().unwrap_err();
    assert_eq!(err.kind(), "Action");
}

#[test]
fn test_try_from_u8() {
    assert_eq!(Action::try_from(0u8).unwrap(), Action::Buy);
    assert_eq!(Action::try_from(1u8).unwrap(), Action::Sell);
    assert_eq!(Action::try_from(2u8).unwrap(), Action::Other);
    assert!(Action::try_from(3u8).is_err());
}

#[test]
fn test_display_parse_roundtrip() {
    for variant in [Action::Buy, Action::Sell, Action::Other] {
        let parsed: Action = format!("{variant}").parse().unwrap();
        assert_eq!(variant, parsed);
    }
}

#[test]
fn test_as_str_matches_display() {
    for variant in [Action::Buy, Action::Sell, Action::Other] {
        assert_eq!(variant.as_str(), format!("{variant}"));
    }
}

#[test]
fn test_as_str_is_const() {
    const BUY: &str = Action::Buy.as_str();
    assert_eq!(BUY, "Buy");
}

#[test]
fn test_all_variants_ordered() {
    assert_eq!(Action::ALL, &[Action::Buy, Action::Sell, Action::Other]);
}

#[test]
fn test_opposite() {
    assert_eq!(Action::Buy.opposite(), Action::Sell);
    assert_eq!(Action::Sell.opposite(), Action::Buy);
    assert_eq!(Action::Other.opposite(), Action::Other);
    // Symmetry
    for v in Action::ALL {
        assert_eq!(v.opposite().opposite(), *v);
    }
}

#[test]
fn test_ord_matches_discriminants() {
    assert!(Action::Buy < Action::Sell);
    assert!(Action::Sell < Action::Other);
}
