//! The error of every fallible instrument operation.

use alloc::string::String;

use rust_decimal::Decimal;

use super::asset::AssetCode;

/// Why an instrument operation failed.
///
/// Every variant is a refusal, never an approximation: no operation of this
/// module returns a rounded, saturated or clamped value in place of an
/// error. The variants are stable across minor releases.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InstrumentError {
    /// A field is outside its domain.
    #[error("invalid {field}: {reason}")]
    InvalidField {
        /// The field.
        field: &'static str,
        /// Why.
        reason: String,
    },
    /// The fields are valid one by one but not together, such as a
    /// perpetual with an expiry or option terms on a future.
    #[error("invalid combination: {reason}")]
    InvalidCombination {
        /// Why.
        reason: String,
    },
    /// An asset differs from the one the contract requires.
    #[error("currency mismatch for {field}: expected {expected}, got {got}")]
    CurrencyMismatch {
        /// The field.
        field: &'static str,
        /// The asset required.
        expected: AssetCode,
        /// The asset given.
        got: AssetCode,
    },
    /// A value that must be strictly positive is not.
    #[error("{field} must be positive, got {value}")]
    NotPositive {
        /// The field.
        field: &'static str,
        /// The value given.
        value: Decimal,
    },
    /// A value that must not be negative is.
    #[error("{field} must not be negative, got {value}")]
    Negative {
        /// The field.
        field: &'static str,
        /// The value given.
        value: Decimal,
    },
    /// A value is not a whole multiple of its grid step.
    #[error("{field} {value} is not a multiple of {step}")]
    NotOnGrid {
        /// The field.
        field: &'static str,
        /// The value given.
        value: Decimal,
        /// The step it must be a multiple of.
        step: Decimal,
    },
    /// The exact result needs more significant digits than a `Decimal`
    /// holds (a mantissa of 96 bits at scale 28 at most).
    #[error("{op}: the exact result needs more precision than a Decimal holds")]
    Inexact {
        /// The operation.
        op: &'static str,
    },
    /// The result, or an integer intermediate (`i128`), is out of range.
    /// This is reported even when a mathematically equivalent reordering of
    /// the computation might fit: the arithmetic is exact within these
    /// bounds, not of arbitrary precision.
    #[error("{op}: out of the representable range")]
    Overflow {
        /// The operation.
        op: &'static str,
    },
    /// The operation is not defined for this contract, such as physical
    /// delivery of an inverse option or American exercise.
    #[error("unsupported: {what}")]
    Unsupported {
        /// What.
        what: &'static str,
    },
}

impl InstrumentError {
    #[cold]
    #[inline(never)]
    pub(crate) fn invalid_field(field: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidField {
            field,
            reason: reason.into(),
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn invalid_combination(reason: impl Into<String>) -> Self {
        Self::InvalidCombination {
            reason: reason.into(),
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn currency_mismatch(
        field: &'static str,
        expected: &AssetCode,
        got: &AssetCode,
    ) -> Self {
        Self::CurrencyMismatch {
            field,
            expected: expected.clone(),
            got: got.clone(),
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn not_positive(field: &'static str, value: Decimal) -> Self {
        Self::NotPositive { field, value }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn negative(field: &'static str, value: Decimal) -> Self {
        Self::Negative { field, value }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn not_on_grid(field: &'static str, value: Decimal, step: Decimal) -> Self {
        Self::NotOnGrid { field, value, step }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn inexact(op: &'static str) -> Self {
        Self::Inexact { op }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn overflow(op: &'static str) -> Self {
        Self::Overflow { op }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn unsupported(what: &'static str) -> Self {
        Self::Unsupported { what }
    }
}
