//! Integration tests for the `underlying_asset_type` module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::UnderlyingAssetType;

#[test]
fn test_default() {
    assert_eq!(UnderlyingAssetType::default(), UnderlyingAssetType::Stock);
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", UnderlyingAssetType::Crypto), "Crypto");
    assert_eq!(format!("{}", UnderlyingAssetType::Stock), "Stock");
    assert_eq!(format!("{}", UnderlyingAssetType::Forex), "Forex");
    assert_eq!(format!("{}", UnderlyingAssetType::Commodity), "Commodity");
    assert_eq!(format!("{}", UnderlyingAssetType::Bond), "Bond");
    assert_eq!(format!("{}", UnderlyingAssetType::Other), "Other");
    assert_eq!(format!("{}", UnderlyingAssetType::Future), "Future");
    assert_eq!(format!("{}", UnderlyingAssetType::Forward), "Forward");
}

#[test]
fn test_is_helpers() {
    assert!(UnderlyingAssetType::Stock.is_stock());
    assert!(UnderlyingAssetType::Crypto.is_crypto());
    assert!(UnderlyingAssetType::Forex.is_forex());
    assert!(UnderlyingAssetType::Commodity.is_commodity());
    assert!(UnderlyingAssetType::Bond.is_bond());
    assert!(UnderlyingAssetType::Future.is_future());
    assert!(UnderlyingAssetType::Forward.is_forward());
    assert!(!UnderlyingAssetType::Other.is_stock());
    assert!(!UnderlyingAssetType::Stock.is_crypto());
    // Negative coverage for every helper — kills mutants that
    // replace the body with `true`.
    assert!(!UnderlyingAssetType::Stock.is_forex());
    assert!(!UnderlyingAssetType::Stock.is_commodity());
    assert!(!UnderlyingAssetType::Stock.is_bond());
    assert!(!UnderlyingAssetType::Stock.is_future());
    assert!(!UnderlyingAssetType::Stock.is_forward());
    assert!(!UnderlyingAssetType::Crypto.is_stock());
    assert!(!UnderlyingAssetType::Other.is_crypto());
    assert!(!UnderlyingAssetType::Other.is_forex());
    assert!(!UnderlyingAssetType::Other.is_commodity());
    assert!(!UnderlyingAssetType::Other.is_bond());
    assert!(!UnderlyingAssetType::Other.is_future());
    assert!(!UnderlyingAssetType::Other.is_forward());
    assert!(!UnderlyingAssetType::Future.is_forward());
    assert!(!UnderlyingAssetType::Forward.is_future());
}

#[test]
fn test_copy() {
    let asset = UnderlyingAssetType::Crypto;
    let copied = asset;
    assert_eq!(asset, copied);
}

#[test]
fn test_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(UnderlyingAssetType::Stock);
    set.insert(UnderlyingAssetType::Crypto);
    set.insert(UnderlyingAssetType::Stock); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_serialization_roundtrip() {
    let asset = UnderlyingAssetType::Forex;
    let json = serde_json::to_string(&asset).unwrap();
    let deserialized: UnderlyingAssetType = serde_json::from_str(&json).unwrap();
    assert_eq!(asset, deserialized);
}

#[test]
fn test_all_variants_serialize() {
    let variants = [
        UnderlyingAssetType::Crypto,
        UnderlyingAssetType::Stock,
        UnderlyingAssetType::Forex,
        UnderlyingAssetType::Commodity,
        UnderlyingAssetType::Bond,
        UnderlyingAssetType::Other,
        UnderlyingAssetType::Future,
        UnderlyingAssetType::Forward,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: UnderlyingAssetType = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized);
    }
}

#[test]
fn test_repr_u8_size() {
    assert_eq!(
        std::mem::size_of::<UnderlyingAssetType>(),
        1,
        "UnderlyingAssetType should be 1 byte with #[repr(u8)]"
    );
}

#[test]
fn test_from_str_valid() {
    assert_eq!(
        "Stock".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Stock
    );
    assert_eq!(
        "crypto".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Crypto
    );
    assert_eq!(
        "FOREX".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Forex
    );
    assert_eq!(
        "  Commodity  ".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Commodity
    );
    assert_eq!(
        "Future".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Future
    );
    assert_eq!(
        "forward".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Forward
    );
    assert_eq!(
        "  FORWARD  ".parse::<UnderlyingAssetType>().unwrap(),
        UnderlyingAssetType::Forward
    );
}

#[test]
fn test_from_str_invalid() {
    let err = "equity".parse::<UnderlyingAssetType>().unwrap_err();
    assert_eq!(err.kind(), "UnderlyingAssetType");
    assert_eq!(err.input(), "equity");
}

#[test]
fn test_try_from_str() {
    let asset: UnderlyingAssetType = "Bond".try_into().unwrap();
    assert_eq!(asset, UnderlyingAssetType::Bond);
}

#[test]
fn test_try_from_u8_valid() {
    assert_eq!(
        UnderlyingAssetType::try_from(0u8).unwrap(),
        UnderlyingAssetType::Crypto
    );
    assert_eq!(
        UnderlyingAssetType::try_from(1u8).unwrap(),
        UnderlyingAssetType::Stock
    );
    assert_eq!(
        UnderlyingAssetType::try_from(2u8).unwrap(),
        UnderlyingAssetType::Forex
    );
    assert_eq!(
        UnderlyingAssetType::try_from(3u8).unwrap(),
        UnderlyingAssetType::Commodity
    );
    assert_eq!(
        UnderlyingAssetType::try_from(4u8).unwrap(),
        UnderlyingAssetType::Bond
    );
    assert_eq!(
        UnderlyingAssetType::try_from(5u8).unwrap(),
        UnderlyingAssetType::Other
    );
    assert_eq!(
        UnderlyingAssetType::try_from(6u8).unwrap(),
        UnderlyingAssetType::Future
    );
    assert_eq!(
        UnderlyingAssetType::try_from(7u8).unwrap(),
        UnderlyingAssetType::Forward
    );
}

#[test]
fn test_try_from_u8_invalid() {
    let err = UnderlyingAssetType::try_from(99u8).unwrap_err();
    assert_eq!(err.kind(), "UnderlyingAssetType");
}

#[test]
fn test_display_parse_roundtrip() {
    for variant in [
        UnderlyingAssetType::Crypto,
        UnderlyingAssetType::Stock,
        UnderlyingAssetType::Forex,
        UnderlyingAssetType::Commodity,
        UnderlyingAssetType::Bond,
        UnderlyingAssetType::Other,
        UnderlyingAssetType::Future,
        UnderlyingAssetType::Forward,
    ] {
        let s = format!("{variant}");
        let parsed: UnderlyingAssetType = s.parse().unwrap();
        assert_eq!(variant, parsed);
    }
}

#[test]
fn test_as_str_matches_display() {
    for variant in [
        UnderlyingAssetType::Crypto,
        UnderlyingAssetType::Stock,
        UnderlyingAssetType::Forex,
        UnderlyingAssetType::Commodity,
        UnderlyingAssetType::Bond,
        UnderlyingAssetType::Other,
        UnderlyingAssetType::Future,
        UnderlyingAssetType::Forward,
    ] {
        assert_eq!(variant.as_str(), format!("{variant}"));
    }
}

#[test]
fn test_as_str_is_const() {
    const STOCK: &str = UnderlyingAssetType::Stock.as_str();
    assert_eq!(STOCK, "Stock");
}

#[test]
fn test_all_variants_ordered() {
    assert_eq!(
        UnderlyingAssetType::ALL,
        &[
            UnderlyingAssetType::Crypto,
            UnderlyingAssetType::Stock,
            UnderlyingAssetType::Forex,
            UnderlyingAssetType::Commodity,
            UnderlyingAssetType::Bond,
            UnderlyingAssetType::Other,
            UnderlyingAssetType::Future,
            UnderlyingAssetType::Forward,
        ]
    );
    for (index, variant) in UnderlyingAssetType::ALL.iter().enumerate() {
        let round = UnderlyingAssetType::try_from(index as u8).unwrap();
        assert_eq!(round, *variant);
    }
}

#[test]
fn test_ord_matches_discriminants() {
    let mut v = vec![
        UnderlyingAssetType::Other,
        UnderlyingAssetType::Crypto,
        UnderlyingAssetType::Bond,
        UnderlyingAssetType::Stock,
    ];
    v.sort();
    assert_eq!(
        v,
        vec![
            UnderlyingAssetType::Crypto,
            UnderlyingAssetType::Stock,
            UnderlyingAssetType::Bond,
            UnderlyingAssetType::Other,
        ]
    );
    assert!(UnderlyingAssetType::Crypto < UnderlyingAssetType::Stock);
}
