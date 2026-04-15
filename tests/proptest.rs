//! Integration tests for the `proptest` module.

#![cfg(feature = "proptest")]
#![allow(clippy::unwrap_used)]

use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_side_serde_roundtrip(side: Side) {
        let json = serde_json::to_string(&side).unwrap();
        let back: Side = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(side, back);
    }

    #[test]
    fn prop_action_display_parse_roundtrip(action: Action) {
        let parsed: Action = action.as_str().parse().unwrap();
        prop_assert_eq!(action, parsed);
    }

    #[test]
    fn prop_underlying_asset_u8_roundtrip(asset: UnderlyingAssetType) {
        let pos = UnderlyingAssetType::ALL
            .iter()
            .position(|v| v == &asset)
            .unwrap();
        let back = UnderlyingAssetType::try_from(pos as u8).unwrap();
        prop_assert_eq!(asset, back);
    }

    #[test]
    fn prop_option_style_opposite_involution(style: OptionStyle) {
        prop_assert_eq!(style.opposite().opposite(), style);
    }
}
