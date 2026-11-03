//! Notional, PnL, option premium and option expiry, exactly.
//!
//! # Sign convention
//!
//! Every cashflow is signed from the holder's point of view: positive is
//! received, negative is paid. [`Side::Long`] holds or bought the contract,
//! [`Side::Short`] sold it. A long option pays its premium and receives its
//! intrinsic value at expiry; a long future gains when the price rises.
//!
//! # Exactness
//!
//! Every product and difference is exact (see the module's exact
//! arithmetic), and a result a `Decimal` cannot hold is an error. Amounts
//! that need not terminate in decimal (inverse contracts) are exact
//! [`MoneyRatio`]s, rounded only by an explicit [`MoneyRatio::settle`].

use rust_decimal::Decimal;

use super::error::InstrumentError;
use super::exact::{exact_mul, exact_sub};
use super::money::{Money, MoneyRatio, Ratio};
use super::spec::{
    ContractKind, ExerciseStyle, InstrumentSpec, Margining, SettlementType, SizeKind, positive,
};
use crate::{OptionStyle, Side};

/// The outcome of an option at expiry, for one position.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExpiryOutcome {
    /// A cash settlement of the intrinsic value (zero at or out of the
    /// money).
    Cash(MoneyRatio),
    /// A physical delivery, in the money only: the underlying received
    /// (positive) or delivered (negative), against the strike paid or
    /// received.
    Delivery {
        /// The underlying leg.
        underlying: Money,
        /// The strike leg, in the price asset.
        cash: Money,
    },
    /// A physically settled option at or out of the money: nothing is
    /// exchanged.
    Abandoned,
}

/// `+value` for a long position, `-value` for a short one.
#[inline]
fn signed(side: Side, value: Decimal) -> Decimal {
    match side {
        Side::Long => value,
        Side::Short => -value,
    }
}

impl InstrumentSpec {
    /// `contracts` times the contract size: a quantity of the underlying, or
    /// a value in the price asset.
    fn units(&self, op: &'static str, contracts: Decimal) -> Result<Decimal, InstrumentError> {
        positive("contracts", contracts)?;
        exact_mul(op, contracts, self.size().amount())
    }

    /// The notional of `contracts` at `price`: `c·m·p` in the price asset
    /// for a contract sized in the underlying, `c·CV/p` in the underlying
    /// for an inverse contract sized in value.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] for contracts or a price that are
    /// not positive, and [`InstrumentError::Overflow`] or
    /// [`InstrumentError::Inexact`] when an exact product does not fit.
    pub fn notional(
        &self,
        contracts: Decimal,
        price: Decimal,
    ) -> Result<MoneyRatio, InstrumentError> {
        const OP: &str = "notional";
        positive("price", price)?;
        let units = self.units(OP, contracts)?;
        Ok(match self.size().kind() {
            SizeKind::BaseQuantity => MoneyRatio::new(
                Ratio::from_decimal(exact_mul(OP, units, price)?),
                self.price_asset().clone(),
            ),
            SizeKind::QuoteValue => {
                MoneyRatio::new(Ratio::new(units, price)?, self.underlying().clone())
            }
        })
    }

    /// The PnL of closing `contracts` opened at `entry` at `exit`, in the
    /// settlement asset (the premium asset for an option, whose prices are
    /// premiums).
    ///
    /// - Spot, linear future or perpetual: `±c·m·(exit - entry)`.
    /// - Inverse future or perpetual: `±c·CV·(exit - entry) / (entry·exit)`.
    /// - Option: `±c·m·(exit - entry)` on the premium.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] for contracts or prices that are not
    /// positive, and [`InstrumentError::Overflow`] or
    /// [`InstrumentError::Inexact`] when an exact product does not fit.
    pub fn realized_pnl(
        &self,
        side: Side,
        contracts: Decimal,
        entry: Decimal,
        exit: Decimal,
    ) -> Result<MoneyRatio, InstrumentError> {
        const OP: &str = "realized pnl";
        positive("entry price", entry)?;
        positive("exit price", exit)?;
        let units = self.units(OP, contracts)?;
        let change = exact_sub(OP, exit, entry)?;
        let num = signed(side, exact_mul(OP, units, change)?);
        Ok(match (self.kind(), self.size().kind()) {
            (ContractKind::Option, _) => {
                let premium_asset = self
                    .option()
                    .map(|t| t.premium_asset().clone())
                    .ok_or_else(|| {
                        InstrumentError::invalid_combination("an option needs option terms")
                    })?;
                MoneyRatio::new(Ratio::from_decimal(num), premium_asset)
            }
            (_, SizeKind::BaseQuantity) => {
                MoneyRatio::new(Ratio::from_decimal(num), self.settlement_asset().clone())
            }
            (_, SizeKind::QuoteValue) => MoneyRatio::new(
                Ratio::new(num, exact_mul(OP, entry, exit)?)?,
                self.settlement_asset().clone(),
            ),
        })
    }

    /// The premium cashflow of `contracts` of an option at `premium` per
    /// unit of the underlying: `-c·m·premium` for a long, `+` for a short,
    /// in the premium asset.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::Unsupported`] for a contract that is not an
    /// option, [`InstrumentError::Negative`] for a negative premium,
    /// [`InstrumentError::NotPositive`] for contracts, and
    /// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`].
    pub fn option_premium(
        &self,
        side: Side,
        contracts: Decimal,
        premium: Decimal,
    ) -> Result<Money, InstrumentError> {
        const OP: &str = "option premium";
        let terms = self.option().ok_or_else(|| {
            InstrumentError::unsupported("a premium of a contract that is not an option")
        })?;
        if premium < Decimal::ZERO {
            return Err(InstrumentError::negative("premium", premium));
        }
        let units = self.units(OP, contracts)?;
        let paid = exact_mul(OP, units, premium)?;
        Ok(Money::new(
            -signed(side, paid),
            terms.premium_asset().clone(),
        ))
    }

    /// The outcome at expiry of `contracts` of a European option, at the
    /// venue's settlement price `settlement_price` (its delivery price, in
    /// the price asset; never a mark).
    ///
    /// - Cash, linear: `±c·m·intrinsic` in the price asset.
    /// - Cash, inverse: `±c·m·intrinsic / S` in the underlying. This is the
    ///   net cashflow; how the venue books it (for instance through an
    ///   expiry future that settles at once) is not modelled.
    /// - Physical, linear, in the money: a long call receives `c·m` of the
    ///   underlying and pays `c·m·K`; a long put delivers `c·m` and receives
    ///   `c·m·K`; a short takes the opposite signs. At or out of the money,
    ///   [`ExpiryOutcome::Abandoned`].
    ///
    /// The intrinsic value is `max(S - K, 0)` for a call and `max(K - S, 0)`
    /// for a put.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::Unsupported`] for a contract that is not an
    /// option, American exercise, or physical settlement of an inverse
    /// option; [`InstrumentError::NotPositive`] for contracts or a
    /// settlement price that are not positive; and
    /// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`].
    pub fn option_expiry(
        &self,
        side: Side,
        contracts: Decimal,
        settlement_price: Decimal,
    ) -> Result<ExpiryOutcome, InstrumentError> {
        const OP: &str = "option expiry";
        let terms = self.option().ok_or_else(|| {
            InstrumentError::unsupported("the expiry of a contract that is not an option")
        })?;
        if terms.exercise() != ExerciseStyle::European {
            return Err(InstrumentError::unsupported("exercise other than European"));
        }
        positive("settlement price", settlement_price)?;
        let units = self.units(OP, contracts)?;
        let strike = terms.strike();
        let intrinsic = match terms.style() {
            OptionStyle::Call => exact_sub(OP, settlement_price, strike)?,
            OptionStyle::Put => exact_sub(OP, strike, settlement_price)?,
        }
        .max(Decimal::ZERO);
        match (terms.settlement(), self.margining()) {
            (SettlementType::Cash, Margining::Linear) => {
                let num = signed(side, exact_mul(OP, units, intrinsic)?);
                Ok(ExpiryOutcome::Cash(MoneyRatio::new(
                    Ratio::from_decimal(num),
                    self.settlement_asset().clone(),
                )))
            }
            (SettlementType::Cash, Margining::Inverse) => {
                let num = signed(side, exact_mul(OP, units, intrinsic)?);
                Ok(ExpiryOutcome::Cash(MoneyRatio::new(
                    Ratio::new(num, settlement_price)?,
                    self.settlement_asset().clone(),
                )))
            }
            (SettlementType::Physical, Margining::Linear) => {
                if intrinsic.is_zero() {
                    return Ok(ExpiryOutcome::Abandoned);
                }
                let strike_cash = exact_mul(OP, units, strike)?;
                // A long call buys the underlying at the strike; a long put
                // sells it.
                let (underlying, cash) = match terms.style() {
                    OptionStyle::Call => (units, -strike_cash),
                    OptionStyle::Put => (-units, strike_cash),
                };
                Ok(ExpiryOutcome::Delivery {
                    underlying: Money::new(signed(side, underlying), self.underlying().clone()),
                    cash: Money::new(signed(side, cash), self.price_asset().clone()),
                })
            }
            (SettlementType::Physical, _) => Err(InstrumentError::unsupported(
                "physical settlement of an inverse option",
            )),
        }
    }
}
