//! Shared, hand computed instrument fixtures (feature `fixtures`).
//!
//! Each [`Case`] is a specification, an [`Operation`] and the [`Expected`]
//! result, computed by hand from the contract terms (shown in each case's
//! name and in the comments) and checked independently with exact
//! fractions, never with this crate's formulas. This crate runs every case
//! against its own implementation ([`check`]); downstream crates run the
//! same cases through their adapters, so no formula can diverge silently.
//!
//! The cases are versioned data: changing, adding or removing one is a
//! change to this crate, recorded in the changelog.
//!
//! The contract terms come from the dated snapshots in
//! [`reference`](mod@super::reference); cases marked "illustrative" use a
//! made-up linear option, as neither reference venue lists one.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use rust_decimal::Decimal;

use super::asset::{AssetCode, InstrumentId};
use super::error::InstrumentError;
use super::formulas::ExpiryOutcome;
use super::money::{Money, MoneyRatio, Ratio, RoundingRule};
use super::reference::{binance_usdm, deribit};
use super::spec::{
    ContractKind, ContractSize, ExerciseStyle, InstrumentDraft, InstrumentSpec, Margining,
    OptionTerms, SettlementType, TickSchedule,
};
use crate::{OptionStyle, Side};

/// The expiry used by the dated cases: 2026-11-27 08:00 UTC.
pub const EXPIRY_MS: u64 = 1_795_766_400_000;

/// What a case computes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Operation {
    /// [`InstrumentSpec::notional`].
    Notional {
        /// Contracts.
        contracts: Decimal,
        /// Price.
        price: Decimal,
    },
    /// [`InstrumentSpec::realized_pnl`].
    RealizedPnl {
        /// Side.
        side: Side,
        /// Contracts.
        contracts: Decimal,
        /// Entry price.
        entry: Decimal,
        /// Exit price.
        exit: Decimal,
    },
    /// [`InstrumentSpec::option_premium`].
    Premium {
        /// Side.
        side: Side,
        /// Contracts.
        contracts: Decimal,
        /// Premium per unit of the underlying.
        premium: Decimal,
    },
    /// [`InstrumentSpec::option_expiry`].
    Expiry {
        /// Side.
        side: Side,
        /// Contracts.
        contracts: Decimal,
        /// The venue's settlement price.
        settlement_price: Decimal,
    },
    /// [`InstrumentSpec::check_price`].
    CheckPrice {
        /// Price.
        price: Decimal,
    },
    /// [`InstrumentSpec::check_lot`].
    CheckLot {
        /// Contracts.
        contracts: Decimal,
    },
    /// [`MoneyRatio::settle`] of a given value (the spec gives the asset).
    Settle {
        /// The value.
        value: Ratio,
        /// Scale.
        scale: u32,
        /// Rule.
        rule: RoundingRule,
    },
}

/// A settled amount and its exact residual.
#[derive(Debug, Clone)]
pub struct Settled {
    /// Scale.
    pub scale: u32,
    /// Rule.
    pub rule: RoundingRule,
    /// The settled amount.
    pub amount: Decimal,
    /// `exact - amount`.
    pub residual: Ratio,
}

/// The category of an expected error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// [`InstrumentError::InvalidField`].
    InvalidField,
    /// [`InstrumentError::InvalidCombination`].
    InvalidCombination,
    /// [`InstrumentError::CurrencyMismatch`].
    CurrencyMismatch,
    /// [`InstrumentError::NotPositive`].
    NotPositive,
    /// [`InstrumentError::Negative`].
    Negative,
    /// [`InstrumentError::NotOnGrid`].
    NotOnGrid,
    /// [`InstrumentError::Inexact`].
    Inexact,
    /// [`InstrumentError::Overflow`].
    Overflow,
    /// [`InstrumentError::Unsupported`].
    Unsupported,
}

impl ErrorKind {
    /// The category of `error`.
    #[must_use]
    pub fn of(error: &InstrumentError) -> Self {
        match error {
            InstrumentError::InvalidField { .. } => Self::InvalidField,
            InstrumentError::InvalidCombination { .. } => Self::InvalidCombination,
            InstrumentError::CurrencyMismatch { .. } => Self::CurrencyMismatch,
            InstrumentError::NotPositive { .. } => Self::NotPositive,
            InstrumentError::Negative { .. } => Self::Negative,
            InstrumentError::NotOnGrid { .. } => Self::NotOnGrid,
            InstrumentError::Inexact { .. } => Self::Inexact,
            InstrumentError::Overflow { .. } => Self::Overflow,
            InstrumentError::Unsupported { .. } => Self::Unsupported,
        }
    }
}

/// The expected result of a case.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Expected {
    /// An exact amount, and optionally its settlement.
    Amount {
        /// The exact amount.
        exact: MoneyRatio,
        /// Its settlement.
        settled: Option<Settled>,
    },
    /// An exact decimal amount (a premium).
    Money(Money),
    /// A physical delivery.
    Delivery {
        /// The underlying leg.
        underlying: Money,
        /// The strike leg.
        cash: Money,
    },
    /// A physical option abandoned at or out of the money.
    Abandoned,
    /// A settlement of [`Operation::Settle`].
    Settled(Settled),
    /// A valid price or size.
    Valid,
    /// An error of this category.
    Error(ErrorKind),
}

/// One fixture.
#[derive(Debug, Clone)]
pub struct Case {
    /// What it checks, with the hand computation.
    pub name: &'static str,
    /// The contract.
    pub spec: InstrumentSpec,
    /// The operation.
    pub operation: Operation,
    /// The expected result.
    pub expected: Expected,
}

fn dec(mantissa: i64, scale: u32) -> Decimal {
    Decimal::new(mantissa, scale)
}

fn int(value: i64) -> Decimal {
    Decimal::from(value)
}

fn ratio(num: i64, den: i64) -> Result<Ratio, InstrumentError> {
    Ratio::new(int(num), int(den))
}

fn asset(code: &str) -> Result<AssetCode, InstrumentError> {
    AssetCode::new(code)
}

fn exact(num: i64, den: i64, code: &str) -> Result<MoneyRatio, InstrumentError> {
    Ok(MoneyRatio::new(ratio(num, den)?, asset(code)?))
}

fn settled(
    scale: u32,
    rule: RoundingRule,
    amount: Decimal,
    residual: (i64, i64),
) -> Result<Settled, InstrumentError> {
    Ok(Settled {
        scale,
        rule,
        amount,
        residual: ratio(residual.0, residual.1)?,
    })
}

/// An illustrative linear BTC option in USDT, strike 100, 1 BTC per
/// contract.
fn linear_option(
    style: OptionStyle,
    settlement: SettlementType,
) -> Result<InstrumentSpec, InstrumentError> {
    let (btc, usdt) = (asset("BTC")?, asset("USDT")?);
    InstrumentSpec::new(InstrumentDraft {
        id: InstrumentId::new("ILLUSTRATIVE-LINEAR-OPTION")?,
        kind: ContractKind::Option,
        margining: Margining::Linear,
        underlying: btc.clone(),
        price_asset: usdt.clone(),
        settlement_asset: usdt.clone(),
        size: ContractSize::base(Decimal::ONE, btc)?,
        price_ticks: TickSchedule::fixed(dec(1, 2))?,
        lot: dec(1, 1),
        expiry_ms: Some(EXPIRY_MS),
        option: Some(OptionTerms::new(
            int(100),
            style,
            ExerciseStyle::European,
            settlement,
            usdt,
        )?),
    })
}

/// Every fixture.
///
/// # Errors
///
/// An [`InstrumentError`] only if a reference specification no longer
/// validates, which is a bug in this crate.
#[allow(clippy::too_many_lines)]
pub fn cases() -> Result<Vec<Case>, InstrumentError> {
    use RoundingRule::*;
    use Side::{Long, Short};
    let binance = binance_usdm::btcusdt_perpetual()?;
    let perp = deribit::btc_perpetual()?;
    let future = deribit::btc_future(InstrumentId::new("BTC-27NOV26")?, EXPIRY_MS)?;
    let call = deribit::btc_option(
        InstrumentId::new("BTC-27NOV26-100000-C")?,
        EXPIRY_MS,
        int(100_000),
        OptionStyle::Call,
    )?;
    let put = deribit::btc_option(
        InstrumentId::new("BTC-27NOV26-100000-P")?,
        EXPIRY_MS,
        int(100_000),
        OptionStyle::Put,
    )?;
    let phys_call = linear_option(OptionStyle::Call, SettlementType::Physical)?;
    let phys_put = linear_option(OptionStyle::Put, SettlementType::Physical)?;
    let cash_call = linear_option(OptionStyle::Call, SettlementType::Cash)?;
    let he8 = |amount: Decimal, res: (i64, i64)| settled(8, HalfEven, amount, res);

    let mut cases = Vec::new();
    let mut add =
        |name: &'static str, spec: &InstrumentSpec, operation: Operation, expected: Expected| {
            cases.push(Case {
                name,
                spec: spec.clone(),
                operation,
                expected,
            });
        };

    // ------------------------------------------------- linear (Binance)
    add(
        "linear notional: 0.5 BTC x 60000.1 USDT = 30000.05 USDT",
        &binance,
        Operation::Notional {
            contracts: dec(5, 1),
            price: dec(600_001, 1),
        },
        Expected::Amount {
            exact: MoneyRatio::new(Ratio::from_decimal(dec(3_000_005, 2)), asset("USDT")?),
            settled: None,
        },
    );
    add(
        "linear pnl long: 0.5 x (61000.5 - 60000) = +500.25 USDT",
        &binance,
        Operation::RealizedPnl {
            side: Long,
            contracts: dec(5, 1),
            entry: int(60_000),
            exit: dec(610_005, 1),
        },
        Expected::Amount {
            exact: MoneyRatio::new(Ratio::from_decimal(dec(50_025, 2)), asset("USDT")?),
            settled: None,
        },
    );
    add(
        "linear pnl short: the same move is -500.25 USDT",
        &binance,
        Operation::RealizedPnl {
            side: Short,
            contracts: dec(5, 1),
            entry: int(60_000),
            exit: dec(610_005, 1),
        },
        Expected::Amount {
            exact: MoneyRatio::new(Ratio::from_decimal(dec(-50_025, 2)), asset("USDT")?),
            settled: None,
        },
    );

    // ------------------------------------------ inverse (Deribit, CV 10)
    add(
        "inverse notional: 1000 x 10 USD / 60000 = 1/6 BTC, 0.16666667 at satoshi",
        &perp,
        Operation::Notional {
            contracts: int(1000),
            price: int(60_000),
        },
        Expected::Amount {
            exact: exact(1, 6, "BTC")?,
            settled: Some(he8(dec(16_666_667, 8), (-1, 300_000_000))?),
        },
    );
    add(
        "inverse pnl long: 10000 x (1/60000 - 1/61000) = 1/366 BTC, 0.00273224",
        &perp,
        Operation::RealizedPnl {
            side: Long,
            contracts: int(1000),
            entry: int(60_000),
            exit: int(61_000),
        },
        Expected::Amount {
            exact: exact(1, 366, "BTC")?,
            settled: Some(he8(dec(273_224, 8), (1, 2_287_500_000))?),
        },
    );
    add(
        "inverse pnl short: -1/366 BTC, -0.00273224",
        &perp,
        Operation::RealizedPnl {
            side: Short,
            contracts: int(1000),
            entry: int(60_000),
            exit: int(61_000),
        },
        Expected::Amount {
            exact: exact(-1, 366, "BTC")?,
            settled: Some(he8(dec(-273_224, 8), (-1, 2_287_500_000))?),
        },
    );
    add(
        "inverse future pnl long: 1000 USD from 60000 to 59000 = -1/3540 BTC, -0.00028249",
        &future,
        Operation::RealizedPnl {
            side: Long,
            contracts: int(100),
            entry: int(60_000),
            exit: int(59_000),
        },
        Expected::Amount {
            exact: exact(-1, 3540, "BTC")?,
            settled: Some(he8(dec(-28_249, 8), (73, 17_700_000_000))?),
        },
    );

    // ------------------------------------- inverse options (Deribit BTC)
    add(
        "option premium long: 1.5 x 1 BTC x 0.0125 = -0.01875 BTC paid",
        &call,
        Operation::Premium {
            side: Long,
            contracts: dec(15, 1),
            premium: dec(125, 4),
        },
        Expected::Money(Money::new(dec(-1875, 5), asset("BTC")?)),
    );
    add(
        "option premium short: +0.01875 BTC received",
        &call,
        Operation::Premium {
            side: Short,
            contracts: dec(15, 1),
            premium: dec(125, 4),
        },
        Expected::Money(Money::new(dec(1875, 5), asset("BTC")?)),
    );
    add(
        "inverse call long, S 110000, K 100000: 2 x 10000 / 110000 = 2/11 BTC",
        &call,
        Operation::Expiry {
            side: Long,
            contracts: int(2),
            settlement_price: int(110_000),
        },
        Expected::Amount {
            exact: exact(2, 11, "BTC")?,
            settled: Some(he8(dec(18_181_818, 8), (1, 550_000_000))?),
        },
    );
    add(
        "inverse call short: -2/11 BTC",
        &call,
        Operation::Expiry {
            side: Short,
            contracts: int(2),
            settlement_price: int(110_000),
        },
        Expected::Amount {
            exact: exact(-2, 11, "BTC")?,
            settled: Some(he8(dec(-18_181_818, 8), (-1, 550_000_000))?),
        },
    );
    add(
        "inverse call at the money: 0",
        &call,
        Operation::Expiry {
            side: Long,
            contracts: int(2),
            settlement_price: int(100_000),
        },
        Expected::Amount {
            exact: exact(0, 1, "BTC")?,
            settled: None,
        },
    );
    add(
        "inverse call out of the money: 0",
        &call,
        Operation::Expiry {
            side: Long,
            contracts: int(2),
            settlement_price: int(90_000),
        },
        Expected::Amount {
            exact: exact(0, 1, "BTC")?,
            settled: None,
        },
    );
    add(
        "inverse put long, S 90000: 2 x 10000 / 90000 = 2/9 BTC",
        &put,
        Operation::Expiry {
            side: Long,
            contracts: int(2),
            settlement_price: int(90_000),
        },
        Expected::Amount {
            exact: exact(2, 9, "BTC")?,
            settled: Some(he8(dec(22_222_222, 8), (1, 450_000_000))?),
        },
    );
    add(
        "inverse put out of the money: 0",
        &put,
        Operation::Expiry {
            side: Short,
            contracts: int(2),
            settlement_price: int(110_000),
        },
        Expected::Amount {
            exact: exact(0, 1, "BTC")?,
            settled: None,
        },
    );

    // --------------------------------------- linear options (illustrative)
    add(
        "illustrative linear cash call long, S 120, K 100: 3 x 20 = +60 USDT",
        &cash_call,
        Operation::Expiry {
            side: Long,
            contracts: int(3),
            settlement_price: int(120),
        },
        Expected::Amount {
            exact: exact(60, 1, "USDT")?,
            settled: None,
        },
    );
    let delivery = |u: i64, c: i64| -> Result<Expected, InstrumentError> {
        Ok(Expected::Delivery {
            underlying: Money::new(int(u), asset("BTC")?),
            cash: Money::new(int(c), asset("USDT")?),
        })
    };
    add(
        "illustrative physical call long, S 120: +3 BTC, -300 USDT",
        &phys_call,
        Operation::Expiry {
            side: Long,
            contracts: int(3),
            settlement_price: int(120),
        },
        delivery(3, -300)?,
    );
    add(
        "illustrative physical call short, S 120: -3 BTC, +300 USDT",
        &phys_call,
        Operation::Expiry {
            side: Short,
            contracts: int(3),
            settlement_price: int(120),
        },
        delivery(-3, 300)?,
    );
    add(
        "illustrative physical put long, S 80: -3 BTC, +300 USDT",
        &phys_put,
        Operation::Expiry {
            side: Long,
            contracts: int(3),
            settlement_price: int(80),
        },
        delivery(-3, 300)?,
    );
    add(
        "illustrative physical put short, S 80: +3 BTC, -300 USDT",
        &phys_put,
        Operation::Expiry {
            side: Short,
            contracts: int(3),
            settlement_price: int(80),
        },
        delivery(3, -300)?,
    );
    add(
        "illustrative physical call out of the money: abandoned",
        &phys_call,
        Operation::Expiry {
            side: Long,
            contracts: int(3),
            settlement_price: int(90),
        },
        Expected::Abandoned,
    );
    add(
        "illustrative physical call at the money: abandoned",
        &phys_call,
        Operation::Expiry {
            side: Short,
            contracts: int(3),
            settlement_price: int(100),
        },
        Expected::Abandoned,
    );
    add(
        "illustrative physical put out of the money: abandoned",
        &phys_put,
        Operation::Expiry {
            side: Long,
            contracts: int(3),
            settlement_price: int(120),
        },
        Expected::Abandoned,
    );

    // ------------------------------------------------------------- grids
    let price = |p: Decimal| Operation::CheckPrice { price: p };
    add(
        "Deribit option premium 0.005: on the 0.0001 band (inclusive)",
        &call,
        price(dec(5, 3)),
        Expected::Valid,
    );
    add(
        "Deribit option premium 0.0049: on 0.0001",
        &call,
        price(dec(49, 4)),
        Expected::Valid,
    );
    add(
        "Deribit option premium 0.00495: off 0.0001",
        &call,
        price(dec(495, 5)),
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Deribit option premium 0.0051: above 0.005, off 0.0005",
        &call,
        price(dec(51, 4)),
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Deribit option premium 0.0055: on 0.0005",
        &call,
        price(dec(55, 4)),
        Expected::Valid,
    );
    add(
        "Deribit future price 60002.5: on 2.5",
        &future,
        price(dec(600_025, 1)),
        Expected::Valid,
    );
    add(
        "Deribit future price 60001: off 2.5",
        &future,
        price(int(60_001)),
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Binance price 60000.15: off 0.1",
        &binance,
        price(dec(6_000_015, 2)),
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Binance size 0.0015 BTC: off 0.001",
        &binance,
        Operation::CheckLot {
            contracts: dec(15, 4),
        },
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Deribit option size 0.15: off 0.1",
        &call,
        Operation::CheckLot {
            contracts: dec(15, 2),
        },
        Expected::Error(ErrorKind::NotOnGrid),
    );
    add(
        "Deribit option size 0.3: on 0.1",
        &call,
        Operation::CheckLot {
            contracts: dec(3, 1),
        },
        Expected::Valid,
    );

    // ---------------------------------------------------- rounding ties
    // (rule, +1/8 rounded, -1/8 rounded, +residual, -residual), in
    // hundredths and as num/den.
    type Tie = (RoundingRule, i64, i64, (i64, i64), (i64, i64));
    let ties: [Tie; 6] = [
        (Floor, 12, -13, (1, 200), (1, 200)),
        (Ceil, 13, -12, (-1, 200), (-1, 200)),
        (TowardZero, 12, -12, (1, 200), (-1, 200)),
        (AwayFromZero, 13, -13, (-1, 200), (1, 200)),
        (HalfEven, 12, -12, (1, 200), (-1, 200)),
        (HalfAwayFromZero, 13, -13, (-1, 200), (1, 200)),
    ];
    for (rule, pos, neg, pos_res, neg_res) in ties {
        add(
            "tie +1/8 at 2 places",
            &perp,
            Operation::Settle {
                value: ratio(1, 8)?,
                scale: 2,
                rule,
            },
            Expected::Settled(settled(2, rule, dec(pos, 2), pos_res)?),
        );
        add(
            "tie -1/8 at 2 places",
            &perp,
            Operation::Settle {
                value: ratio(-1, 8)?,
                scale: 2,
                rule,
            },
            Expected::Settled(settled(2, rule, dec(neg, 2), neg_res)?),
        );
    }
    add(
        "tie 3/8 half even: the odd floor goes up, 0.38",
        &perp,
        Operation::Settle {
            value: ratio(3, 8)?,
            scale: 2,
            rule: HalfEven,
        },
        Expected::Settled(settled(2, HalfEven, dec(38, 2), (-1, 200))?),
    );
    add(
        "2/3 at scale 28",
        &perp,
        Operation::Settle {
            value: ratio(2, 3)?,
            scale: 28,
            rule: HalfEven,
        },
        Expected::Settled(Settled {
            scale: 28,
            rule: HalfEven,
            amount: Decimal::from_i128_with_scale(6_666_666_666_666_666_666_666_666_667, 28),
            residual: Ratio::new(
                int(-1),
                Decimal::from_i128_with_scale(30_000_000_000_000_000_000_000_000_000, 0),
            )?,
        }),
    );
    add(
        "scale 29 is refused",
        &perp,
        Operation::Settle {
            value: ratio(1, 3)?,
            scale: 29,
            rule: HalfEven,
        },
        Expected::Error(ErrorKind::InvalidField),
    );

    // -------------------------------------------------------- boundaries
    add(
        "Decimal::MAX contracts x price 2: overflow",
        &binance,
        Operation::Notional {
            contracts: Decimal::MAX,
            price: int(2),
        },
        Expected::Error(ErrorKind::Overflow),
    );
    add(
        "1.000000000000001 x 1.00000000000001: 31 digits, inexact",
        &binance,
        Operation::Notional {
            contracts: dec(1_000_000_000_000_001, 15),
            price: dec(100_000_000_000_001, 14),
        },
        Expected::Error(ErrorKind::Inexact),
    );
    add(
        "1e-28 x 1e-28: below scale 28, inexact",
        &binance,
        Operation::Notional {
            contracts: dec(1, 28),
            price: dec(1, 28),
        },
        Expected::Error(ErrorKind::Inexact),
    );
    add(
        "negative premium is refused",
        &call,
        Operation::Premium {
            side: Long,
            contracts: int(1),
            premium: dec(-1, 4),
        },
        Expected::Error(ErrorKind::Negative),
    );
    add(
        "zero contracts are refused",
        &perp,
        Operation::Notional {
            contracts: int(0),
            price: int(60_000),
        },
        Expected::Error(ErrorKind::NotPositive),
    );
    add(
        "expiry of a perpetual is unsupported",
        &perp,
        Operation::Expiry {
            side: Long,
            contracts: int(1),
            settlement_price: int(60_000),
        },
        Expected::Error(ErrorKind::Unsupported),
    );
    Ok(cases)
}

fn same(name: &str, what: &str, ok: Result<bool, InstrumentError>) -> Result<(), String> {
    match ok {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!("{name}: {what} differs")),
        Err(e) => Err(format!("{name}: comparing {what}: {e}")),
    }
}

fn check_settled(name: &str, value: &MoneyRatio, expected: &Settled) -> Result<(), String> {
    let rounded = value
        .settle(expected.scale, expected.rule)
        .map_err(|e| format!("{name}: settle: {e}"))?;
    if rounded.amount.amount != expected.amount {
        return Err(format!(
            "{name}: settled {} instead of {}",
            rounded.amount.amount, expected.amount
        ));
    }
    same(
        name,
        "residual",
        rounded.residual.ratio.equals(&expected.residual),
    )
}

/// Runs `case` against this crate's implementation.
///
/// # Errors
///
/// A description of the first difference.
pub fn check(case: &Case) -> Result<(), String> {
    let name = case.name;
    let spec = &case.spec;
    let result: Result<Outcome, InstrumentError> = match &case.operation {
        Operation::Notional { contracts, price } => {
            spec.notional(*contracts, *price).map(Outcome::Amount)
        }
        Operation::RealizedPnl {
            side,
            contracts,
            entry,
            exit,
        } => spec
            .realized_pnl(*side, *contracts, *entry, *exit)
            .map(Outcome::Amount),
        Operation::Premium {
            side,
            contracts,
            premium,
        } => spec
            .option_premium(*side, *contracts, *premium)
            .map(Outcome::Money),
        Operation::Expiry {
            side,
            contracts,
            settlement_price,
        } => spec
            .option_expiry(*side, *contracts, *settlement_price)
            .map(Outcome::Expiry),
        Operation::CheckPrice { price } => spec.check_price(*price).map(|()| Outcome::Valid),
        Operation::CheckLot { contracts } => spec.check_lot(*contracts).map(|()| Outcome::Valid),
        Operation::Settle { value, scale, rule } => {
            MoneyRatio::new(*value, spec.settlement_asset().clone())
                .settle(*scale, *rule)
                .map(|r| Outcome::Rounded(r.amount.amount, r.residual.ratio))
        }
    };
    match (&case.expected, result) {
        (Expected::Error(kind), Err(e)) if ErrorKind::of(&e) == *kind => Ok(()),
        (Expected::Error(kind), other) => Err(format!("{name}: expected {kind:?}, got {other:?}")),
        (_, Err(e)) => Err(format!("{name}: unexpected error {e}")),
        (Expected::Amount { exact, settled }, Ok(Outcome::Amount(value)))
        | (Expected::Amount { exact, settled }, Ok(Outcome::Expiry(ExpiryOutcome::Cash(value)))) => {
            same(name, "exact amount", value.equals(exact))?;
            settled
                .as_ref()
                .map_or(Ok(()), |s| check_settled(name, &value, s))
        }
        (Expected::Money(m), Ok(Outcome::Money(got))) if got == *m => Ok(()),
        (
            Expected::Delivery { underlying, cash },
            Ok(Outcome::Expiry(ExpiryOutcome::Delivery {
                underlying: u,
                cash: c,
            })),
        ) if u == *underlying && c == *cash => Ok(()),
        (Expected::Abandoned, Ok(Outcome::Expiry(ExpiryOutcome::Abandoned)))
        | (Expected::Valid, Ok(Outcome::Valid)) => Ok(()),
        (Expected::Settled(s), Ok(Outcome::Rounded(amount, residual))) => {
            if amount != s.amount {
                return Err(format!("{name}: settled {amount} instead of {}", s.amount));
            }
            same(name, "residual", residual.equals(&s.residual))
        }
        (expected, got) => Err(format!("{name}: expected {expected:?}, got {got:?}")),
    }
}

#[derive(Debug)]
enum Outcome {
    Amount(MoneyRatio),
    Money(Money),
    Expiry(ExpiryOutcome),
    Valid,
    Rounded(Decimal, Ratio),
}
