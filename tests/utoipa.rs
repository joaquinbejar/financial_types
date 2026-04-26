//! Integration tests for the optional `utoipa` feature.
//!
//! Verifies that every public enum produces a valid OpenAPI schema
//! containing every variant name. Without this, a forgotten
//! `#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]` on a
//! new enum would silently ship.

#![cfg(feature = "utoipa")]
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
use utoipa::PartialSchema;

fn schema_json<T: PartialSchema>() -> String {
    serde_json::to_string(&T::schema()).unwrap()
}

#[test]
fn schema_underlying_asset_type_lists_every_variant() {
    let json = schema_json::<UnderlyingAssetType>();
    for variant in [
        "Crypto",
        "Stock",
        "Forex",
        "Commodity",
        "Bond",
        "Other",
        "Future",
        "Forward",
    ] {
        assert!(
            json.contains(variant),
            "UnderlyingAssetType schema missing variant {variant}: {json}"
        );
    }
}

#[test]
fn schema_action_lists_every_variant() {
    let json = schema_json::<Action>();
    for variant in ["Buy", "Sell", "Other"] {
        assert!(
            json.contains(variant),
            "Action schema missing variant {variant}: {json}"
        );
    }
}

#[test]
fn schema_side_lists_every_variant() {
    let json = schema_json::<Side>();
    for variant in ["Long", "Short"] {
        assert!(
            json.contains(variant),
            "Side schema missing variant {variant}: {json}"
        );
    }
}

#[test]
fn schema_option_style_lists_every_variant() {
    let json = schema_json::<OptionStyle>();
    for variant in ["Call", "Put"] {
        assert!(
            json.contains(variant),
            "OptionStyle schema missing variant {variant}: {json}"
        );
    }
}
