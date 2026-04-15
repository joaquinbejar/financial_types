//! Round-trip every public enum through `serde_json` and verify the
//! decoded value equals the original. The wire format is part of the
//! crate's public contract; this example is a concrete demonstration.

use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
use serde::{Deserialize, Serialize};

fn roundtrip<T>(value: &T) -> Result<String, serde_json::Error>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value)?;
    let back: T = serde_json::from_str(&json)?;
    assert_eq!(&back, value, "round-trip mismatch");
    Ok(json)
}

fn main() -> Result<(), serde_json::Error> {
    for asset in UnderlyingAssetType::ALL {
        println!("{:<22} {}", "UnderlyingAssetType", roundtrip(asset)?);
    }
    for action in Action::ALL {
        println!("{:<22} {}", "Action", roundtrip(action)?);
    }
    for side in Side::ALL {
        println!("{:<22} {}", "Side", roundtrip(side)?);
    }
    for style in OptionStyle::ALL {
        println!("{:<22} {}", "OptionStyle", roundtrip(style)?);
    }
    Ok(())
}
