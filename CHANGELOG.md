# Changelog

All notable changes to `financial_types` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
