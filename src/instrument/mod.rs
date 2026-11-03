//! Instrument specifications and exact cashflow formulas (feature
//! `instruments`).
//!
//! An [`InstrumentSpec`] describes one tradeable contract: spot, a dated
//! future, a perpetual or an option, linear or inverse, with its assets, its
//! contract size, its price and size grids, its expiry and, for an option,
//! its terms. It is validated on construction, also when deserialized, and
//! the formulas on it compute notional, realized PnL, option premiums and
//! option expiry outcomes.
//!
//! # Exactness
//!
//! Nothing here rounds implicitly. Products and differences are computed on
//! the integer mantissas of the decimals, and a result a `Decimal` cannot
//! hold exactly is an error ([`InstrumentError::Inexact`] or
//! [`InstrumentError::Overflow`]), never a rounded value. Amounts that need
//! not terminate in decimal, such as a contract value divided by a price,
//! are exact rationals ([`Ratio`], [`MoneyRatio`]); the only rounding is the
//! explicit [`MoneyRatio::settle`] at a chosen scale and [`RoundingRule`],
//! which also returns the exact residual.
//!
//! # Signs
//!
//! Cashflows are signed from the holder's point of view: positive is
//! received, negative is paid (see the formulas). Positions are given as a
//! [`Side`](crate::Side); an [`Action`](crate::Action) converts to and from
//! it here.
//!
//! # Reference contracts
//!
//! [`reference`](mod@reference) holds dated snapshots of Deribit BTC options, futures and
//! the perpetual, and of the Binance USDⓈ-M `BTCUSDT` perpetual.
//!
//! # Example
//!
//! ```rust
//! use financial_types::Side;
//! use financial_types::instrument::{RoundingRule, reference::deribit};
//! use rust_decimal::Decimal;
//!
//! let perp = deribit::btc_perpetual().unwrap();
//! // Long 1000 contracts (10 000 USD) from 60 000 to 61 000 USD.
//! let pnl = perp
//!     .realized_pnl(Side::Long, Decimal::from(1000), Decimal::from(60_000), Decimal::from(61_000))
//!     .unwrap();
//! // Exactly 1/366 BTC, settled at satoshi scale.
//! let settled = pnl.settle(8, RoundingRule::HalfEven).unwrap();
//! assert_eq!(settled.amount.amount, Decimal::new(273_224, 8));
//! assert_eq!(settled.amount.asset.as_str(), "BTC");
//! ```

mod asset;
mod direction;
mod error;
mod exact;
mod formulas;
mod money;
pub mod reference;
mod serde_decimal;
mod spec;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use asset::{AssetCode, InstrumentId, MAX_ASSET_CODE_LEN, MAX_INSTRUMENT_ID_LEN};
pub use error::InstrumentError;
pub use formulas::ExpiryOutcome;
pub use money::{Money, MoneyRatio, Ratio, Rounded, RoundingRule};
pub use spec::{
    ContractKind, ContractSize, ExerciseStyle, InstrumentDraft, InstrumentSpec, Margining,
    OptionTerms, SettlementType, SizeKind, TickBand, TickSchedule,
};
