//! Instrument specifications and their validation.

use alloc::vec::Vec;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::asset::{AssetCode, InstrumentId};
use super::error::InstrumentError;
use super::exact::is_multiple;
use crate::OptionStyle;

/// What a contract is.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum ContractKind {
    /// An exchange of the underlying against the price asset.
    Spot = 0,
    /// A future with an expiry.
    Future = 1,
    /// A perpetual swap, without expiry.
    Perpetual = 2,
    /// An option with an expiry.
    Option = 3,
}

/// How a contract's value relates to the price.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum Margining {
    /// Settled in the price asset; value proportional to the price.
    Linear = 0,
    /// Settled in the underlying; value proportional to the inverse of the
    /// price (futures and perpetuals) or quoted in the underlying (options).
    Inverse = 1,
}

/// What one contract is made of.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum SizeKind {
    /// A quantity of the underlying per contract.
    BaseQuantity = 0,
    /// A value in the price asset per contract (an inverse contract value).
    QuoteValue = 1,
}

/// When an option may be exercised.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum ExerciseStyle {
    /// At expiry only.
    European = 0,
    /// At any time up to expiry. Representable; no formula supports it.
    American = 1,
}

/// How an option settles.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum SettlementType {
    /// The intrinsic value is paid in the settlement asset.
    Cash = 0,
    /// The underlying is delivered against the strike.
    Physical = 1,
}

pub(crate) fn positive(field: &'static str, value: Decimal) -> Result<Decimal, InstrumentError> {
    if value > Decimal::ZERO {
        Ok(value)
    } else {
        Err(InstrumentError::not_positive(field, value))
    }
}

/// The size of one contract: a quantity of the underlying, or a value in
/// the price asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ContractSizeRepr", into = "ContractSizeRepr")]
pub struct ContractSize {
    kind: SizeKind,
    amount: Decimal,
    asset: AssetCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContractSizeRepr {
    kind: SizeKind,
    amount: Decimal,
    asset: AssetCode,
}

impl TryFrom<ContractSizeRepr> for ContractSize {
    type Error = InstrumentError;

    fn try_from(r: ContractSizeRepr) -> Result<Self, Self::Error> {
        Self::new(r.kind, r.amount, r.asset)
    }
}

impl From<ContractSize> for ContractSizeRepr {
    fn from(s: ContractSize) -> Self {
        Self {
            kind: s.kind,
            amount: s.amount,
            asset: s.asset,
        }
    }
}

impl ContractSize {
    /// A contract size.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `amount > 0`.
    pub fn new(kind: SizeKind, amount: Decimal, asset: AssetCode) -> Result<Self, InstrumentError> {
        Ok(Self {
            kind,
            amount: positive("contract size", amount)?,
            asset,
        })
    }

    /// `amount` of the underlying `asset` per contract.
    ///
    /// # Errors
    ///
    /// As [`ContractSize::new`].
    pub fn base(amount: Decimal, asset: AssetCode) -> Result<Self, InstrumentError> {
        Self::new(SizeKind::BaseQuantity, amount, asset)
    }

    /// A value of `amount` in the price `asset` per contract.
    ///
    /// # Errors
    ///
    /// As [`ContractSize::new`].
    pub fn quote_value(amount: Decimal, asset: AssetCode) -> Result<Self, InstrumentError> {
        Self::new(SizeKind::QuoteValue, amount, asset)
    }

    /// What the size is.
    #[must_use]
    #[inline]
    pub fn kind(&self) -> SizeKind {
        self.kind
    }

    /// The amount per contract.
    #[must_use]
    #[inline]
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// The asset of the amount.
    #[must_use]
    #[inline]
    pub fn asset(&self) -> &AssetCode {
        &self.asset
    }
}

/// One band of a tick schedule: prices up to `up_to` (inclusive), or every
/// price above the previous band when `up_to` is `None`, move by `tick`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickBand {
    /// The inclusive upper bound; `None` for the last, unbounded band.
    pub up_to: Option<Decimal>,
    /// The price step in the band.
    pub tick: Decimal,
}

/// The price steps of a contract, by price band.
///
/// Bands are ordered by strictly increasing inclusive upper bound, and only
/// the last one is unbounded. The tick of a price is the one of the first
/// band whose bound is at or above it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<TickBand>", into = "Vec<TickBand>")]
pub struct TickSchedule(Vec<TickBand>);

impl TryFrom<Vec<TickBand>> for TickSchedule {
    type Error = InstrumentError;

    fn try_from(bands: Vec<TickBand>) -> Result<Self, Self::Error> {
        Self::new(bands)
    }
}

impl From<TickSchedule> for Vec<TickBand> {
    fn from(s: TickSchedule) -> Self {
        s.0
    }
}

impl TickSchedule {
    /// A schedule from its bands.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidField`] if there is no band, a bound is
    /// missing before the last band or present on it, or bounds do not
    /// increase strictly; [`InstrumentError::NotPositive`] for a tick or a
    /// bound that is not positive.
    pub fn new(bands: Vec<TickBand>) -> Result<Self, InstrumentError> {
        let invalid = |reason: &str| InstrumentError::invalid_field("tick schedule", reason);
        let last = bands
            .len()
            .checked_sub(1)
            .ok_or_else(|| invalid("no band"))?;
        let mut previous: Option<Decimal> = None;
        for (i, band) in bands.iter().enumerate() {
            positive("tick", band.tick)?;
            match (band.up_to, i == last) {
                (None, true) => {}
                (Some(_), true) => return Err(invalid("the last band must be unbounded")),
                (None, false) => return Err(invalid("only the last band may be unbounded")),
                (Some(bound), false) => {
                    positive("tick band bound", bound)?;
                    if previous.is_some_and(|p| bound <= p) {
                        return Err(invalid("band bounds must increase strictly"));
                    }
                    previous = Some(bound);
                }
            }
        }
        Ok(Self(bands))
    }

    /// One tick for every price.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `tick > 0`.
    pub fn fixed(tick: Decimal) -> Result<Self, InstrumentError> {
        Self::new(alloc::vec![TickBand { up_to: None, tick }])
    }

    /// The bands.
    #[must_use]
    #[inline]
    pub fn bands(&self) -> &[TickBand] {
        &self.0
    }

    /// The tick of `price`.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `price > 0`.
    pub fn tick_for(&self, price: Decimal) -> Result<Decimal, InstrumentError> {
        positive("price", price)?;
        self.0
            .iter()
            .find(|band| band.up_to.is_none_or(|bound| price <= bound))
            .map(|band| band.tick)
            .ok_or_else(|| InstrumentError::invalid_field("tick schedule", "no band"))
    }
}

/// The terms of an option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "OptionTermsRepr", into = "OptionTermsRepr")]
pub struct OptionTerms {
    strike: Decimal,
    style: OptionStyle,
    exercise: ExerciseStyle,
    settlement: SettlementType,
    premium_asset: AssetCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OptionTermsRepr {
    strike: Decimal,
    style: OptionStyle,
    exercise: ExerciseStyle,
    settlement: SettlementType,
    premium_asset: AssetCode,
}

impl TryFrom<OptionTermsRepr> for OptionTerms {
    type Error = InstrumentError;

    fn try_from(r: OptionTermsRepr) -> Result<Self, Self::Error> {
        Self::new(r.strike, r.style, r.exercise, r.settlement, r.premium_asset)
    }
}

impl From<OptionTerms> for OptionTermsRepr {
    fn from(t: OptionTerms) -> Self {
        Self {
            strike: t.strike,
            style: t.style,
            exercise: t.exercise,
            settlement: t.settlement,
            premium_asset: t.premium_asset,
        }
    }
}

impl OptionTerms {
    /// Option terms. The strike is in the instrument's price asset; the
    /// premium is quoted in `premium_asset` per unit of the underlying.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `strike > 0`.
    pub fn new(
        strike: Decimal,
        style: OptionStyle,
        exercise: ExerciseStyle,
        settlement: SettlementType,
        premium_asset: AssetCode,
    ) -> Result<Self, InstrumentError> {
        Ok(Self {
            strike: positive("strike", strike)?,
            style,
            exercise,
            settlement,
            premium_asset,
        })
    }

    /// The strike, in the instrument's price asset.
    #[must_use]
    #[inline]
    pub fn strike(&self) -> Decimal {
        self.strike
    }

    /// Call or put.
    #[must_use]
    #[inline]
    pub fn style(&self) -> OptionStyle {
        self.style
    }

    /// When it may be exercised.
    #[must_use]
    #[inline]
    pub fn exercise(&self) -> ExerciseStyle {
        self.exercise
    }

    /// How it settles.
    #[must_use]
    #[inline]
    pub fn settlement(&self) -> SettlementType {
        self.settlement
    }

    /// The asset the premium is quoted in.
    #[must_use]
    #[inline]
    pub fn premium_asset(&self) -> &AssetCode {
        &self.premium_asset
    }
}

/// The fields of an [`InstrumentSpec`] before validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstrumentDraft {
    /// Identifier.
    pub id: InstrumentId,
    /// What the contract is.
    pub kind: ContractKind,
    /// Linear or inverse.
    pub margining: Margining,
    /// The underlying asset.
    pub underlying: AssetCode,
    /// The asset prices, marks, strikes and settlement prices are in.
    pub price_asset: AssetCode,
    /// The asset cashflows are paid in.
    pub settlement_asset: AssetCode,
    /// The size of one contract.
    pub size: ContractSize,
    /// Order price steps (the premium, for an option).
    pub price_ticks: TickSchedule,
    /// Order size step, in contracts.
    pub lot: Decimal,
    /// Expiry, milliseconds since the Unix epoch; futures and options only.
    pub expiry_ms: Option<u64>,
    /// Option terms; options only.
    pub option: Option<OptionTerms>,
}

/// A validated contract specification.
///
/// Built only by [`InstrumentSpec::new`], directly or through serde, which
/// checks the admissible combinations:
///
/// | kind | margining | size | settled in | expiry | option terms |
/// |---|---|---|---|---|---|
/// | spot | linear | 1 underlying | price asset | none | none |
/// | future | linear | underlying quantity | price asset | required | none |
/// | future | inverse | price asset value | underlying | required | none |
/// | perpetual | linear | underlying quantity | price asset | none | none |
/// | perpetual | inverse | price asset value | underlying | none | none |
/// | option | linear | underlying quantity | price asset, premium in price asset | required | required |
/// | option | inverse | underlying quantity | underlying, premium in underlying | required | required |
///
/// The underlying and the price asset must differ. Nothing converts one
/// asset into another: `USD` and `USDT` are different.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "InstrumentDraft", into = "InstrumentDraft")]
pub struct InstrumentSpec(InstrumentDraft);

impl TryFrom<InstrumentDraft> for InstrumentSpec {
    type Error = InstrumentError;

    fn try_from(draft: InstrumentDraft) -> Result<Self, Self::Error> {
        Self::new(draft)
    }
}

impl From<InstrumentSpec> for InstrumentDraft {
    fn from(spec: InstrumentSpec) -> Self {
        spec.0
    }
}

impl InstrumentSpec {
    /// A validated specification.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidCombination`] for a combination outside the
    /// table above, [`InstrumentError::CurrencyMismatch`] for a size,
    /// settlement or premium asset other than the one required, and
    /// [`InstrumentError::NotPositive`] for a lot that is not positive.
    pub fn new(draft: InstrumentDraft) -> Result<Self, InstrumentError> {
        let d = &draft;
        positive("lot", d.lot)?;
        if d.underlying == d.price_asset {
            return Err(InstrumentError::invalid_combination(
                "the underlying and the price asset must differ",
            ));
        }
        let expect = |field: &'static str, expected: &AssetCode, got: &AssetCode| {
            if expected == got {
                Ok(())
            } else {
                Err(InstrumentError::currency_mismatch(field, expected, got))
            }
        };
        let size_kind = |kind: SizeKind| {
            if d.size.kind() == kind {
                Ok(())
            } else {
                Err(InstrumentError::invalid_combination(alloc::format!(
                    "a {:?} {:?} contract is sized as {kind:?}",
                    d.margining,
                    d.kind
                )))
            }
        };
        let expiry = |required: bool| match (d.expiry_ms.is_some(), required) {
            (true, false) => Err(InstrumentError::invalid_combination(alloc::format!(
                "a {:?} has no expiry",
                d.kind
            ))),
            (false, true) => Err(InstrumentError::invalid_combination(alloc::format!(
                "a {:?} needs an expiry",
                d.kind
            ))),
            _ => Ok(()),
        };
        let no_option = || {
            if d.option.is_some() {
                Err(InstrumentError::invalid_combination(alloc::format!(
                    "a {:?} has no option terms",
                    d.kind
                )))
            } else {
                Ok(())
            }
        };
        match (d.kind, d.margining) {
            (ContractKind::Spot, Margining::Linear) => {
                expiry(false)?;
                no_option()?;
                size_kind(SizeKind::BaseQuantity)?;
                expect("size asset", &d.underlying, d.size.asset())?;
                if d.size.amount() != Decimal::ONE {
                    return Err(InstrumentError::invalid_combination(
                        "a spot contract is one unit of the underlying",
                    ));
                }
                expect("settlement asset", &d.price_asset, &d.settlement_asset)?;
            }
            (ContractKind::Future | ContractKind::Perpetual, margining) => {
                expiry(d.kind == ContractKind::Future)?;
                no_option()?;
                match margining {
                    Margining::Linear => {
                        size_kind(SizeKind::BaseQuantity)?;
                        expect("size asset", &d.underlying, d.size.asset())?;
                        expect("settlement asset", &d.price_asset, &d.settlement_asset)?;
                    }
                    Margining::Inverse => {
                        size_kind(SizeKind::QuoteValue)?;
                        expect("size asset", &d.price_asset, d.size.asset())?;
                        expect("settlement asset", &d.underlying, &d.settlement_asset)?;
                    }
                }
            }
            (ContractKind::Option, margining) => {
                expiry(true)?;
                let terms = d.option.as_ref().ok_or_else(|| {
                    InstrumentError::invalid_combination("an option needs option terms")
                })?;
                size_kind(SizeKind::BaseQuantity)?;
                expect("size asset", &d.underlying, d.size.asset())?;
                let paid_in = match margining {
                    Margining::Linear => &d.price_asset,
                    Margining::Inverse => &d.underlying,
                };
                expect("settlement asset", paid_in, &d.settlement_asset)?;
                expect("premium asset", paid_in, terms.premium_asset())?;
            }
            (kind, margining) => {
                return Err(InstrumentError::invalid_combination(alloc::format!(
                    "a {margining:?} {kind:?} is not supported"
                )));
            }
        }
        Ok(Self(draft))
    }

    /// The fields, validated.
    #[must_use]
    #[inline]
    pub fn draft(&self) -> &InstrumentDraft {
        &self.0
    }

    /// Identifier.
    #[must_use]
    #[inline]
    pub fn id(&self) -> &InstrumentId {
        &self.0.id
    }

    /// What the contract is.
    #[must_use]
    #[inline]
    pub fn kind(&self) -> ContractKind {
        self.0.kind
    }

    /// Linear or inverse.
    #[must_use]
    #[inline]
    pub fn margining(&self) -> Margining {
        self.0.margining
    }

    /// The underlying asset.
    #[must_use]
    #[inline]
    pub fn underlying(&self) -> &AssetCode {
        &self.0.underlying
    }

    /// The asset prices, strikes and settlement prices are in.
    #[must_use]
    #[inline]
    pub fn price_asset(&self) -> &AssetCode {
        &self.0.price_asset
    }

    /// The asset cashflows are paid in.
    #[must_use]
    #[inline]
    pub fn settlement_asset(&self) -> &AssetCode {
        &self.0.settlement_asset
    }

    /// The size of one contract.
    #[must_use]
    #[inline]
    pub fn size(&self) -> &ContractSize {
        &self.0.size
    }

    /// Order price steps.
    #[must_use]
    #[inline]
    pub fn price_ticks(&self) -> &TickSchedule {
        &self.0.price_ticks
    }

    /// Order size step, in contracts.
    #[must_use]
    #[inline]
    pub fn lot(&self) -> Decimal {
        self.0.lot
    }

    /// Expiry, milliseconds since the Unix epoch.
    #[must_use]
    #[inline]
    pub fn expiry_ms(&self) -> Option<u64> {
        self.0.expiry_ms
    }

    /// Option terms.
    #[must_use]
    #[inline]
    pub fn option(&self) -> Option<&OptionTerms> {
        self.0.option.as_ref()
    }

    /// Checks an order price against the tick schedule, exactly.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `price > 0`,
    /// [`InstrumentError::NotOnGrid`] off its tick, and
    /// [`InstrumentError::Overflow`] if the exact check leaves `i128`.
    pub fn check_price(&self, price: Decimal) -> Result<(), InstrumentError> {
        let tick = self.0.price_ticks.tick_for(price)?;
        if is_multiple("price check", price, tick)? {
            Ok(())
        } else {
            Err(InstrumentError::not_on_grid("price", price, tick))
        }
    }

    /// Checks an order size, in contracts, against the lot, exactly.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::NotPositive`] unless `contracts > 0`,
    /// [`InstrumentError::NotOnGrid`] off the lot, and
    /// [`InstrumentError::Overflow`] if the exact check leaves `i128`.
    pub fn check_lot(&self, contracts: Decimal) -> Result<(), InstrumentError> {
        positive("contracts", contracts)?;
        if is_multiple("lot check", contracts, self.0.lot)? {
            Ok(())
        } else {
            Err(InstrumentError::not_on_grid(
                "contracts",
                contracts,
                self.0.lot,
            ))
        }
    }
}
