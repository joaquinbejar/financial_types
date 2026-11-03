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
    Decimal::from_str_exact(s)
        .map_err(|_| alloc::format!("{s:?} is not exactly representable as a Decimal"))
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
        assert!(parse("0.00000000000000000000000000001").is_err());
        // 29 significant digits.
        assert!(parse("9.0000000000000000000000000001").is_err());
        assert!(parse("79228162514264337593543950336").is_err());
    }
}
