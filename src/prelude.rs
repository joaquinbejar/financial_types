//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the
//! `financial_types` crate, allowing users to import everything they need
//! with a single `use` statement:
//!
//! ```rust
//! use financial_types::prelude::*;
//! ```

pub use crate::Action;
pub use crate::OptionStyle;
pub use crate::ParseEnumError;
pub use crate::Side;
pub use crate::UnderlyingAssetType;
