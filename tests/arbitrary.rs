//! Integration tests for the `arbitrary` module.

#![cfg(feature = "arbitrary")]
#![allow(clippy::unwrap_used)]

use arbitrary::{Arbitrary, Unstructured};
use financial_types::{Action, Side};

#[test]
fn test_arbitrary_side() {
    let raw = [0u8, 1, 2, 3, 4, 5, 6, 7];
    let mut u = Unstructured::new(&raw);
    let _ = Side::arbitrary(&mut u).unwrap();
}

#[test]
fn test_arbitrary_action() {
    let raw = [0u8, 1, 2, 3, 4];
    let mut u = Unstructured::new(&raw);
    let _ = Action::arbitrary(&mut u).unwrap();
}
