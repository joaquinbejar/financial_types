//! Conversions between [`Action`] and [`Side`].
//!
//! The instrument formulas take a [`Side`]. Consumers that hold an
//! [`Action`] convert it here rather than each in its own way.

use super::error::InstrumentError;
use crate::{Action, Side};

impl From<Side> for Action {
    /// [`Side::Long`] is [`Action::Buy`], [`Side::Short`] is [`Action::Sell`].
    #[inline]
    fn from(side: Side) -> Self {
        match side {
            Side::Long => Self::Buy,
            Side::Short => Self::Sell,
        }
    }
}

impl TryFrom<Action> for Side {
    type Error = InstrumentError;

    /// [`Action::Buy`] is [`Side::Long`], [`Action::Sell`] is
    /// [`Side::Short`].
    ///
    /// # Errors
    ///
    /// [`InstrumentError::Unsupported`] for an action without a direction
    /// ([`Action::Other`]).
    fn try_from(action: Action) -> Result<Self, Self::Error> {
        match action {
            Action::Buy => Ok(Self::Long),
            Action::Sell => Ok(Self::Short),
            Action::Other => Err(InstrumentError::unsupported(
                "an action without a direction",
            )),
        }
    }
}
