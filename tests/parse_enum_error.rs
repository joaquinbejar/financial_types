//! Integration tests for the `parse_enum_error` module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use financial_types::ParseEnumError;

#[test]
fn test_display() {
    let err = ParseEnumError::new("Side", "sideways");
    assert_eq!(format!("{err}"), "invalid Side value: \"sideways\"");
}

#[test]
fn test_error_trait() {
    let err = ParseEnumError::new("Action", "hold");
    let _: &dyn std::error::Error = &err;
}
