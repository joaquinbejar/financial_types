//! # Financial Types
//!
//! Core financial type definitions for trading systems.
//!
//! This crate provides fundamental enums used across financial applications:
//! - [`UnderlyingAssetType`] — Classification of asset classes (stocks, crypto, forex, etc.)
//! - [`Action`] — Trading actions (buy, sell)
//! - [`Side`] — Position directional exposure (long, short)
//! - [`OptionStyle`] — Option contract style (call, put)
//!
//! All enums use `#[repr(u8)]` for compact memory layout and are designed for
//! high-performance financial systems.
//!
//! ## Features
//!
//! - Full `serde` serialization/deserialization support
//! - Optional `utoipa` support for OpenAPI schema generation (enable the `utoipa` feature)
//! - `#[repr(u8)]` on all enums for deterministic layout
//! - `#[must_use]` on pure helper methods
//! - `FromStr`, `TryFrom<&str>`, and `TryFrom<u8>` on every public enum,
//!   returning [`ParseEnumError`] on failure. String parsing is
//!   case-insensitive and trims whitespace.
//!
//! ## Usage
//!
//! ```rust
//! use financial_types::{Action, Side, OptionStyle, UnderlyingAssetType};
//!
//! let action = Action::Buy;
//! let side = Side::Long;
//! let style = OptionStyle::Call;
//! let asset = UnderlyingAssetType::Stock;
//!
//! assert_eq!(format!("{action}"), "Buy");
//! assert_eq!(format!("{side}"), "Long");
//! assert_eq!(format!("{style}"), "Call");
//! assert_eq!(format!("{asset}"), "Stock");
//!
//! assert!(style.is_call());
//! assert!(side.is_long());
//! assert!(action.is_buy());
//! ```

pub mod prelude;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Error returned when a string or `u8` cannot be converted into one of the
/// public financial enums defined by this crate.
///
/// This is the error type produced by `FromStr`, `TryFrom<&str>`, and
/// `TryFrom<u8>` implementations on [`UnderlyingAssetType`], [`Action`],
/// [`Side`], and [`OptionStyle`].
///
/// # Examples
///
/// ```rust
/// use financial_types::{Side, ParseEnumError};
/// use std::str::FromStr;
///
/// let err = Side::from_str("sideways").unwrap_err();
/// assert_eq!(err.kind(), "Side");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ParseEnumError {
    /// Name of the enum that failed to parse (e.g. `"Side"`).
    kind: &'static str,
    /// Human-readable description of the invalid input.
    input: String,
}

impl ParseEnumError {
    /// Creates a new [`ParseEnumError`].
    #[must_use]
    #[inline]
    pub fn new(kind: &'static str, input: impl Into<String>) -> Self {
        Self {
            kind,
            input: input.into(),
        }
    }

    /// Returns the name of the enum that failed to parse.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// Returns the original (stringified) input that was rejected.
    #[must_use]
    #[inline]
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl fmt::Display for ParseEnumError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {} value: {:?}", self.kind, self.input)
    }
}

impl std::error::Error for ParseEnumError {}

/// Classification of the underlying asset for a financial instrument.
///
/// Represents the broad asset class to which an instrument belongs.
/// Used for routing, risk bucketing, and display purposes.
///
/// # Examples
///
/// ```rust
/// use financial_types::UnderlyingAssetType;
///
/// let asset = UnderlyingAssetType::default();
/// assert_eq!(asset, UnderlyingAssetType::Stock);
/// assert_eq!(format!("{asset}"), "Stock");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
pub enum UnderlyingAssetType {
    /// Cryptocurrency assets (e.g., BTC, ETH).
    Crypto = 0,
    /// Stock/equity assets (e.g., AAPL, GOOGL).
    #[default]
    Stock = 1,
    /// Foreign exchange currency pairs (e.g., EUR/USD).
    Forex = 2,
    /// Commodity assets (e.g., Gold, Oil).
    Commodity = 3,
    /// Bond/fixed income securities.
    Bond = 4,
    /// Other asset types not covered by the above categories.
    Other = 5,
}

impl UnderlyingAssetType {
    /// Returns `true` if this is a [`Stock`](Self::Stock) variant.
    #[must_use]
    #[inline]
    pub const fn is_stock(&self) -> bool {
        matches!(self, Self::Stock)
    }

    /// Returns `true` if this is a [`Crypto`](Self::Crypto) variant.
    #[must_use]
    #[inline]
    pub const fn is_crypto(&self) -> bool {
        matches!(self, Self::Crypto)
    }

    /// Returns `true` if this is a [`Forex`](Self::Forex) variant.
    #[must_use]
    #[inline]
    pub const fn is_forex(&self) -> bool {
        matches!(self, Self::Forex)
    }

    /// Returns `true` if this is a [`Commodity`](Self::Commodity) variant.
    #[must_use]
    #[inline]
    pub const fn is_commodity(&self) -> bool {
        matches!(self, Self::Commodity)
    }

    /// Returns `true` if this is a [`Bond`](Self::Bond) variant.
    #[must_use]
    #[inline]
    pub const fn is_bond(&self) -> bool {
        matches!(self, Self::Bond)
    }

    /// Returns the canonical string representation of this variant.
    ///
    /// Matches the [`fmt::Display`] output exactly and is allocation-free.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Crypto => "Crypto",
            Self::Stock => "Stock",
            Self::Forex => "Forex",
            Self::Commodity => "Commodity",
            Self::Bond => "Bond",
            Self::Other => "Other",
        }
    }

    /// All variants in discriminant order.
    ///
    /// Useful for iteration, UI dropdowns, exhaustive validation, and
    /// schema generation. Order matches the `#[repr(u8)]` values.
    pub const ALL: &'static [Self] = &[
        Self::Crypto,
        Self::Stock,
        Self::Forex,
        Self::Commodity,
        Self::Bond,
        Self::Other,
    ];
}

impl fmt::Display for UnderlyingAssetType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for UnderlyingAssetType {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "crypto" => Ok(Self::Crypto),
            "stock" => Ok(Self::Stock),
            "forex" => Ok(Self::Forex),
            "commodity" => Ok(Self::Commodity),
            "bond" => Ok(Self::Bond),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError::new("UnderlyingAssetType", s)),
        }
    }
}

impl TryFrom<&str> for UnderlyingAssetType {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<u8> for UnderlyingAssetType {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Crypto),
            1 => Ok(Self::Stock),
            2 => Ok(Self::Forex),
            3 => Ok(Self::Commodity),
            4 => Ok(Self::Bond),
            5 => Ok(Self::Other),
            other => Err(ParseEnumError::new(
                "UnderlyingAssetType",
                other.to_string(),
            )),
        }
    }
}

/// Represents trading actions in a financial context.
///
/// Defines the fundamental trade operations that can be performed in a
/// trading system. These actions represent the direction of a trade
/// transaction.
///
/// # Examples
///
/// ```rust
/// use financial_types::Action;
///
/// let action = Action::Buy;
/// assert!(action.is_buy());
/// assert_eq!(format!("{action}"), "Buy");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
pub enum Action {
    /// Represents a purchase transaction, where assets are acquired.
    #[default]
    Buy = 0,
    /// Represents a selling transaction, where assets are disposed of.
    Sell = 1,
    /// Action is not applicable to this type of transaction.
    Other = 2,
}

impl Action {
    /// Returns `true` if this is a [`Buy`](Self::Buy) action.
    #[must_use]
    #[inline]
    pub const fn is_buy(&self) -> bool {
        matches!(self, Self::Buy)
    }

    /// Returns `true` if this is a [`Sell`](Self::Sell) action.
    #[must_use]
    #[inline]
    pub const fn is_sell(&self) -> bool {
        matches!(self, Self::Sell)
    }

    /// Returns the canonical string representation of this variant.
    ///
    /// Matches the [`fmt::Display`] output exactly and is allocation-free.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "Buy",
            Self::Sell => "Sell",
            Self::Other => "Other",
        }
    }

    /// All variants in discriminant order.
    pub const ALL: &'static [Self] = &[Self::Buy, Self::Sell, Self::Other];

    /// Returns the opposite trading action.
    ///
    /// - `Buy` → `Sell`
    /// - `Sell` → `Buy`
    /// - `Other` → `Other` (no meaningful inverse)
    #[must_use]
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
            Self::Other => Self::Other,
        }
    }
}

impl fmt::Display for Action {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Action {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "buy" => Ok(Self::Buy),
            "sell" => Ok(Self::Sell),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError::new("Action", s)),
        }
    }
}

impl TryFrom<&str> for Action {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<u8> for Action {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Buy),
            1 => Ok(Self::Sell),
            2 => Ok(Self::Other),
            other => Err(ParseEnumError::new("Action", other.to_string())),
        }
    }
}

/// Defines the directional exposure of a financial position.
///
/// Indicates whether a trader expects to profit from rising prices (Long)
/// or falling prices (Short). This is a fundamental concept in trading that
/// determines how profits and losses are calculated and affects risk
/// management considerations.
///
/// # Examples
///
/// ```rust
/// use financial_types::Side;
///
/// let side = Side::Long;
/// assert!(side.is_long());
/// assert!(!side.is_short());
/// assert_eq!(format!("{side}"), "Long");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
pub enum Side {
    /// Profits when the underlying asset's price increases.
    /// Long positions involve buying an asset with the expectation of selling
    /// at a higher price.
    #[default]
    Long = 0,
    /// Profits when the underlying asset's price decreases.
    /// Short positions involve selling an asset (often borrowed) with the
    /// expectation of buying it back at a lower price.
    Short = 1,
}

impl Side {
    /// Returns `true` if this is a [`Long`](Self::Long) position.
    #[must_use]
    #[inline]
    pub const fn is_long(&self) -> bool {
        matches!(self, Self::Long)
    }

    /// Returns `true` if this is a [`Short`](Self::Short) position.
    #[must_use]
    #[inline]
    pub const fn is_short(&self) -> bool {
        matches!(self, Self::Short)
    }

    /// Returns the opposite side.
    ///
    /// - `Long` → `Short`
    /// - `Short` → `Long`
    #[must_use]
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Long => Self::Short,
            Self::Short => Self::Long,
        }
    }

    /// Returns the canonical string representation of this variant.
    ///
    /// Matches the [`fmt::Display`] output exactly and is allocation-free.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Long => "Long",
            Self::Short => "Short",
        }
    }

    /// All variants in discriminant order.
    pub const ALL: &'static [Self] = &[Self::Long, Self::Short];
}

impl fmt::Display for Side {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Side {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Long => write!(f, "Side::Long"),
            Self::Short => write!(f, "Side::Short"),
        }
    }
}

impl FromStr for Side {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(ParseEnumError::new("Side", s)),
        }
    }
}

impl TryFrom<&str> for Side {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<u8> for Side {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Long),
            1 => Ok(Self::Short),
            other => Err(ParseEnumError::new("Side", other.to_string())),
        }
    }
}

/// Specifies the style of an option contract.
///
/// Defines the fundamental classification of options contracts based on their
/// payoff direction. The style determines whether the holder has the right to
/// buy (Call) or sell (Put) the underlying asset.
///
/// This is a critical attribute for options contracts as it directly affects
/// valuation, pricing models, and exercise strategies.
///
/// # Examples
///
/// ```rust
/// use financial_types::OptionStyle;
///
/// let style = OptionStyle::Call;
/// assert!(style.is_call());
/// assert!(!style.is_put());
/// assert_eq!(format!("{style}"), "Call");
/// assert!(OptionStyle::Call < OptionStyle::Put);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
pub enum OptionStyle {
    /// A call option gives the holder the right (but not obligation) to **buy**
    /// the underlying asset at the strike price before or at expiration.
    /// Call options typically increase in value when the underlying asset price rises.
    #[default]
    Call = 0,
    /// A put option gives the holder the right (but not obligation) to **sell**
    /// the underlying asset at the strike price before or at expiration.
    /// Put options typically increase in value when the underlying asset price falls.
    Put = 1,
}

impl OptionStyle {
    /// Returns `true` if this is a [`Call`](Self::Call) option.
    #[must_use]
    #[inline]
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::Call)
    }

    /// Returns `true` if this is a [`Put`](Self::Put) option.
    #[must_use]
    #[inline]
    pub const fn is_put(&self) -> bool {
        matches!(self, Self::Put)
    }

    /// Returns the opposite option style.
    ///
    /// - `Call` → `Put`
    /// - `Put` → `Call`
    #[must_use]
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Call => Self::Put,
            Self::Put => Self::Call,
        }
    }

    /// Returns the canonical string representation of this variant.
    ///
    /// Matches the [`fmt::Display`] output exactly and is allocation-free.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "Call",
            Self::Put => "Put",
        }
    }

    /// All variants in discriminant order.
    pub const ALL: &'static [Self] = &[Self::Call, Self::Put];
}

impl fmt::Display for OptionStyle {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for OptionStyle {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Call => write!(f, "OptionStyle::Call"),
            Self::Put => write!(f, "OptionStyle::Put"),
        }
    }
}

impl FromStr for OptionStyle {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "call" => Ok(Self::Call),
            "put" => Ok(Self::Put),
            _ => Err(ParseEnumError::new("OptionStyle", s)),
        }
    }
}

impl TryFrom<&str> for OptionStyle {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<u8> for OptionStyle {
    type Error = ParseEnumError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Call),
            1 => Ok(Self::Put),
            other => Err(ParseEnumError::new("OptionStyle", other.to_string())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests_underlying_asset_type {
    use super::*;

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
    }

    #[test]
    fn test_is_helpers() {
        assert!(UnderlyingAssetType::Stock.is_stock());
        assert!(UnderlyingAssetType::Crypto.is_crypto());
        assert!(UnderlyingAssetType::Forex.is_forex());
        assert!(UnderlyingAssetType::Commodity.is_commodity());
        assert!(UnderlyingAssetType::Bond.is_bond());
        assert!(!UnderlyingAssetType::Other.is_stock());
        assert!(!UnderlyingAssetType::Stock.is_crypto());
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
            ]
        );
        for (index, variant) in UnderlyingAssetType::ALL.iter().enumerate() {
            let round = UnderlyingAssetType::try_from(index as u8).unwrap();
            assert_eq!(round, *variant);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests_action {
    use super::*;

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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests_side {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(Side::default(), Side::Long);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Side::Long), "Long");
        assert_eq!(format!("{}", Side::Short), "Short");
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Side::Long), "Side::Long");
        assert_eq!(format!("{:?}", Side::Short), "Side::Short");
    }

    #[test]
    fn test_is_helpers() {
        assert!(Side::Long.is_long());
        assert!(!Side::Long.is_short());
        assert!(Side::Short.is_short());
        assert!(!Side::Short.is_long());
    }

    #[test]
    fn test_opposite() {
        assert_eq!(Side::Long.opposite(), Side::Short);
        assert_eq!(Side::Short.opposite(), Side::Long);
    }

    #[test]
    fn test_copy() {
        let side = Side::Long;
        let copied = side;
        assert_eq!(side, copied);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Side::Long);
        set.insert(Side::Short);
        set.insert(Side::Long); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let side = Side::Short;
        let json = serde_json::to_string(&side).unwrap();
        let deserialized: Side = serde_json::from_str(&json).unwrap();
        assert_eq!(side, deserialized);
    }

    #[test]
    fn test_repr_u8_size() {
        assert_eq!(
            std::mem::size_of::<Side>(),
            1,
            "Side should be 1 byte with #[repr(u8)]"
        );
    }

    #[test]
    fn test_from_str_valid() {
        assert_eq!("Long".parse::<Side>().unwrap(), Side::Long);
        assert_eq!("short".parse::<Side>().unwrap(), Side::Short);
        assert_eq!("LONG".parse::<Side>().unwrap(), Side::Long);
    }

    #[test]
    fn test_from_str_invalid() {
        let err = "sideways".parse::<Side>().unwrap_err();
        assert_eq!(err.kind(), "Side");
        assert_eq!(err.input(), "sideways");
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(Side::try_from(0u8).unwrap(), Side::Long);
        assert_eq!(Side::try_from(1u8).unwrap(), Side::Short);
        assert!(Side::try_from(7u8).is_err());
    }

    #[test]
    fn test_display_parse_roundtrip() {
        for variant in [Side::Long, Side::Short] {
            let parsed: Side = format!("{variant}").parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn test_as_str_matches_display() {
        for variant in [Side::Long, Side::Short] {
            assert_eq!(variant.as_str(), format!("{variant}"));
        }
    }

    #[test]
    fn test_as_str_is_const() {
        const LONG: &str = Side::Long.as_str();
        assert_eq!(LONG, "Long");
    }

    #[test]
    fn test_all_variants_ordered() {
        assert_eq!(Side::ALL, &[Side::Long, Side::Short]);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests_option_style {
    use super::*;

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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests_parse_enum_error {
    use super::*;

    #[test]
    fn test_display() {
        let err = ParseEnumError::new("Side", "sideways");
        assert_eq!(format!("{err}"), "invalid Side value: \"sideways\"");
    }

    #[test]
    fn test_error_trait() {
        let err = ParseEnumError::new("Action", "hold");
        let _: &dyn std::error::Error = &err;
    }
}
