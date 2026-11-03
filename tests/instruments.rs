//! Instrument specifications: the shared fixtures, validation, serde and
//! direction conversions.

#![cfg(feature = "instruments")]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use financial_types::instrument::{
    AssetCode, ContractKind, ContractSize, ExerciseStyle, ExpiryOutcome, InstrumentDraft,
    InstrumentError, InstrumentId, InstrumentSpec, Margining, MoneyRatio, OptionTerms, Ratio,
    RoundingRule, SettlementType, SizeKind, TickBand, TickSchedule, reference,
};
use financial_types::{Action, OptionStyle, Side};
use rust_decimal::Decimal;

fn a(code: &str) -> AssetCode {
    AssetCode::new(code).unwrap()
}

fn d(mantissa: i64, scale: u32) -> Decimal {
    Decimal::new(mantissa, scale)
}

#[cfg(feature = "fixtures")]
#[test]
fn test_every_shared_fixture_passes() {
    use financial_types::instrument::fixtures::{cases, check};
    let cases = cases().unwrap();
    assert!(cases.len() >= 50, "{} cases", cases.len());
    let failures: Vec<String> = cases.iter().filter_map(|c| check(c).err()).collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

fn linear_perpetual_draft() -> InstrumentDraft {
    InstrumentDraft {
        id: InstrumentId::new("BTCUSDT").unwrap(),
        kind: ContractKind::Perpetual,
        margining: Margining::Linear,
        underlying: a("BTC"),
        price_asset: a("USDT"),
        settlement_asset: a("USDT"),
        size: ContractSize::base(Decimal::ONE, a("BTC")).unwrap(),
        price_ticks: TickSchedule::fixed(d(1, 1)).unwrap(),
        lot: d(1, 3),
        expiry_ms: None,
        option: None,
    }
}

fn option_terms(premium: &str) -> OptionTerms {
    OptionTerms::new(
        Decimal::from(100_000),
        OptionStyle::Call,
        ExerciseStyle::European,
        SettlementType::Cash,
        a(premium),
    )
    .unwrap()
}

#[test]
fn test_inadmissible_combinations_are_typed_errors() {
    let ok = linear_perpetual_draft();
    assert!(InstrumentSpec::new(ok.clone()).is_ok());
    type Case = (&'static str, InstrumentDraft, fn(&InstrumentError) -> bool);
    let cases: Vec<Case> = vec![
        (
            "perpetual with expiry",
            InstrumentDraft {
                expiry_ms: Some(1),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "future without expiry",
            InstrumentDraft {
                kind: ContractKind::Future,
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "option terms on a perpetual",
            InstrumentDraft {
                option: Some(option_terms("USDT")),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "linear settled in USD instead of USDT",
            InstrumentDraft {
                settlement_asset: a("USD"),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::CurrencyMismatch { .. }),
        ),
        (
            "linear sized in value",
            InstrumentDraft {
                size: ContractSize::quote_value(Decimal::TEN, a("USDT")).unwrap(),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "inverse sized in the underlying",
            InstrumentDraft {
                margining: Margining::Inverse,
                settlement_asset: a("BTC"),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "inverse spot",
            InstrumentDraft {
                kind: ContractKind::Spot,
                margining: Margining::Inverse,
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "spot of two units",
            InstrumentDraft {
                kind: ContractKind::Spot,
                size: ContractSize::base(Decimal::TWO, a("BTC")).unwrap(),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "same underlying and price asset",
            InstrumentDraft {
                price_asset: a("BTC"),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "zero lot",
            InstrumentDraft {
                lot: Decimal::ZERO,
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::NotPositive { .. }),
        ),
        (
            "option without terms",
            InstrumentDraft {
                kind: ContractKind::Option,
                expiry_ms: Some(1),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::InvalidCombination { .. }),
        ),
        (
            "linear option with the premium in BTC",
            InstrumentDraft {
                kind: ContractKind::Option,
                expiry_ms: Some(1),
                option: Some(option_terms("BTC")),
                ..ok.clone()
            },
            |e| matches!(e, InstrumentError::CurrencyMismatch { .. }),
        ),
    ];
    for (name, draft, expected) in cases {
        let error = InstrumentSpec::new(draft).unwrap_err();
        assert!(expected(&error), "{name}: {error:?}");
    }
}

#[test]
fn test_field_constructors_refuse_invalid_values() {
    assert!(AssetCode::new("").is_err());
    assert!(AssetCode::new("usdt").is_err());
    assert!(AssetCode::new("ABCDEFGHIJKLMNOPQ").is_err());
    assert!(InstrumentId::new("").is_err());
    assert!(ContractSize::base(Decimal::ZERO, a("BTC")).is_err());
    assert!(TickSchedule::new(vec![]).is_err());
    assert!(TickSchedule::fixed(Decimal::ZERO).is_err());
    let band = |up_to: Option<i64>, tick: i64| TickBand {
        up_to: up_to.map(|u| d(u, 3)),
        tick: d(tick, 4),
    };
    // The last band must be unbounded, and only it.
    assert!(TickSchedule::new(vec![band(Some(5), 1)]).is_err());
    assert!(TickSchedule::new(vec![band(None, 1), band(None, 5)]).is_err());
    // Bounds must increase.
    assert!(TickSchedule::new(vec![band(Some(5), 1), band(Some(5), 5), band(None, 5)]).is_err());
    assert!(Ratio::new(Decimal::ONE, Decimal::ZERO).is_err());
}

#[test]
fn test_serde_round_trips_and_validates() {
    let spec = reference::deribit::btc_option(
        InstrumentId::new("BTC-27NOV26-100000-C").unwrap(),
        1_795_766_400_000,
        Decimal::from(100_000),
        OptionStyle::Call,
    )
    .unwrap();
    let json = serde_json::to_string(&spec).unwrap();
    // Decimals travel as exact strings, never floats.
    assert!(json.contains("\"strike\":\"100000\""), "{json}");
    assert!(json.contains("\"tick\":\"0.0001\""), "{json}");
    let back: InstrumentSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back, spec);

    // Deserialization goes through validation: a perpetual with an
    // expiry, a bad asset code, a zero denominator are refused.
    let mut value: serde_json::Value =
        serde_json::to_value(reference::deribit::btc_perpetual().unwrap()).unwrap();
    value["expiry_ms"] = serde_json::json!(1);
    assert!(serde_json::from_value::<InstrumentSpec>(value.clone()).is_err());
    value["expiry_ms"] = serde_json::Value::Null;
    value["underlying"] = serde_json::json!("btc");
    assert!(serde_json::from_value::<InstrumentSpec>(value).is_err());
    assert!(serde_json::from_str::<Ratio>(r#"{"num":"1","den":"0"}"#).is_err());
    let ratio: Ratio = serde_json::from_str(r#"{"num":"1","den":"-3"}"#).unwrap();
    assert_eq!(
        (ratio.num(), ratio.den()),
        (Decimal::from(-1), Decimal::from(3))
    );
    assert!(serde_json::from_str::<TickSchedule>(r#"[{"up_to":"1","tick":"0.1"}]"#).is_err());

    let money = MoneyRatio::new(Ratio::new(d(1, 0), d(3, 0)).unwrap(), a("BTC"));
    let rounded = money.settle(8, RoundingRule::HalfEven).unwrap();
    let json = serde_json::to_string(&rounded).unwrap();
    assert!(json.contains("\"0.33333333\""), "{json}");
}

#[test]
fn test_direction_conversions() {
    assert_eq!(Action::from(Side::Long), Action::Buy);
    assert_eq!(Action::from(Side::Short), Action::Sell);
    assert_eq!(Side::try_from(Action::Buy).unwrap(), Side::Long);
    assert_eq!(Side::try_from(Action::Sell).unwrap(), Side::Short);
    assert!(matches!(
        Side::try_from(Action::Other),
        Err(InstrumentError::Unsupported { .. })
    ));
}

#[test]
fn test_new_enums_are_one_byte() {
    assert_eq!(core::mem::size_of::<ContractKind>(), 1);
    assert_eq!(core::mem::size_of::<Margining>(), 1);
    assert_eq!(core::mem::size_of::<SizeKind>(), 1);
    assert_eq!(core::mem::size_of::<ExerciseStyle>(), 1);
    assert_eq!(core::mem::size_of::<SettlementType>(), 1);
    assert_eq!(core::mem::size_of::<RoundingRule>(), 1);
}

#[test]
fn test_unsupported_expiries_are_typed() {
    let mut draft: InstrumentDraft = reference::deribit::btc_option(
        InstrumentId::new("X").unwrap(),
        1,
        Decimal::from(100),
        OptionStyle::Call,
    )
    .unwrap()
    .into();
    // Inverse physical: refused at construction.
    draft.option = Some(
        OptionTerms::new(
            Decimal::from(100),
            OptionStyle::Call,
            ExerciseStyle::European,
            SettlementType::Physical,
            a("BTC"),
        )
        .unwrap(),
    );
    assert!(matches!(
        InstrumentSpec::new(draft.clone()),
        Err(InstrumentError::InvalidCombination { .. })
    ));
    draft.option = Some(
        OptionTerms::new(
            Decimal::from(100),
            OptionStyle::Call,
            ExerciseStyle::American,
            SettlementType::Cash,
            a("BTC"),
        )
        .unwrap(),
    );
    let american = InstrumentSpec::new(draft).unwrap();
    assert!(matches!(
        american.option_expiry(Side::Long, Decimal::ONE, Decimal::from(120)),
        Err(InstrumentError::Unsupported { .. })
    ));
    // Cash ITM stays an exact ratio in the underlying.
    let call = reference::deribit::btc_option(
        InstrumentId::new("Y").unwrap(),
        1,
        Decimal::from(100),
        OptionStyle::Call,
    )
    .unwrap();
    let ExpiryOutcome::Cash(v) = call
        .option_expiry(Side::Long, Decimal::ONE, Decimal::from(120))
        .unwrap()
    else {
        panic!("cash expected");
    };
    assert!(
        v.ratio
            .equals(&Ratio::new(d(1, 0), d(6, 0)).unwrap())
            .unwrap()
    );
    assert_eq!(v.asset, a("BTC"));
}

#[test]
fn test_decimals_deserialize_only_from_exact_strings() {
    use financial_types::instrument::Money;
    // A value Decimal would round to 0, and a JSON number: both refused.
    assert!(
        serde_json::from_str::<Money>(
            r#"{"amount":"0.00000000000000000000000000001","asset":"BTC"}"#
        )
        .is_err()
    );
    assert!(serde_json::from_str::<Money>(r#"{"amount":0.1,"asset":"BTC"}"#).is_err());
    assert!(serde_json::from_str::<Ratio>(r#"{"num":0.1,"den":"1"}"#).is_err());
    assert!(serde_json::from_str::<Ratio>(r#"{"num":"1e5","den":"1"}"#).is_err());
    assert!(
        serde_json::from_str::<Ratio>(r#"{"num":"9.0000000000000000000000000001","den":"1"}"#)
            .is_err()
    );
    let ok: Money = serde_json::from_str(r#"{"amount":"-0.01875","asset":"BTC"}"#).unwrap();
    assert_eq!(ok.amount, d(-1875, 5));

    // Nested paths of a spec: strike, tick band bound, tick, contract size
    // and lot.
    let spec = reference::deribit::btc_option(
        InstrumentId::new("BTC-27NOV26-100000-C").unwrap(),
        1,
        Decimal::from(100_000),
        OptionStyle::Call,
    )
    .unwrap();
    let base: serde_json::Value = serde_json::to_value(&spec).unwrap();
    let tiny = serde_json::json!("0.00000000000000000000000000001");
    let paths: [&[&str]; 4] = [
        &["option", "strike"],
        &["size", "amount"],
        &["lot"],
        &["price_ticks"],
    ];
    for path in paths {
        let mut v = base.clone();
        let mut slot = &mut v;
        for key in path {
            slot = &mut slot[*key];
        }
        if path == ["price_ticks"] {
            slot[0]["up_to"] = tiny.clone();
        } else {
            *slot = tiny.clone();
        }
        assert!(
            serde_json::from_value::<InstrumentSpec>(v.clone()).is_err(),
            "{path:?}"
        );
        // And a JSON number in the same place.
        if path != ["price_ticks"] {
            let mut n = base.clone();
            let mut slot = &mut n;
            for key in path {
                slot = &mut slot[*key];
            }
            *slot = serde_json::json!(0.5);
            assert!(
                serde_json::from_value::<InstrumentSpec>(n).is_err(),
                "{path:?}"
            );
        }
    }
    let mut t = base.clone();
    t["price_ticks"][1]["tick"] = serde_json::json!(0.0005);
    assert!(serde_json::from_value::<InstrumentSpec>(t).is_err());
}

#[test]
fn test_accessors_and_display() {
    let btc = a("BTC");
    assert_eq!(btc.to_string(), "BTC");
    assert_eq!(btc.as_str(), "BTC");
    let id = InstrumentId::new("BTC-PERPETUAL").unwrap();
    assert_eq!(id.to_string(), "BTC-PERPETUAL");
    assert_eq!(id.as_str(), "BTC-PERPETUAL");
    let option = reference::deribit::btc_option(
        InstrumentId::new("BTC-27NOV26-100000-C").unwrap(),
        1_795_766_400_000,
        Decimal::from(100_000),
        OptionStyle::Call,
    )
    .unwrap();
    assert_eq!(option.lot(), d(1, 1));
    assert_eq!(option.expiry_ms(), Some(1_795_766_400_000));
    assert_eq!(option.price_ticks().bands().len(), 2);
    assert_eq!(option.price_ticks().bands()[0].up_to, Some(d(5, 3)));
    assert_eq!(
        reference::deribit::btc_perpetual().unwrap().expiry_ms(),
        None
    );
    // A zero premium is allowed (an option quoted at 0), and is 0 on both
    // sides, never -0.
    for side in [Side::Long, Side::Short] {
        let zero = option
            .option_premium(side, Decimal::ONE, Decimal::ZERO)
            .unwrap();
        assert_eq!(zero.amount, Decimal::ZERO);
        assert_eq!(
            serde_json::to_string(&zero).unwrap(),
            r#"{"amount":"0","asset":"BTC"}"#
        );
    }
}

#[cfg(feature = "fixtures")]
#[test]
fn test_check_refuses_a_wrong_expectation() {
    use financial_types::instrument::Money;
    use financial_types::instrument::fixtures::{Case, ErrorKind, Expected, cases, check};
    let all = cases().unwrap();
    let find = |name: &str| -> Case {
        all.iter()
            .find(|c| c.name.starts_with(name))
            .cloned()
            .unwrap_or_else(|| panic!("no case {name}"))
    };
    let wrong = |mut case: Case, expected: Expected| {
        case.expected = expected;
        assert!(
            check(&case).is_err(),
            "{} accepted a wrong expectation",
            case.name
        );
    };
    // An exact amount, its settlement and its residual.
    let pnl = find("inverse pnl long");
    wrong(
        pnl.clone(),
        Expected::Amount {
            exact: MoneyRatio::new(Ratio::new(d(1, 0), d(365, 0)).unwrap(), a("BTC")),
            settled: None,
        },
    );
    let Expected::Amount {
        exact,
        settled: Some(mut s),
    } = pnl.expected.clone()
    else {
        panic!("amount expected");
    };
    s.amount = d(273_225, 8);
    wrong(
        pnl.clone(),
        Expected::Amount {
            exact: exact.clone(),
            settled: Some(s.clone()),
        },
    );
    s.amount = d(273_224, 8);
    s.residual = Ratio::new(d(2, 0), d(2_287_500_000, 0)).unwrap();
    wrong(
        pnl,
        Expected::Amount {
            exact,
            settled: Some(s),
        },
    );
    // An error of another kind, and an error where a value is expected.
    let overflow = find("Decimal::MAX contracts");
    wrong(overflow, Expected::Error(ErrorKind::Inexact));
    wrong(
        find("linear notional"),
        Expected::Error(ErrorKind::Overflow),
    );
    // A premium, a delivery leg, an abandoned option and a settlement.
    wrong(
        find("option premium long"),
        Expected::Money(Money::new(d(-1874, 5), a("BTC"))),
    );
    wrong(
        find("illustrative physical call long"),
        Expected::Delivery {
            underlying: Money::new(d(3, 0), a("BTC")),
            cash: Money::new(d(-301, 0), a("USDT")),
        },
    );
    wrong(
        find("illustrative physical call long"),
        Expected::Delivery {
            underlying: Money::new(d(4, 0), a("BTC")),
            cash: Money::new(d(-300, 0), a("USDT")),
        },
    );
    wrong(
        find("illustrative physical call out of the money"),
        Expected::Valid,
    );
    let tie = find("tie 3/8");
    let Expected::Settled(mut s) = tie.expected.clone() else {
        panic!("settled expected");
    };
    s.amount = d(37, 2);
    wrong(tie.clone(), Expected::Settled(s.clone()));
    s.amount = d(38, 2);
    s.residual = Ratio::new(d(1, 0), d(200, 0)).unwrap();
    wrong(tie, Expected::Settled(s));
}
