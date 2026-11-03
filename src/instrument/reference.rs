//! Reference contract specifications, as dated snapshots.
//!
//! Each value below was read from the venue's public data on 2026-11-03 and
//! is a snapshot, not a claim about the live market: venues change their
//! contract terms. The constructors validate through
//! [`InstrumentSpec::new`] like any other specification.

use rust_decimal::Decimal;

use super::asset::{AssetCode, InstrumentId};
use super::error::InstrumentError;
use super::spec::{
    ContractKind, ContractSize, ExerciseStyle, InstrumentDraft, InstrumentSpec, Margining,
    OptionTerms, SettlementType, TickBand, TickSchedule,
};
use crate::OptionStyle;

fn asset(code: &str) -> Result<AssetCode, InstrumentError> {
    AssetCode::new(code)
}

/// Deribit BTC contracts.
///
/// Sources, read 2026-11-03 (14:34 UTC):
/// - `https://www.deribit.com/api/v2/public/get_instruments?currency=BTC&kind=option`
///   and `...&kind=future`: `contract_size`, `min_trade_amount`,
///   `tick_size`, `tick_size_steps`, `settlement_currency`,
///   `counter_currency`;
/// - `https://support.deribit.com/hc/en-us/articles/31424939096093-Inverse-Options`,
///   `.../31424938981533-Inverse-Futures` and `.../31424954847133`
///   (inverse perpetual).
pub mod deribit {
    use super::*;

    /// A BTC option: European, cash settled, 1 BTC per contract, the
    /// premium quoted in BTC per BTC and settled in BTC, the strike and the
    /// settlement price in USD. Orders move by 0.1 contracts; the premium
    /// tick is 0.0001 BTC up to 0.005 BTC inclusive and 0.0005 BTC above.
    ///
    /// Since 2026-08-01 Deribit settles an in the money option through an
    /// expiry future that settles at once in BTC, with the same net payoff;
    /// [`InstrumentSpec::option_expiry`] gives that net cashflow.
    ///
    /// # Errors
    ///
    /// As [`InstrumentSpec::new`] and [`OptionTerms::new`].
    pub fn btc_option(
        id: InstrumentId,
        expiry_ms: u64,
        strike: Decimal,
        style: OptionStyle,
    ) -> Result<InstrumentSpec, InstrumentError> {
        let btc = asset("BTC")?;
        InstrumentSpec::new(InstrumentDraft {
            id,
            kind: ContractKind::Option,
            margining: Margining::Inverse,
            underlying: btc.clone(),
            price_asset: asset("USD")?,
            settlement_asset: btc.clone(),
            size: ContractSize::base(Decimal::ONE, btc.clone())?,
            price_ticks: TickSchedule::new(alloc::vec![
                TickBand {
                    up_to: Some(Decimal::new(5, 3)),
                    tick: Decimal::new(1, 4),
                },
                TickBand {
                    up_to: None,
                    tick: Decimal::new(5, 4),
                },
            ])?,
            lot: Decimal::new(1, 1),
            expiry_ms: Some(expiry_ms),
            option: Some(OptionTerms::new(
                strike,
                style,
                ExerciseStyle::European,
                SettlementType::Cash,
                btc,
            )?),
        })
    }

    fn inverse(
        id: InstrumentId,
        kind: ContractKind,
        tick: Decimal,
        expiry_ms: Option<u64>,
    ) -> Result<InstrumentSpec, InstrumentError> {
        let usd = asset("USD")?;
        InstrumentSpec::new(InstrumentDraft {
            id,
            kind,
            margining: Margining::Inverse,
            underlying: asset("BTC")?,
            price_asset: usd.clone(),
            settlement_asset: asset("BTC")?,
            size: ContractSize::quote_value(Decimal::TEN, usd)?,
            price_ticks: TickSchedule::fixed(tick)?,
            lot: Decimal::ONE,
            expiry_ms,
            option: None,
        })
    }

    /// A BTC inverse future: 10 USD per contract, settled in BTC, tick
    /// 2.5 USD, orders in whole contracts (multiples of 10 USD).
    ///
    /// # Errors
    ///
    /// As [`InstrumentSpec::new`].
    pub fn btc_future(id: InstrumentId, expiry_ms: u64) -> Result<InstrumentSpec, InstrumentError> {
        inverse(
            id,
            ContractKind::Future,
            Decimal::new(25, 1),
            Some(expiry_ms),
        )
    }

    /// `BTC-PERPETUAL`: 10 USD per contract, settled in BTC, tick 0.5 USD,
    /// orders in whole contracts.
    ///
    /// # Errors
    ///
    /// As [`InstrumentSpec::new`].
    pub fn btc_perpetual() -> Result<InstrumentSpec, InstrumentError> {
        inverse(
            InstrumentId::new("BTC-PERPETUAL")?,
            ContractKind::Perpetual,
            Decimal::new(5, 1),
            None,
        )
    }
}

/// Binance USDⓈ-M (linear) contracts.
///
/// Source, read 2026-11-03: `https://fapi.binance.com/fapi/v1/exchangeInfo`,
/// symbol `BTCUSDT`: `contractType` `PERPETUAL`, `baseAsset` `BTC`,
/// `quoteAsset` and `marginAsset` `USDT`, `PRICE_FILTER.tickSize` `0.10`,
/// `LOT_SIZE.stepSize` `0.001`.
pub mod binance_usdm {
    use super::*;

    /// `BTCUSDT` perpetual: the quantity is in BTC (1 BTC per contract
    /// unit), prices and settlement in USDT, tick 0.1 USDT, step 0.001 BTC.
    ///
    /// # Errors
    ///
    /// As [`InstrumentSpec::new`].
    pub fn btcusdt_perpetual() -> Result<InstrumentSpec, InstrumentError> {
        let (btc, usdt) = (asset("BTC")?, asset("USDT")?);
        InstrumentSpec::new(InstrumentDraft {
            id: InstrumentId::new("BTCUSDT")?,
            kind: ContractKind::Perpetual,
            margining: Margining::Linear,
            underlying: btc.clone(),
            price_asset: usdt.clone(),
            settlement_asset: usdt,
            size: ContractSize::base(Decimal::ONE, btc)?,
            price_ticks: TickSchedule::fixed(Decimal::new(1, 1))?,
            lot: Decimal::new(1, 3),
            expiry_ms: None,
            option: None,
        })
    }
}
