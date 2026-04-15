//! Parse an `Action` from a string supplied on the command line.
//!
//! Usage:
//!
//! ```bash
//! cargo run --example parse_action -- buy
//! cargo run --example parse_action -- sell
//! cargo run --example parse_action -- "  Other  "
//! ```

use financial_types::Action;
use std::env;
use std::process::ExitCode;
use std::str::FromStr;

fn main() -> ExitCode {
    let raw = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("usage: parse_action <buy|sell|other>");
            return ExitCode::from(2);
        }
    };

    match Action::from_str(&raw) {
        Ok(action) => {
            println!("parsed: {action} (discriminant = {})", action as u8);
            println!("opposite: {}", action.opposite());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
