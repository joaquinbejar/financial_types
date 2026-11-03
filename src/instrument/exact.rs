//! Exact decimal arithmetic on integer mantissas.
//!
//! A `Decimal` is an integer mantissa `m` of at most 96 bits and a scale `s`
//! between 0 and 28, worth `m / 10^s`. Checked `Decimal` arithmetic is not
//! exact: `checked_mul` and `checked_add` round, and return `Some`, when the
//! true result needs more digits than a `Decimal` holds. These helpers work
//! on the mantissas as `i128` integers instead, and build the result only
//! when it is representable exactly:
//!
//! - trailing zeros are removed (an exact division by 10) while the scale is
//!   above 28 or the mantissa above 96 bits;
//! - a result whose integer part reaches `2^96` is
//!   [`InstrumentError::Overflow`], any other one that still does not fit is
//!   [`InstrumentError::Inexact`];
//! - an `i128` intermediate that overflows is [`InstrumentError::Overflow`].

use rust_decimal::Decimal;

use super::error::InstrumentError;

/// The largest `Decimal` scale.
const MAX_SCALE: u32 = 28;

/// The exclusive bound of a `Decimal` mantissa's magnitude, `2^96`.
const MANTISSA_BOUND: u128 = 1 << 96;

/// `10^n`, or `None` beyond `i128`.
#[inline]
pub(crate) fn pow10(n: u32) -> Option<i128> {
    10_i128.checked_pow(n)
}

/// The exact `Decimal` worth `mantissa / 10^scale`.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] when its integer part reaches `2^96`, and
/// [`InstrumentError::Inexact`] when it needs more digits than a `Decimal`
/// holds.
pub(crate) fn fit(
    op: &'static str,
    mantissa: i128,
    scale: u32,
) -> Result<Decimal, InstrumentError> {
    let (mut m, mut s) = (mantissa, scale);
    loop {
        if s <= MAX_SCALE && m.unsigned_abs() < MANTISSA_BOUND {
            return Decimal::try_from_i128_with_scale(m, s)
                .map_err(|_| InstrumentError::overflow(op));
        }
        if s > 0 && m % 10 == 0 {
            m /= 10;
            s -= 1;
            continue;
        }
        let integer_part_too_large = match pow10(s) {
            Some(p) => (m / p).unsigned_abs() >= MANTISSA_BOUND,
            // 10^s beyond i128 exceeds |m|: the integer part is 0.
            None => false,
        };
        return Err(if integer_part_too_large {
            InstrumentError::overflow(op)
        } else {
            InstrumentError::inexact(op)
        });
    }
}

/// Both values as integers at a common scale: `(a·10^s, b·10^s, s)`.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] if a scaled mantissa leaves `i128`.
pub(crate) fn align(
    op: &'static str,
    a: Decimal,
    b: Decimal,
) -> Result<(i128, i128, u32), InstrumentError> {
    let (a, b) = (a.normalize(), b.normalize());
    let scale = a.scale().max(b.scale());
    let up = |d: Decimal| {
        pow10(scale - d.scale())
            .and_then(|p| d.mantissa().checked_mul(p))
            .ok_or_else(|| InstrumentError::overflow(op))
    };
    Ok((up(a)?, up(b)?, scale))
}

/// `a · b`, exactly.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`], as
/// [`fit`].
pub(crate) fn exact_mul(
    op: &'static str,
    a: Decimal,
    b: Decimal,
) -> Result<Decimal, InstrumentError> {
    let (a, b) = (a.normalize(), b.normalize());
    let mantissa = a
        .mantissa()
        .checked_mul(b.mantissa())
        .ok_or_else(|| InstrumentError::overflow(op))?;
    fit(op, mantissa, a.scale() + b.scale())
}

/// `a + b`, exactly.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`].
#[cfg(test)]
pub(crate) fn exact_add(
    op: &'static str,
    a: Decimal,
    b: Decimal,
) -> Result<Decimal, InstrumentError> {
    let (x, y, scale) = align(op, a, b)?;
    let sum = x
        .checked_add(y)
        .ok_or_else(|| InstrumentError::overflow(op))?;
    fit(op, sum, scale)
}

/// `a - b`, exactly.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] or [`InstrumentError::Inexact`].
pub(crate) fn exact_sub(
    op: &'static str,
    a: Decimal,
    b: Decimal,
) -> Result<Decimal, InstrumentError> {
    let (x, y, scale) = align(op, a, b)?;
    let difference = x
        .checked_sub(y)
        .ok_or_else(|| InstrumentError::overflow(op))?;
    fit(op, difference, scale)
}

/// Whether `value` is a whole multiple of `step` (`step > 0`), exactly.
///
/// # Errors
///
/// [`InstrumentError::Overflow`] if the aligned integers leave `i128`.
pub(crate) fn is_multiple(
    op: &'static str,
    value: Decimal,
    step: Decimal,
) -> Result<bool, InstrumentError> {
    let (v, t, _) = align(op, value, step)?;
    if t <= 0 {
        return Err(InstrumentError::not_positive("step", step));
    }
    Ok(v.rem_euclid(t) == 0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_exact_mul_refuses_what_checked_mul_rounds() {
        // 29 significant digits: checked_mul returns a rounded Some.
        let a = d("1.000000000000001");
        let b = d("1.00000000000001");
        let rounded = a.checked_mul(b).unwrap();
        assert_ne!(rounded.to_string(), "1.000000000000011000000000000010");
        assert_eq!(exact_mul("t", a, b), Err(InstrumentError::inexact("t")));
        // Representable products are exact.
        assert_eq!(exact_mul("t", d("1.5"), d("2.5")), Ok(d("3.75")));
        assert_eq!(exact_mul("t", d("-0.001"), d("1000")), Ok(d("-1")));
    }

    #[test]
    fn test_exact_add_refuses_what_checked_add_rounds() {
        let big = d("99999999999999999999.99999999");
        let small = d("0.000000001");
        assert_eq!(big.checked_add(small), Some(big));
        assert_eq!(
            exact_add("t", big, small),
            Err(InstrumentError::inexact("t"))
        );
        assert_eq!(
            exact_sub("t", big, small),
            Err(InstrumentError::inexact("t"))
        );
        assert_eq!(exact_add("t", d("0.1"), d("0.2")), Ok(d("0.3")));
    }

    #[test]
    fn test_overflow_versus_inexact() {
        assert_eq!(
            exact_mul("t", Decimal::MAX, d("2")),
            Err(InstrumentError::overflow("t"))
        );
        assert_eq!(
            exact_add("t", Decimal::MAX, d("1")),
            Err(InstrumentError::overflow("t"))
        );
        // 1e-56 is not representable but not large: inexact.
        let tiny = d("0.0000000000000000000000000001");
        assert_eq!(
            exact_mul("t", tiny, tiny),
            Err(InstrumentError::inexact("t"))
        );
        // Trailing zeros removed before refusing: 1e14 x 1e-14 is 1.
        assert_eq!(
            exact_mul("t", d("100000000000000"), d("0.00000000000001")),
            Ok(d("1"))
        );
    }

    #[test]
    fn test_fit_strips_trailing_zeros_before_refusing() {
        // 0.2 x 5e-28: mantissa 10 at scale 29, stripped to 1e-28.
        assert_eq!(
            exact_mul("t", d("0.2"), d("0.0000000000000000000000000005")),
            Ok(d("0.0000000000000000000000000001"))
        );
        // 0.25 x 4e-27 = 1e-27: two trailing zeros at scale 29.
        assert_eq!(
            exact_mul("t", d("0.25"), d("0.000000000000000000000000004")),
            Ok(d("0.000000000000000000000000001"))
        );
        // Mantissa exactly 2^96 at scale 1: not representable, and its
        // integer part is below 2^96, so inexact rather than overflow.
        assert_eq!(
            fit("t", 1_i128 << 96, 1),
            Err(InstrumentError::inexact("t"))
        );
        assert_eq!(
            fit("t", 1_i128 << 96, 0),
            Err(InstrumentError::overflow("t"))
        );
        assert_eq!(fit("t", (1_i128 << 96) - 1, 0), Ok(Decimal::MAX));
        assert_eq!(fit("t", 1_000, 30), Ok(d("0.000000000000000000000000001")));
    }

    #[test]
    fn test_is_multiple_is_exact() {
        assert_eq!(is_multiple("t", d("60000.1"), d("0.1")), Ok(true));
        assert_eq!(is_multiple("t", d("60000.15"), d("0.1")), Ok(false));
        assert_eq!(is_multiple("t", d("-0.0055"), d("0.0005")), Ok(true));
        assert!(is_multiple("t", d("1"), d("0")).is_err());
    }
}
