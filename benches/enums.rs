//! Criterion benchmarks for the hot-path operations on every public
//! enum. Tracks `as_str`, `Display`, `FromStr`, `TryFrom<u8>`, and
//! serde JSON round-trip. Run with `cargo bench`.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
use std::hint::black_box;
use std::str::FromStr;

fn bench_as_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("as_str");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            for v in UnderlyingAssetType::ALL {
                black_box(v.as_str());
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            for v in Action::ALL {
                black_box(v.as_str());
            }
        });
    });
    group.bench_function("Side", |b| {
        b.iter(|| {
            for v in Side::ALL {
                black_box(v.as_str());
            }
        });
    });
    group.bench_function("OptionStyle", |b| {
        b.iter(|| {
            for v in OptionStyle::ALL {
                black_box(v.as_str());
            }
        });
    });
    group.finish();
}

fn bench_display(c: &mut Criterion) {
    use std::fmt::Write as _;
    let mut group = c.benchmark_group("display");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            let mut buf = String::with_capacity(16);
            for v in UnderlyingAssetType::ALL {
                buf.clear();
                let _ = write!(buf, "{v}");
                black_box(&buf);
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            let mut buf = String::with_capacity(8);
            for v in Action::ALL {
                buf.clear();
                let _ = write!(buf, "{v}");
                black_box(&buf);
            }
        });
    });
    group.finish();
}

fn bench_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_str");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            for input in ["Crypto", "stock", "FOREX", "  Commodity  ", "Bond", "Other"] {
                black_box(UnderlyingAssetType::from_str(black_box(input)).ok());
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            for input in ["buy", "Sell", "OTHER"] {
                black_box(Action::from_str(black_box(input)).ok());
            }
        });
    });
    group.bench_function("Side", |b| {
        b.iter(|| {
            for input in ["Long", "short"] {
                black_box(Side::from_str(black_box(input)).ok());
            }
        });
    });
    group.bench_function("OptionStyle", |b| {
        b.iter(|| {
            for input in ["Call", "put"] {
                black_box(OptionStyle::from_str(black_box(input)).ok());
            }
        });
    });
    group.finish();
}

fn bench_try_from_u8(c: &mut Criterion) {
    let mut group = c.benchmark_group("try_from_u8");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            for v in 0u8..6 {
                black_box(UnderlyingAssetType::try_from(black_box(v)).ok());
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            for v in 0u8..3 {
                black_box(Action::try_from(black_box(v)).ok());
            }
        });
    });
    group.bench_function("Side", |b| {
        b.iter(|| {
            for v in 0u8..2 {
                black_box(Side::try_from(black_box(v)).ok());
            }
        });
    });
    group.bench_function("OptionStyle", |b| {
        b.iter(|| {
            for v in 0u8..2 {
                black_box(OptionStyle::try_from(black_box(v)).ok());
            }
        });
    });
    group.finish();
}

fn bench_serde_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_roundtrip");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            for v in UnderlyingAssetType::ALL {
                let s = serde_json::to_string(v).ok();
                if let Some(s) = s {
                    let back: Result<UnderlyingAssetType, _> = serde_json::from_str(&s);
                    black_box(back.ok());
                }
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            for v in Action::ALL {
                let s = serde_json::to_string(v).ok();
                if let Some(s) = s {
                    let back: Result<Action, _> = serde_json::from_str(&s);
                    black_box(back.ok());
                }
            }
        });
    });
    group.finish();
}

fn bench_is_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_helpers");
    group.bench_function("UnderlyingAssetType", |b| {
        b.iter(|| {
            for v in UnderlyingAssetType::ALL {
                black_box(v.is_stock());
                black_box(v.is_crypto());
                black_box(v.is_forex());
                black_box(v.is_commodity());
                black_box(v.is_bond());
            }
        });
    });
    group.bench_function("Action", |b| {
        b.iter(|| {
            for v in Action::ALL {
                black_box(v.is_buy());
                black_box(v.is_sell());
                black_box(v.opposite());
            }
        });
    });
    group.bench_function("Side", |b| {
        b.iter(|| {
            for v in Side::ALL {
                black_box(v.is_long());
                black_box(v.is_short());
                black_box(v.opposite());
            }
        });
    });
    group.bench_function("OptionStyle", |b| {
        b.iter(|| {
            for v in OptionStyle::ALL {
                black_box(v.is_call());
                black_box(v.is_put());
                black_box(v.opposite());
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_as_str,
    bench_display,
    bench_from_str,
    bench_try_from_u8,
    bench_serde_roundtrip,
    bench_is_helpers,
);
criterion_main!(benches);
