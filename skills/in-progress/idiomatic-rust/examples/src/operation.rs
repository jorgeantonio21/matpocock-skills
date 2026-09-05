//! An invariant-preserving operation: negating a nonzero value keeps it nonzero, and can overflow.
//!
//! 1. `Offset` is never zero, so `sign` is total and `magnitude` is nonzero.
//! 2. `checked_neg` returns `Option<Self>`. Nonzero survives negation, but `i32::MIN` has no negation.
//! 3. The return type states both facts. A plain `-x` panics in debug and wraps in release.

use std::num::{NonZeroI32, NonZeroU32};

/// A signed, nonzero displacement. The caller expresses "no move" as `None`, not as a zero.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Offset(NonZeroI32);

/// The direction of an [`Offset`]. Total, because an offset is never zero.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sign {
    /// The offset is below zero.
    Negative,
    /// The offset is above zero.
    Positive,
}

impl Offset {
    /// Returns the offset, or `None` when `value` is zero.
    #[must_use]
    pub const fn new(value: i32) -> Option<Self> {
        match NonZeroI32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// The offset as a plain number.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0.get()
    }

    /// The direction. Every offset has one.
    #[must_use]
    pub const fn sign(self) -> Sign {
        if self.0.get() < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    /// The distance, which is nonzero because the offset is. `i32::MIN` fits: its magnitude is `2^31`.
    #[must_use]
    pub const fn magnitude(self) -> NonZeroU32 {
        self.0.unsigned_abs()
    }

    /// The opposite offset, or `None` for `i32::MIN`, whose negation does not fit in an `i32`.
    #[must_use]
    pub const fn checked_neg(self) -> Option<Self> {
        match self.0.checked_neg() {
            Some(negated) => Some(Self(negated)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_of_min_reports_overflow_instead_of_panicking() -> anyhow::Result<()> {
        let min = Offset::new(i32::MIN).ok_or_else(|| anyhow::anyhow!("i32::MIN is nonzero"))?;
        assert_eq!(min.checked_neg(), None);
        // -1 negates to 1
        let minus_one = Offset::new(-1).ok_or_else(|| anyhow::anyhow!("-1 is nonzero"))?;
        assert_eq!(minus_one.checked_neg().map(Offset::get), Some(1));
        Ok(())
    }

    #[test]
    fn test_magnitude_of_min_is_two_to_the_31() -> anyhow::Result<()> {
        let min = Offset::new(i32::MIN).ok_or_else(|| anyhow::anyhow!("i32::MIN is nonzero"))?;
        // |-2^31| = 2^31 = 2 147 483 648, above i32::MAX and inside u32
        assert_eq!(min.magnitude().get(), 2_147_483_648);
        Ok(())
    }

    #[test]
    fn test_sign_is_total() {
        assert_eq!(Offset::new(-7).map(Offset::sign), Some(Sign::Negative));
        assert_eq!(Offset::new(7).map(Offset::sign), Some(Sign::Positive));
        assert_eq!(Offset::new(0), None);
    }
}
