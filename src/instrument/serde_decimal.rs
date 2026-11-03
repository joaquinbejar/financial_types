//! Exact, string-only serde for `Decimal` fields.
//!
//! `rust_decimal`'s own `Deserialize` accepts JSON numbers and rounds
//! strings with more digits than a `Decimal` holds (a value with 29
//! decimal places becomes 0). Every `Decimal` of this module is therefore
//! serialized as its exact string and deserialized only from a string of
//! the grammar `-?[0-9]+(\.[0-9]+)?` (no exponent, sign `+`, underscore or
//! whitespace) that `Decimal::from_str_exact` parses without rounding.

use alloc::string::{String, ToString};
use core::fmt;

use rust_decimal::Decimal;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};

/// Whether `s` is `-?[0-9]+(\.[0-9]+)?`.
fn grammar(s: &str) -> bool {
    let digits = s.strip_prefix('-').unwrap_or(s);
    let mut parts = digits.splitn(2, '.');
    let integer = parts.next().unwrap_or("");
    let fraction = parts.next();
    let all_digits = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    all_digits(integer) && fraction.is_none_or(all_digits)
}

/// The exact `Decimal` of `s`.
///
/// # Errors
///
/// A message when `s` is outside the grammar or not exactly representable.
pub(crate) fn parse(s: &str) -> Result<Decimal, String> {
    if !grammar(s) {
        return Err(alloc::format!("{s:?} is not a plain decimal string"));
    }
    // Trailing zeros after the point carry no value: drop them, so that a
    // value is refused only for more than 28 significant decimal places.
    let trimmed = match s.split_once('.') {
        Some((integer, fraction)) => {
            let fraction = fraction.trim_end_matches('0');
            if fraction.len() > 28 {
                return Err(alloc::format!("{s:?} has more than 28 decimal places"));
            }
            if fraction.is_empty() {
                String::from(integer)
            } else {
                alloc::format!("{integer}.{fraction}")
            }
        }
        None => String::from(s),
    };
    let value = Decimal::from_str_exact(&trimmed)
        .map_err(|_| alloc::format!("{s:?} is not exactly representable as a Decimal"))?;
    // "-0" is zero.
    Ok(if value.is_zero() {
        Decimal::ZERO
    } else {
        value
    })
}

struct ExactVisitor;

impl Visitor<'_> for ExactVisitor {
    type Value = Decimal;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a decimal string, exactly representable")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Decimal, E> {
        parse(v).map_err(E::custom)
    }
}

/// Serializes `value` as its exact string.
///
/// # Errors
///
/// Those of the serializer.
pub(crate) fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

/// Deserializes an exact `Decimal` from a string.
///
/// # Errors
///
/// A non-string, a string outside the grammar, or one a `Decimal` cannot
/// hold exactly.
pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
    deserializer.deserialize_str(ExactVisitor)
}

/// The same for `Option<Decimal>`, `null` being `None`.
pub(crate) mod option {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Serializes an optional exact decimal.
    ///
    /// # Errors
    ///
    /// Those of the serializer.
    pub(crate) fn serialize<S: Serializer>(
        value: &Option<Decimal>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.map(|v| v.to_string()).serialize(serializer)
    }

    /// Deserializes an optional exact decimal.
    ///
    /// # Errors
    ///
    /// As [`super::deserialize`].
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Decimal>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|s| parse(&s).map_err(de::Error::custom))
            .transpose()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_is_strict_and_exact() {
        assert_eq!(parse("1.5"), Ok(Decimal::new(15, 1)));
        assert_eq!(parse("-0.0001"), Ok(Decimal::new(-1, 4)));
        assert_eq!(parse("100000"), Ok(Decimal::from(100_000)));
        for bad in [
            "", "-", "1.", ".5", "+1", "1e5", "1_000", " 1", "1 ", "0x10", "1.2.3",
        ] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
        // 29 decimal places: rounded to 0 by Decimal::from_str, refused here.
        let error = parse("0.00000000000000000000000000001").unwrap_err();
        assert!(error.contains("more than 28 decimal places"), "{error}");
        // Trailing zeros beyond 28 places carry no value: accepted.
        assert_eq!(parse("1.00000000000000000000000000000"), Ok(Decimal::ONE));
        assert_eq!(parse("2.50"), Ok(Decimal::new(25, 1)));
        // Exactly representable: 28 decimal places, mantissa 10^28 + 1 < 2^96.
        assert_eq!(
            parse("1.0000000000000000000000000001"),
            Ok(Decimal::from_i128_with_scale(
                10_000_000_000_000_000_000_000_000_001,
                28
            ))
        );
        // The non-zero digit is the 28th place; the 29th is a zero: 1e-28.
        assert_eq!(
            parse("0.00000000000000000000000000010"),
            Ok(Decimal::new(1, 28))
        );
        // A non-zero digit at the 29th place: refused.
        assert!(parse("0.000000000000000000000000000010").is_err());
        assert!(parse("1.00000000000000000000000000001").is_err());
        assert!(!parse("-0").unwrap().is_sign_negative());
        assert!(!parse("-0.000").unwrap().is_sign_negative());
        // 29 significant digits.
        assert!(parse("9.0000000000000000000000000001").is_err());
        assert!(parse("79228162514264337593543950336").is_err());
    }

    #[test]
    fn test_a_number_is_refused_with_the_expected_message() {
        #[derive(Debug, serde::Deserialize)]
        struct W(#[serde(with = "super")] Decimal);
        let ok = serde_json::from_str::<W>("\"0.1\"").unwrap();
        assert_eq!(ok.0, Decimal::new(1, 1));
        let error = serde_json::from_str::<W>("0.1").unwrap_err().to_string();
        assert!(
            error.contains("a decimal string, exactly representable"),
            "{error}"
        );
    }
}
