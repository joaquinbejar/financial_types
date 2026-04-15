# Changelog

All notable changes to `financial_types` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Every PR that touches the public API must add an entry under
> `[Unreleased]`. Group entries by Added / Changed / Deprecated /
> Removed / Fixed / Security / Internal.

## [Unreleased]

### Changed

- `missing_docs` upgraded from `warn` to `deny`. Adds
  `rustdoc::broken_intra_doc_links = "deny"`. Documentation on every
  public item is now a hard CI gate.

### Added

- README badge strip: crates.io, docs.rs, license, downloads, per-workflow
  CI badges (build, lint, format, audit, semver, coverage), and Codecov.
- `examples/` directory with four runnable examples covering parsing,
  iteration via `ALL`, serde round-trips, and (feature-gated)
  `utoipa::ToSchema` dumps. CI builds them under both default and
  `--all-features` configurations.

### Changed

- **BREAKING**: `UnderlyingAssetType` and `Action` are now marked
  `#[non_exhaustive]`. Future variants (new asset classes, new trading
  actions) can be added without a major version bump. Downstream
  exhaustive `match` expressions on these enums must add a wildcard
  arm (`_ =>`). `Side` and `OptionStyle` remain intentionally exhaustive
  (closed two-state concepts).

### Added

- `ParseEnumError` public error type for failed enum parsing.
- `FromStr`, `TryFrom<&str>`, and `TryFrom<u8>` implementations for
  `UnderlyingAssetType`, `Action`, `Side`, and `OptionStyle`.
- String parsing is case-insensitive and trims surrounding whitespace.
- `TryFrom<u8>` uses the documented `#[repr(u8)]` discriminants.
- `const fn as_str()` on every public enum returning a `&'static str`
  matching the `Display` output. Zero-allocation alternative to `format!`.
- `pub const ALL: &'static [Self]` on every public enum, listing every
  variant in `#[repr(u8)]` discriminant order. Useful for iteration,
  validation, and UI generation.
- `Action::opposite()` (`Buy ↔ Sell`; `Other → Other`). Brings `Action`
  to API parity with `Side::opposite` and `OptionStyle::opposite`.

### Changed

- `Display` impls now delegate to `as_str()` via `f.write_str`, removing
  the formatting-layer overhead on the hot path.
- Crate is now `no_std`-compatible. New `std` feature (enabled by
  default) gates the `serde/std` integration and is implied by the
  `utoipa` feature. The `alloc` crate is always required.
- `serde` dependency switched to `default-features = false`,
  `features = ["derive", "alloc"]`.
- Derives `PartialOrd, Ord` on `UnderlyingAssetType`, `Action`, and
  `Side` (previously only `OptionStyle`). Ordering follows the
  documented `#[repr(u8)]` discriminants and is non-breaking.
- Optional `arbitrary` feature deriving `arbitrary::Arbitrary` on every
  public enum.
- Optional `proptest` feature implementing `proptest::arbitrary::Arbitrary`
  on every public enum (uniform `prop_oneof` over variants). Enables
  property-based testing of code that consumes these types.

### Internal

- Moved per-enum unit tests into a dedicated `tests/` integration
  directory. Each test now exercises only the public API surface,
  matching how downstream consumers use the crate.
- Added `tests/serde_snapshots.rs` asserting the exact JSON encoding
  for every variant of every public enum. The wire format is now a
  SemVer-tracked contract documented in the crate-level docs.
- Added `tests/utoipa.rs` exercising every enum's `utoipa::ToSchema`
  derive. CI catches a forgotten `#[cfg_attr(feature = "utoipa", ...)]`
  on a new enum.

## [0.1.0] - 2025-01-01

### Added

- Initial release with `UnderlyingAssetType`, `Action`, `Side`, and
  `OptionStyle` enums.
- `#[repr(u8)]` on every public enum (1-byte layout).
- `serde` `Serialize`/`Deserialize` support on every enum.
- Optional `utoipa` feature for OpenAPI schema generation.
- Classification helpers (`is_stock`, `is_crypto`, `is_long`, `is_call`, ...).
- `opposite()` helpers on `Side` and `OptionStyle`.
- `Display` on every enum; namespaced `Debug` on `Side` and `OptionStyle`.

[Unreleased]: https://github.com/joaquinbejar/financial_types/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/joaquinbejar/financial_types/releases/tag/v0.1.0
