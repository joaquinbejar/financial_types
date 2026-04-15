//! Iterate every `UnderlyingAssetType` variant via the `ALL` slice.
//!
//! Demonstrates the canonical pattern for building UI dropdowns,
//! exhaustive validation tables, or schema generators without
//! hand-maintaining a parallel list.

use financial_types::UnderlyingAssetType;

fn main() {
    println!("VARIANT    U8  STOCK?");
    for asset in UnderlyingAssetType::ALL {
        println!(
            "{:<10} {:<3} {}",
            asset.as_str(),
            *asset as u8,
            asset.is_stock()
        );
    }
}
