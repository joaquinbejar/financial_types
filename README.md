[![Crates.io](https://img.shields.io/crates/v/financial_types.svg)](https://crates.io/crates/financial_types)
[![docs.rs](https://img.shields.io/docsrs/financial_types)](https://docs.rs/financial_types)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.85%2B-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![Downloads](https://img.shields.io/crates/d/financial_types.svg)](https://crates.io/crates/financial_types)

[![Build](https://github.com/joaquinbejar/financial_types/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/build.yml)
[![Lint](https://github.com/joaquinbejar/financial_types/actions/workflows/lint.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/lint.yml)
[![Format](https://github.com/joaquinbejar/financial_types/actions/workflows/format_check.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/format_check.yml)
[![Audit](https://github.com/joaquinbejar/financial_types/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/audit.yml)
[![Deny](https://github.com/joaquinbejar/financial_types/actions/workflows/deny.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/deny.yml)
[![Semver](https://github.com/joaquinbejar/financial_types/actions/workflows/semver.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/semver.yml)
[![Coverage](https://github.com/joaquinbejar/financial_types/actions/workflows/code_coverage.yml/badge.svg?branch=main)](https://github.com/joaquinbejar/financial_types/actions/workflows/code_coverage.yml)
[![codecov](https://codecov.io/gh/joaquinbejar/financial_types/branch/main/graph/badge.svg)](https://codecov.io/gh/joaquinbejar/financial_types)

[![Stars](https://img.shields.io/github/stars/joaquinbejar/financial_types.svg)](https://github.com/joaquinbejar/financial_types/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/financial_types.svg)](https://github.com/joaquinbejar/financial_types/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/financial_types.svg)](https://github.com/joaquinbejar/financial_types/pulls)


## Financial Types

Core financial type definitions for trading systems in Rust.

### Overview

`financial_types` is a lightweight Rust crate providing fundamental enums for
financial applications. These types are the building blocks used across trading
systems, options pricing libraries, and portfolio management tools.

All enums use `#[repr(u8)]` for compact memory layout (1 byte each) and include
`serde` serialization support out of the box.

### Types

| Type | Variants | Description |
|---|---|---|
| `UnderlyingAssetType` | Crypto, Stock, Forex, Commodity, Bond, Other | Classification of asset classes |
| `Action` | Buy, Sell, Other | Trading actions |
| `Side` | Long, Short | Position directional exposure |
| `OptionStyle` | Call, Put | Option contract style |

### Features

- **Compact**: All enums are `#[repr(u8)]` — 1 byte each
- **Safe**: `#[must_use]` on all pure helper methods
- **Serde**: Full serialization/deserialization support
- **OpenAPI**: Optional `utoipa` support via feature flag
- **Helpers**: `is_*()` and `opposite()` methods on applicable types
- **Parsing**: `FromStr`, `TryFrom<&str>`, `TryFrom<u8>` on every enum
  (case-insensitive string parsing, discriminant-based `u8` conversion)
- **`no_std`**: Compiles without `std`; only `alloc` is required
- **Fuzzing**: Optional `arbitrary` and `proptest` features generate
  random variants for property-based testing

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
financial_types = "0.2"
```

To enable OpenAPI schema support:

```toml
[dependencies]
financial_types = { version = "0.2", features = ["utoipa"] }
```

### Migration: 0.1 → 0.2

`UnderlyingAssetType` and `Action` are now `#[non_exhaustive]`.
Exhaustive `match` expressions on either enum must include a wildcard
arm:

```rust
use financial_types::Action;

let action = Action::Buy;
match action {
    Action::Buy => { /* ... */ }
    Action::Sell => { /* ... */ }
    Action::Other => { /* ... */ }
    _ => { /* future variants */ }
}
```

`Side` and `OptionStyle` remain exhaustive — no migration needed.

### Quick Start

```rust
use financial_types::{Action, Side, OptionStyle, UnderlyingAssetType};

let action = Action::Buy;
let side = Side::Long;
let style = OptionStyle::Call;
let asset = UnderlyingAssetType::Stock;

assert_eq!(format!("{action}"), "Buy");
assert_eq!(format!("{side}"), "Long");
assert_eq!(format!("{style}"), "Call");
assert_eq!(format!("{asset}"), "Stock");

// Helper methods
assert!(style.is_call());
assert!(side.is_long());
assert!(action.is_buy());

// Opposite helpers
assert_eq!(side.opposite(), Side::Short);
assert_eq!(style.opposite(), OptionStyle::Put);
```

### API

#### `UnderlyingAssetType`

```rust
use financial_types::UnderlyingAssetType;

let asset = UnderlyingAssetType::Stock;
assert!(asset.is_stock());
assert!(!asset.is_crypto());
```

Helpers: `is_stock()`, `is_crypto()`, `is_forex()`, `is_commodity()`, `is_bond()`

#### `Action`

```rust
use financial_types::Action;

let action = Action::Buy;
assert!(action.is_buy());
assert!(!action.is_sell());
```

Helpers: `is_buy()`, `is_sell()`

#### `Side`

```rust
use financial_types::Side;

let side = Side::Long;
assert!(side.is_long());
assert_eq!(side.opposite(), Side::Short);
```

Helpers: `is_long()`, `is_short()`, `opposite()`

#### `OptionStyle`

```rust
use financial_types::OptionStyle;

let style = OptionStyle::Call;
assert!(style.is_call());
assert_eq!(style.opposite(), OptionStyle::Put);
assert!(OptionStyle::Call < OptionStyle::Put); // Ord supported
```

Helpers: `is_call()`, `is_put()`, `opposite()`

#### Serialization

```rust
use financial_types::Side;

let side = Side::Long;
let json = serde_json::to_string(&side).unwrap();  // "\"Long\""
let parsed: Side = serde_json::from_str(&json).unwrap();
assert_eq!(side, parsed);
```

#### Parsing

```rust
use financial_types::{Action, Side, UnderlyingAssetType};
use std::str::FromStr;

// FromStr — case-insensitive, trims whitespace
assert_eq!(Side::from_str("Long").unwrap(), Side::Long);
assert_eq!(Side::from_str("  short  ").unwrap(), Side::Short);
assert_eq!("SELL".parse::<Action>().unwrap(), Action::Sell);

// TryFrom<&str>
let asset: UnderlyingAssetType = "Stock".try_into().unwrap();
assert_eq!(asset, UnderlyingAssetType::Stock);

// TryFrom<u8> — uses #[repr(u8)] discriminants
assert_eq!(Side::try_from(0u8).unwrap(), Side::Long);
assert!(Side::try_from(9u8).is_err());
```

### Minimum Supported Rust Version (MSRV)

`financial_types` requires **Rust 1.85** or later (edition 2024).
A dedicated CI job builds and tests on the exact MSRV. Bumping the
MSRV is a **minor** version change — never sneaked into a patch.

### Examples

Runnable examples live in [`examples/`](./examples/):

- [`parse_action`](./examples/parse_action.rs) — parse `Action` from a CLI string.
- [`iterate_assets`](./examples/iterate_assets.rs) — iterate every `UnderlyingAssetType` via `ALL`.
- [`serde_roundtrip`](./examples/serde_roundtrip.rs) — JSON round-trip every variant.
- [`utoipa_schema`](./examples/utoipa_schema.rs) — dump OpenAPI schemas (requires `--features utoipa`).

```bash
cargo run --example parse_action -- buy
cargo run --example iterate_assets
cargo run --example serde_roundtrip
cargo run --example utoipa_schema --features utoipa
```

### Changelog

See [`CHANGELOG.md`](./CHANGELOG.md) for the full history of releases.

### License

This project is licensed under the MIT License.



## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:


### **Contact Information**

- **Author**: Joaquín Béjar García
- **Email**: jb@taunais.com
- **Telegram**: [@joaquin_bejar](https://t.me/joaquin_bejar)
- **Repository**: <https://github.com/joaquinbejar/financial_types>
- **Documentation**: <https://docs.rs/financial_types>

We appreciate your interest and look forward to your contributions!

## ✍️ License

Licensed under **MIT** license
