#![cfg_attr(not(any(feature = "std", test)), no_std)]

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
//! ## Wire format
//!
//! `serde` JSON encoding is part of the public contract. The string
//! used for every variant is fixed and SemVer-tracked: renaming any
//! variant string is a breaking change.
//!
//! | Enum                  | Variant     | JSON          |
//! |-----------------------|-------------|---------------|
//! | `UnderlyingAssetType` | `Crypto`    | `"Crypto"`    |
//! | `UnderlyingAssetType` | `Stock`     | `"Stock"`     |
//! | `UnderlyingAssetType` | `Forex`     | `"Forex"`     |
//! | `UnderlyingAssetType` | `Commodity` | `"Commodity"` |
//! | `UnderlyingAssetType` | `Bond`      | `"Bond"`      |
//! | `UnderlyingAssetType` | `Other`     | `"Other"`     |
//! | `Action`              | `Buy`       | `"Buy"`       |
//! | `Action`              | `Sell`      | `"Sell"`      |
//! | `Action`              | `Other`     | `"Other"`     |
//! | `Side`                | `Long`      | `"Long"`      |
//! | `Side`                | `Short`     | `"Short"`     |
//! | `OptionStyle`         | `Call`      | `"Call"`      |
//! | `OptionStyle`         | `Put`       | `"Put"`       |
//!
//! ## `no_std`
//!
//! The crate compiles in `no_std` environments. Disable default features:
//!
//! ```toml
//! [dependencies]
//! financial_types = { version = "0.1", default-features = false }
//! ```
//!
//! The `alloc` crate is always required (used by `ParseEnumError`).
//! Re-enable the `std` feature to opt into `std::error::Error`-style
//! integration that pulls in `serde/std`.
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

extern crate alloc;

pub mod prelude;

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

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

impl core::error::Error for ParseEnumError {}

/// Classification of the underlying asset for a financial instrument.
///
/// Represents the broad asset class to which an instrument belongs.
/// Used for routing, risk bucketing, and display purposes.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`. New asset classes may be added in
/// minor releases without a major version bump. Downstream `match`
/// expressions must include a wildcard arm (`_ =>`).
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
#[non_exhaustive]
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
/// # Stability
///
/// This enum is `#[non_exhaustive]`. New trading actions may be added in
/// minor releases without a major version bump. Downstream `match`
/// expressions must include a wildcard arm (`_ =>`).
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
#[non_exhaustive]
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
/// # Stability
///
/// This enum is intentionally **exhaustive**. Position directionality is
/// a closed two-state concept (long vs. short); no future variants are
/// planned. Downstream code may rely on exhaustive `match`.
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
/// # Stability
///
/// This enum is intentionally **exhaustive**. Option payoff direction is
/// a closed two-state concept (call vs. put); no future variants are
/// planned. Downstream code may rely on exhaustive `match`.
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
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

#[cfg(feature = "proptest")]
mod proptest_support {
    use super::{Action, OptionStyle, Side, UnderlyingAssetType};
    use proptest::prelude::*;

    impl Arbitrary for UnderlyingAssetType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Self::Crypto),
                Just(Self::Stock),
                Just(Self::Forex),
                Just(Self::Commodity),
                Just(Self::Bond),
                Just(Self::Other),
            ]
            .boxed()
        }
    }

    impl Arbitrary for Action {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![Just(Self::Buy), Just(Self::Sell), Just(Self::Other)].boxed()
        }
    }

    impl Arbitrary for Side {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![Just(Self::Long), Just(Self::Short)].boxed()
        }
    }

    impl Arbitrary for OptionStyle {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![Just(Self::Call), Just(Self::Put)].boxed()
        }
    }
}
