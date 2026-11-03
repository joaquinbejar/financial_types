//! Amounts of an asset: exact decimals, exact rationals, and their explicit
//! rounding.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::asset::AssetCode;
use super::error::InstrumentError;
use super::exact::{fit, pow10};

/// An exact amount of one asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// The amount; serialized as a string, exactly.
    pub amount: Decimal,
    /// The asset.
    pub asset: AssetCode,
}

impl Money {
    /// An amount of `asset`.
    #[must_use]
    #[inline]
    pub fn new(amount: Decimal, asset: AssetCode) -> Self {
        Self { amount, asset }
    }
}

/// An exact rational `num / den`, for amounts that need not terminate in
/// decimal (a contract value divided by a price, for instance).
///
/// The denominator is positive, the sign is carried by the numerator, and
/// zero is `0 / 1`. The fraction is not reduced: `1/2` and `2/4` are
/// different representations of the same value, so this type has no
/// structural equality. Compare values with [`Ratio::equals`], which is exact
/// and fallible.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "RatioRepr", into = "RatioRepr")]
pub struct Ratio {
    num: Decimal,
    den: Decimal,
}

/// The serialized form of a [`Ratio`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct RatioRepr {
    num: Decimal,
    den: Decimal,
}

impl TryFrom<RatioRepr> for Ratio {
    type Error = InstrumentError;

    fn try_from(repr: RatioRepr) -> Result<Self, Self::Error> {
        Self::new(repr.num, repr.den)
    }
}

impl From<Ratio> for RatioRepr {
    #[inline]
    fn from(ratio: Ratio) -> Self {
        Self {
            num: ratio.num,
            den: ratio.den,
        }
    }
}

impl Ratio {
    /// `num / den`, with the sign moved to the numerator and zero as `0/1`.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidField`] if `den` is zero.
    pub fn new(num: Decimal, den: Decimal) -> Result<Self, InstrumentError> {
        if den.is_zero() {
            return Err(InstrumentError::invalid_field(
                "denominator",
                "a ratio's denominator must not be zero",
            ));
        }
        if num.is_zero() {
            return Ok(Self::zero());
        }
        Ok(if den.is_sign_negative() {
            Self {
                num: -num,
                den: -den,
            }
        } else {
            Self { num, den }
        })
    }

    /// A whole decimal value, `value / 1`.
    #[must_use]
    #[inline]
    pub fn from_decimal(value: Decimal) -> Self {
        if value.is_zero() {
            Self::zero()
        } else {
            Self {
                num: value,
                den: Decimal::ONE,
            }
        }
    }

    /// Zero, `0 / 1`.
    #[must_use]
    #[inline]
    pub fn zero() -> Self {
        Self {
            num: Decimal::ZERO,
            den: Decimal::ONE,
        }
    }

    /// The numerator, carrying the sign.
    #[must_use]
    #[inline]
    pub fn num(&self) -> Decimal {
        self.num
    }

    /// The denominator, positive.
    #[must_use]
    #[inline]
    pub fn den(&self) -> Decimal {
        self.den
    }

    /// Whether the value is zero.
    #[must_use]
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }

    /// Whether `self` and `other` are the same value, by exact
    /// cross-multiplication on integers.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::Overflow`] if an integer intermediate leaves
    /// `i128`; an overflow is never reported as `false`.
    pub fn equals(&self, other: &Self) -> Result<bool, InstrumentError> {
        const OP: &str = "ratio comparison";
        let cross = |a: Decimal, b: Decimal| -> Result<(i128, u32), InstrumentError> {
            let (a, b) = (a.normalize(), b.normalize());
            let m = a
                .mantissa()
                .checked_mul(b.mantissa())
                .ok_or_else(|| InstrumentError::overflow(OP))?;
            Ok((m, a.scale() + b.scale()))
        };
        let (left, ls) = cross(self.num, other.den)?;
        let (right, rs) = cross(other.num, self.den)?;
        let scale = ls.max(rs);
        let up = |m: i128, s: u32| {
            pow10(scale - s)
                .and_then(|p| m.checked_mul(p))
                .ok_or_else(|| InstrumentError::overflow(OP))
        };
        Ok(up(left, ls)? == up(right, rs)?)
    }

    /// Rounds to `scale` decimal places with `rule`, exactly.
    ///
    /// The quotient and remainder are computed on integers: with the value
    /// `P / Q` scaled by `10^scale`, `q = floor(P / Q)` and `0 <= r < Q`.
    /// Ties are decided by comparing `r` with `Q - r`, so nothing overflows
    /// on `2r`. The residual is `self - rounded`, exactly.
    ///
    /// # Errors
    ///
    /// [`InstrumentError::InvalidField`] for a scale above 28, and
    /// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`] when an
    /// integer intermediate, the rounded amount or the residual is out of
    /// range.
    pub fn round(
        &self,
        scale: u32,
        rule: RoundingRule,
    ) -> Result<(Decimal, Ratio), InstrumentError> {
        const OP: &str = "rounding";
        if scale > 28 {
            return Err(InstrumentError::invalid_field(
                "scale",
                alloc::format!("{scale} is above 28"),
            ));
        }
        let (num, den) = (self.num.normalize(), self.den.normalize());
        // value * 10^scale = num.m * 10^(den.s + scale) / (den.m * 10^num.s)
        let p = pow10(den.scale() + scale)
            .and_then(|f| num.mantissa().checked_mul(f))
            .ok_or_else(|| InstrumentError::overflow(OP))?;
        let q_den = pow10(num.scale())
            .and_then(|f| den.mantissa().checked_mul(f))
            .ok_or_else(|| InstrumentError::overflow(OP))?;
        if q_den <= 0 {
            return Err(InstrumentError::invalid_field(
                "denominator",
                "a ratio's denominator must be positive",
            ));
        }
        let floor = p.div_euclid(q_den);
        let r = p.rem_euclid(q_den);
        let up = || {
            floor
                .checked_add(1)
                .ok_or_else(|| InstrumentError::overflow(OP))
        };
        let negative = p < 0;
        let q = match rule {
            RoundingRule::Floor => floor,
            RoundingRule::Ceil if r > 0 => up()?,
            RoundingRule::Ceil => floor,
            RoundingRule::TowardZero if negative && r > 0 => up()?,
            RoundingRule::TowardZero => floor,
            RoundingRule::AwayFromZero if !negative && r > 0 => up()?,
            RoundingRule::AwayFromZero => floor,
            RoundingRule::HalfEven | RoundingRule::HalfAwayFromZero if r == 0 => floor,
            RoundingRule::HalfEven | RoundingRule::HalfAwayFromZero => {
                let rest = q_den - r;
                match r.cmp(&rest) {
                    core::cmp::Ordering::Less => floor,
                    core::cmp::Ordering::Greater => up()?,
                    core::cmp::Ordering::Equal => {
                        let to_ceil = match rule {
                            RoundingRule::HalfEven => floor.rem_euclid(2) != 0,
                            // Away from zero: up for a positive value, the
                            // floor (more negative) for a negative one.
                            _ => !negative,
                        };
                        if to_ceil { up()? } else { floor }
                    }
                }
            }
        };
        let amount = fit(OP, q, scale)?;
        let remainder = q
            .checked_mul(q_den)
            .and_then(|qq| p.checked_sub(qq))
            .ok_or_else(|| InstrumentError::overflow(OP))?;
        // residual = remainder / (q_den * 10^scale)
        let residual = Ratio::new(fit(OP, remainder, scale)?, fit(OP, q_den, 0)?)?;
        Ok((amount, residual))
    }
}

/// An exact rational amount of one asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyRatio {
    /// The amount.
    pub ratio: Ratio,
    /// The asset.
    pub asset: AssetCode,
}

impl MoneyRatio {
    /// An amount of `asset`.
    #[must_use]
    #[inline]
    pub fn new(ratio: Ratio, asset: AssetCode) -> Self {
        Self { ratio, asset }
    }

    /// Whether `self` and `other` are the same amount of the same asset.
    ///
    /// # Errors
    ///
    /// As [`Ratio::equals`].
    pub fn equals(&self, other: &Self) -> Result<bool, InstrumentError> {
        Ok(self.asset == other.asset && self.ratio.equals(&other.ratio)?)
    }

    /// The amount settled at `scale` decimal places with `rule`, and what
    /// rounding left out.
    ///
    /// # Errors
    ///
    /// As [`Ratio::round`].
    pub fn settle(&self, scale: u32, rule: RoundingRule) -> Result<Rounded, InstrumentError> {
        let (amount, residual) = self.ratio.round(scale, rule)?;
        Ok(Rounded {
            amount: Money::new(amount, self.asset.clone()),
            residual: MoneyRatio::new(residual, self.asset.clone()),
        })
    }
}

/// A settled amount and the exact residual `original - amount`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rounded {
    /// The amount at the settlement scale.
    pub amount: Money,
    /// What rounding left out, exactly.
    pub residual: MoneyRatio,
}

/// How a value is rounded to a scale.
///
/// # Stability
///
/// This enum is `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[repr(u8)]
#[non_exhaustive]
pub enum RoundingRule {
    /// Toward negative infinity.
    Floor = 0,
    /// Toward positive infinity.
    Ceil = 1,
    /// Toward zero (truncation).
    TowardZero = 2,
    /// Away from zero.
    AwayFromZero = 3,
    /// To the nearest; a tie goes to the even neighbour.
    HalfEven = 4,
    /// To the nearest; a tie goes away from zero.
    HalfAwayFromZero = 5,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn r(num: &str, den: &str) -> Ratio {
        Ratio::new(d(num), d(den)).unwrap()
    }

    #[test]
    fn test_ratio_canonical_sign_and_zero() {
        let x = r("1", "-2");
        assert_eq!((x.num(), x.den()), (d("-1"), d("2")));
        let z = r("0", "-7");
        assert_eq!((z.num(), z.den()), (d("0"), d("1")));
        assert!(Ratio::new(d("1"), d("0")).is_err());
    }

    #[test]
    fn test_ratio_equality_is_by_value() {
        assert!(r("1", "2").equals(&r("2", "4")).unwrap());
        assert!(r("0.5", "1").equals(&r("1", "2")).unwrap());
        assert!(!r("1", "3").equals(&r("33", "100")).unwrap());
        // A cross product beyond i128 is an error, never false.
        let big = Ratio::new(Decimal::MAX, Decimal::MAX).unwrap();
        let other = Ratio::new(Decimal::MAX, Decimal::MAX - Decimal::ONE).unwrap();
        assert!(big.equals(&other).is_err());
    }

    #[test]
    fn test_round_ties_for_every_rule() {
        use RoundingRule::*;
        // +0.125 and -0.125 at 2 places: a tie.
        let cases = [
            (Floor, "0.12", "-0.13"),
            (Ceil, "0.13", "-0.12"),
            (TowardZero, "0.12", "-0.12"),
            (AwayFromZero, "0.13", "-0.13"),
            (HalfEven, "0.12", "-0.12"),
            (HalfAwayFromZero, "0.13", "-0.13"),
        ];
        for (rule, pos, neg) in cases {
            let (p, pr) = r("1", "8").round(2, rule).unwrap();
            let (n, nr) = r("-1", "8").round(2, rule).unwrap();
            assert_eq!((p, n), (d(pos), d(neg)), "{rule:?}");
            // residual = original - rounded, exactly.
            let back = |amount: Decimal, res: Ratio, original: Ratio| {
                let total = Ratio::new(amount * res.den() + res.num(), res.den()).unwrap();
                assert!(total.equals(&original).unwrap(), "{rule:?}");
            };
            back(p, pr, r("1", "8"));
            back(n, nr, r("-1", "8"));
        }
        // A large negative tie: -123456789012345678901233.5 at scale 0.
        let big_tie = Ratio::new(
            Decimal::from_i128_with_scale(-246_913_578_024_691_357_802_467, 0),
            d("2"),
        )
        .unwrap();
        let floor = Decimal::from_i128_with_scale(-123_456_789_012_345_678_901_234, 0);
        let ceil = Decimal::from_i128_with_scale(-123_456_789_012_345_678_901_233, 0);
        assert_eq!(big_tie.round(0, HalfEven).unwrap().0, floor);
        assert_eq!(big_tie.round(0, HalfAwayFromZero).unwrap().0, floor);
        assert_eq!(big_tie.round(0, TowardZero).unwrap().0, ceil);
        // Scaling beyond i128 is a typed overflow.
        let max = Ratio::new(Decimal::MAX, d("3")).unwrap();
        assert!(matches!(
            max.round(28, HalfEven),
            Err(InstrumentError::Overflow { .. })
        ));
        // 0.375: a tie with an odd floor goes up under half even.
        assert_eq!(r("3", "8").round(2, HalfEven).unwrap().0, d("0.38"));
    }

    #[test]
    fn test_round_non_terminating_and_bounds() {
        let (a, res) = r("1", "3").round(8, RoundingRule::HalfEven).unwrap();
        assert_eq!(a, d("0.33333333"));
        assert!(res.equals(&r("1", "300000000")).unwrap());
        assert!(r("1", "3").round(29, RoundingRule::HalfEven).is_err());
        // Scale 28 works.
        let (a, _) = r("2", "3").round(28, RoundingRule::HalfEven).unwrap();
        assert_eq!(a, d("0.6666666666666666666666666667"));
    }
}
