//! Dump the OpenAPI schema for every public enum.
//!
//! Requires the `utoipa` feature:
//!
//! ```bash
//! cargo run --example utoipa_schema --features utoipa
//! ```

use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
use utoipa::PartialSchema;

fn dump<T: PartialSchema>(name: &str) -> Result<(), serde_json::Error> {
    let schema = T::schema();
    let json = serde_json::to_string_pretty(&schema)?;
    println!("=== {name} ===\n{json}\n");
    Ok(())
}

fn main() -> Result<(), serde_json::Error> {
    dump::<UnderlyingAssetType>("UnderlyingAssetType")?;
    dump::<Action>("Action")?;
    dump::<Side>("Side")?;
    dump::<OptionStyle>("OptionStyle")?;
    Ok(())
}
