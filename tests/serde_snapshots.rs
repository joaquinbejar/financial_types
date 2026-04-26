//! Wire-format snapshots for serde JSON serialization.
//!
//! Asserts the *exact* JSON encoding for every variant of every public
//! enum. A round-trip test on its own would silently tolerate a string
//! rename like `"Long"` → `"long"`, which would break every record
//! persisted by downstream consumers. These snapshots make the wire
//! format part of the SemVer-visible public contract.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};

#[test]
fn snapshot_underlying_asset_type() {
    let cases = [
        (UnderlyingAssetType::Crypto, "\"Crypto\""),
        (UnderlyingAssetType::Stock, "\"Stock\""),
        (UnderlyingAssetType::Forex, "\"Forex\""),
        (UnderlyingAssetType::Commodity, "\"Commodity\""),
        (UnderlyingAssetType::Bond, "\"Bond\""),
        (UnderlyingAssetType::Other, "\"Other\""),
        (UnderlyingAssetType::Future, "\"Future\""),
        (UnderlyingAssetType::Forward, "\"Forward\""),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected, "encoding regression for {variant:?}");
        let parsed: UnderlyingAssetType = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed, variant, "decoding regression for {expected}");
    }
}

#[test]
fn snapshot_action() {
    let cases = [
        (Action::Buy, "\"Buy\""),
        (Action::Sell, "\"Sell\""),
        (Action::Other, "\"Other\""),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected, "encoding regression for {variant:?}");
        let parsed: Action = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed, variant, "decoding regression for {expected}");
    }
}

#[test]
fn snapshot_side() {
    let cases = [(Side::Long, "\"Long\""), (Side::Short, "\"Short\"")];
    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected, "encoding regression for {variant:?}");
        let parsed: Side = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed, variant, "decoding regression for {expected}");
    }
}

#[test]
fn snapshot_option_style() {
    let cases = [
        (OptionStyle::Call, "\"Call\""),
        (OptionStyle::Put, "\"Put\""),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected, "encoding regression for {variant:?}");
        let parsed: OptionStyle = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed, variant, "decoding regression for {expected}");
    }
}
