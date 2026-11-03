//! Validated asset codes and instrument identifiers.

use alloc::string::String;
use core::fmt;

use serde::{Deserialize, Serialize};

use super::error::InstrumentError;

/// The longest asset code.
pub const MAX_ASSET_CODE_LEN: usize = 16;

/// The longest instrument identifier.
pub const MAX_INSTRUMENT_ID_LEN: usize = 64;

/// An asset code such as `BTC`, `USD` or `USDT`: 1 to 16 ASCII uppercase
/// letters or digits.
///
/// Codes are compared exactly: `USD` and `USDT` are different assets, and
/// nothing converts one into the other.
///
/// # Examples
///
/// ```rust
/// use financial_types::instrument::AssetCode;
///
/// let btc = AssetCode::new("BTC").unwrap();
/// assert_eq!(btc.as_str(), "BTC");
/// assert!(AssetCode::new("btc").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AssetCode(String);

impl AssetCode {
    /// A validated asset code.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidField`] unless `code` is 1 to 16 ASCII
    /// uppercase letters or digits.
    pub fn new(code: &str) -> Result<Self, InstrumentError> {
        let valid = !code.is_empty()
            && code.len() <= MAX_ASSET_CODE_LEN
            && code
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
        if valid {
            Ok(Self(String::from(code)))
        } else {
            Err(InstrumentError::invalid_field(
                "asset code",
                alloc::format!("{code:?} is not 1 to 16 ASCII uppercase letters or digits"),
            ))
        }
    }

    /// The code.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AssetCode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for AssetCode {
    type Error = InstrumentError;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        Self::new(&code)
    }
}

impl From<AssetCode> for String {
    #[inline]
    fn from(code: AssetCode) -> Self {
        code.0
    }
}

/// An instrument identifier such as `BTC-PERPETUAL`: 1 to 64 printable
/// ASCII characters, without spaces.
///
/// # Examples
///
/// ```rust
/// use financial_types::instrument::InstrumentId;
///
/// assert!(InstrumentId::new("BTC-27NOV26").is_ok());
/// assert!(InstrumentId::new("BTC 27NOV26").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct InstrumentId(String);

impl InstrumentId {
    /// A validated identifier.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidField`] unless `id` is 1 to 64 printable
    /// ASCII characters without spaces.
    pub fn new(id: &str) -> Result<Self, InstrumentError> {
        let valid = !id.is_empty()
            && id.len() <= MAX_INSTRUMENT_ID_LEN
            && id.bytes().all(|b| b.is_ascii_graphic());
        if valid {
            Ok(Self(String::from(id)))
        } else {
            Err(InstrumentError::invalid_field(
                "instrument id",
                alloc::format!("{id:?} is not 1 to 64 printable ASCII characters"),
            ))
        }
    }

    /// The identifier.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstrumentId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for InstrumentId {
    type Error = InstrumentError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(&id)
    }
}

impl From<InstrumentId> for String {
    #[inline]
    fn from(id: InstrumentId) -> Self {
        id.0
    }
}
