//! Types used for Fixed-point operations. Uses [`fixnum::FixedPoint`].
#![allow(clippy::std_instead_of_core)]

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

use iroha_schema::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod llvm_opt {
    #[inline]
    #[cold]
    const fn cold() {}

    #[inline]
    pub(crate) const fn likely(b: bool) -> bool {
        if !b {
            cold()
        }
        b
    }

    #[inline]
    pub(crate) const fn unlikely(b: bool) -> bool {
        if b {
            cold()
        }
        b
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Decode,
    Encode,
    Deserialize,
    Serialize,
    IntoSchema,
)]
pub struct Fixed {
    /// The integral part of the binary coded decimal. Treat it as
    /// your ordinary `u64`.
    integral: u64,
    /// The fractional part of the Binary coded decimal. It's value
    /// ranges from `0` to `9999999999999999999`, which is the last
    /// fully represented decimal number between `0` and `u64::MAX`.
    ///
    /// ## TODO:
    ///
    /// This should be its own type and it should be annotated with
    /// niches.
    decimal_fraction: DecimalFraction,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Decode,
    Deserialize,
    Encode,
    Serialize,
    IntoSchema,
)]
pub struct DecimalFraction(u64);

impl core::fmt::Debug for DecimalFraction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: remove trailing zeros
        write!(f, ".{:019}", self.0)
    }
}

impl core::fmt::Display for DecimalFraction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: remove trailing zeros
        write!(f, ".{:019}", self.0)
    }
}

impl core::fmt::Debug for Fixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{:?}", self.integral, self.decimal_fraction)
    }
}

impl core::fmt::Display for Fixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{:?}", self.integral, self.decimal_fraction)
    }
}


impl DecimalFraction {
    /// Maximum value of the inner `u64`
    pub const MAX_RAW: u64 = 9_999_999_999_999_999_999;

    /// Maximum value scaled properly and represented as a floating point number
    pub const MAX: f64 = 0.999_999_999_999_999_999_9; // TODO: This is lossy.

    /// An _extremely_ dirty way to convert to a float
    #[inline]
    pub fn to_float_lossy(self) -> f64 {
        self.0 as f64 / ((Self::MAX_RAW + 1) as f64)
    }

    /// Minimum value
    pub const MIN: Self = Self::ZERO;

    /// Zero, which is also the [`Self::MIN`]
    pub const ZERO: Self = DecimalFraction(0);
}

impl Fixed {
    /// Zero, which is also the [`Self::MIN`]
    pub const ZERO: Fixed = Fixed {
        integral: 0,
        decimal_fraction: DecimalFraction::ZERO,
    };

    /// Minimum value
    pub const MIN: Self = Self::ZERO;

    /// Maximum value
    pub const MAX: Self = Fixed {
        integral: u64::MAX,
        decimal_fraction: DecimalFraction(DecimalFraction::MAX_RAW),
    };

    #[inline]
    pub const unsafe fn from_raw_unchecked(integral: u64, fractional: u64) -> Self {
        Self {
            integral,
            decimal_fraction: DecimalFraction(fractional),
        }
    }

    /// Convert with loss of precision.
    #[inline]
    pub fn to_float_lossy(self) -> f64 {
        self.integral as f64 + self.decimal_fraction.to_float_lossy()
    }

    /// Checked addition
    ///
    /// # Errors
    /// Return [`None`] If addition overflows.
    #[inline]
    pub const fn checked_add(
        self,
        Self {
            integral,
            decimal_fraction,
        }: Self,
    ) -> Option<Self> {
        // This is a hot path and a `const` function. So `?` operator not used.
        let fractional = (self.decimal_fraction.0 as u128) + (decimal_fraction.0 as u128);
        let carry = (fractional / (DecimalFraction::MAX_RAW as u128 + 1)) as u64; // does not overflow
        let updated_fractional = (fractional % (DecimalFraction::MAX_RAW as u128 + 1)) as u64; // Does not overflow either
        let (acc, overflowed) = self.integral.overflowing_add(integral);
        // If we're on nightly, we can use `std::intrinsics`
        if llvm_opt::unlikely(overflowed) {
            return None;
        }
        let (integral, overflowed_again) = acc.overflowing_add(carry);
        if llvm_opt::unlikely(overflowed_again) {
            None
        } else {
            Some(Self {
                integral,
                decimal_fraction: DecimalFraction(updated_fractional),
            })
        }
    }

    /// Checked subtraction
    ///
    /// # Errors
    /// Return [`None`] If addition overflows (i.e. produces a negative value)
    #[inline]
    pub const fn checked_sub(
        self,
        Self {
            integral,
            decimal_fraction,
        }: Self,
    ) -> Option<Self> {
        // This is a hot path and a `const` function. So `?` operator not used.
        let fractional = (self.decimal_fraction.0 as i128) - (decimal_fraction.0 as i128);
        let (carry, updated_fractional) = if fractional >= 0 {
            // We already checked for negativity, and it's unsigned,
            // so range is compressed
            (0, fractional as u64)
        } else {
            // At worst, `fractional` is negative
            // `DecimalFraction::MAX_RAW as i128`.  Computing
            // complement is necessary. 2.1 - 1.2 = 0.9, fractional is
            // -.1, complement is ((MAX=.9) + .1)-.1 = 0.9.
            let complement = fractional + 1_i128 + (DecimalFraction::MAX_RAW as i128);
            (1, complement as u64)
        };
        // This is ugly, because `const` functions don't allow pattern
        // matching and the `?` operator
        let (acc, overflowed) = self.integral.overflowing_sub(integral);
        if llvm_opt::unlikely(overflowed) {
            // If we're on nightly, we can use `std::intrinsics`
            return None;
        }
        let (integral, overflowed_again) = acc.overflowing_sub(carry);
        if llvm_opt::unlikely(overflowed_again) {
            None
        } else {
            Some(Self {
                integral,
                decimal_fraction: DecimalFraction(updated_fractional),
            })
        }
    }

    /// Checked multiplication
    ///
    /// Result is rounded to nearest representable value.
    ///
    /// # Errors
    /// If either of the operands is negative or if the multiplication overflows.
    #[inline]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        // The key insight here is that entropy is additive under
        // multiplication. The largest product that we can attain by
        // multiplying `u64` fits inside `u128`. And because we have a
        // binary coded decimal, all we need to do is to split via div
        // and remainder. The only place where overflow can occur is
        // in the final checked addition.
        let norm: u128 = DecimalFraction::MAX_RAW as u128 + 1;
        let lhs: u128 = (dbg!(self).integral as u128 * dbg!(norm)) + self.decimal_fraction.0 as u128;
        let rhs: u128 = (dbg!(other).integral as u128 * norm) + other.decimal_fraction.0 as u128;
        let prod = lhs.checked_mul(rhs)?;
        Some(Self {
            integral: (dbg!(prod) / dbg!((norm * norm))) as u64,
            decimal_fraction: DecimalFraction((prod % (norm * norm)) as u64)
        })
    }

    /// Checked division
    ///
    /// Result is rounded to nearest representable value.
    ///
    /// # Errors
    /// If either of the operands is negative or if the multiplication overflows.
    #[inline]
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        todo!()
    }
}

/// Custom error type for Fixed point operation errors.
#[allow(variant_size_differences)]
#[derive(Debug, derive_more::Display, Clone, iroha_macro::FromVariant)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum FixedPointOperationError {
    /// All [`Fixed`] values should be positive.
    #[display(fmt = "Negative value not allowed")]
    NegativeValue,
    /// Conversion failed.
    #[display(fmt = "Failed to produce fixed point number")]
    Conversion(#[cfg_attr(feature = "std", source)] fixnum::ConvertError),
    /// Overflow
    #[display(fmt = "Overflow")]
    Overflow,
    /// Division by zero
    #[display(fmt = "Division by zero")]
    DivideByZero,
    /// Domain violation. E.g. computing `sqrt(-1)`
    #[display(fmt = "Domain violation")]
    DomainViolation,
    /// Arithmetic
    #[display(fmt = "Unknown Arithmetic error")]
    Arithmetic,
}

impl TryFrom<f64> for Fixed {
    type Error = FixedPointOperationError;

    #[inline]
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if llvm_opt::unlikely(value < 0_f64) {
            return Err(FixedPointOperationError::NegativeValue);
        } else if llvm_opt::unlikely(value > u64::MAX as f64) {
            return Err(FixedPointOperationError::Overflow);
        } else {
            let integral: u64 = value.trunc() as u64;
            let fraction: u64 = (value.fract() * ((DecimalFraction::MAX_RAW + 1) as f64)) as u64;
            Ok(Self {
                integral,
                decimal_fraction: DecimalFraction(fraction),
            })
        }
    }
}

mod ffi {
    #![allow(unsafe_code)]
    use super::*;

    // SAFETY: Type is robust with respect to the inner type
    unsafe impl iroha_ffi::ir::InfallibleTransmute for Fixed {}

    // iroha_ffi::ffi_type! {unsafe impl Transparent for Fixed[Base] validated with {|_| true}}
}

/// Export of inner items.
pub mod prelude {
    pub use super::Fixed;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::restriction, clippy::panic)]
    use super::*;

    fn fixed(a: u64, b: u64) -> Fixed {
        unsafe { Fixed::from_raw_unchecked(a, b) }
    }

    #[test]
    fn add_sub_neutral_element() {
        assert_eq!(fixed(1, 2).checked_add(fixed(0, 0)), Some(fixed(1, 2)));
        assert_eq!(
            fixed(u64::MAX, 2).checked_add(fixed(0, 0)),
            Some(fixed(u64::MAX, 2))
        );
        assert_eq!(
            fixed(u64::MAX, DecimalFraction::MAX_RAW + 1).checked_add(Fixed::ZERO),
            None,
            "Adding neutral element to malformed addition is malformed"
        );
        assert_eq!(
            fixed(u64::MAX, DecimalFraction::MAX_RAW).checked_add(Fixed::ZERO),
            Some(fixed(u64::MAX, DecimalFraction::MAX_RAW)),
            "Adding neutral element to the last valid value is valid"
        );
        assert_eq!(fixed(1, 2).checked_sub(fixed(0, 0)), Some(fixed(1, 2)));
        assert_eq!(
            fixed(u64::MAX, 2).checked_sub(fixed(0, 0)),
            Some(fixed(u64::MAX, 2))
        );
        // **ATTENTION**: This is why `from_raw_unchecked` is `unsafe`
        assert_eq!(
            fixed(u64::MAX, DecimalFraction::MAX_RAW + 1).checked_sub(Fixed::ZERO),
            Some(fixed(u64::MAX, DecimalFraction::MAX_RAW + 1)),
            "Subtraction does not itself check for validity."
        );
        assert_eq!(
            fixed(u64::MAX, DecimalFraction::MAX_RAW).checked_sub(Fixed::ZERO),
            Some(fixed(u64::MAX, DecimalFraction::MAX_RAW)),
            "Subtraction from the last correct value should be correct"
        );
    }

    #[test]
    fn mul_div_neutral_element() {
        assert_eq!(Fixed::try_from(1.0).expect("1.2").checked_mul(fixed(1,0)), Some(fixed(1,0)));
        assert_eq!(fixed(2, 0).checked_mul(fixed(1, 0)), Some(fixed(2, 0)));
        assert_eq!(fixed(0, DecimalFraction::MAX_RAW).checked_mul(fixed(1, 0)), Some(fixed(0, DecimalFraction::MAX_RAW)));
        assert_eq!(fixed(u64::MAX, DecimalFraction::MAX_RAW).checked_mul(fixed(1, 0)), Some(fixed(u64::MAX, DecimalFraction::MAX_RAW)));
        // assert_eq!(Fixed::try_from(1.0).expect("1.2").checked_div(fixed(1,0)), Some(fixed(1,0)));
        // assert_eq!(fixed(0, DecimalFraction::MAX_RAW).checked_div(fixed(1, 0)), Some(fixed(0, DecimalFraction::MAX_RAW)));
        // assert_eq!(fixed(u64::MAX, DecimalFraction::MAX_RAW).checked_div(fixed(1, 0)), Some(fixed(u64::MAX, DecimalFraction::MAX_RAW)));
    }


    #[test]
    fn add_sub_semantics() {
        assert_eq!(
            fixed(0, 2).checked_sub(fixed(0, 1)),
            Some(fixed(0, 1)),
            "Note this is NOT true of floating point numbers or _binary_ fixed precision"
        );
        assert_eq!(
            fixed(u64::MAX, DecimalFraction::MAX_RAW - 30).checked_add(fixed(0, 29)),
            Some(fixed(u64::MAX, DecimalFraction::MAX_RAW - 1))
        );
        let one = fixed(1, 0);
        let zero = Fixed::ZERO;
        let two = fixed(2, 0);
        let three = fixed(3, 0);
        assert_eq!(two.checked_add(one), Some(three));
        assert_eq!(two.checked_sub(one), Some(one));
        assert_eq!(two.checked_sub(two), Some(zero));
        assert_eq!(one.checked_sub(zero), Some(one));
        assert_eq!(zero.checked_sub(zero), Some(zero));
        assert_eq!(two.checked_sub(three), None, "Negative number as a result");
        assert_eq!(
            one.checked_add(Fixed::try_from(1_f64).expect("1"))
                .expect("2")
                .checked_sub(two),
            Some(Fixed::ZERO),
            "Negative number results"
        );
    }

    #[test]
    fn mul_div_semantics() {
        assert_eq!(fixed(1,0).checked_mul(fixed(1,2)), Some(fixed(1,2)));
    }


    #[test]
    fn negative_numbers() {
        assert!(matches!(
            Fixed::try_from(-123.45_f64),
            Err(FixedPointOperationError::NegativeValue)
        ));
    }

    #[test]
    fn deserialize_positive_from_json_should_succeed() {
        let initial = fixed(2, 234);
        let serialized =
            serde_json::to_string(&initial).expect("Should be possible to serialize any `Fixed`");
        let fixed: Fixed =
            serde_json::from_str(&serialized).expect("Should be possible to deserialize");
        assert_eq!(fixed, initial);
        let serialized = serde_json::to_string(&Fixed::ZERO)
            .expect("Should be possible to serialize any `Fixed`");
        let fixed: Fixed =
            serde_json::from_str(&serialized).expect("Should be possible to deserialize");
        assert_eq!(fixed, Fixed::ZERO);
    }

    #[test]
    fn encode_decode_parity_scale() {
        let initial = fixed(2, 234);
        let encoded = initial.encode();
        let fixed: Fixed =
            Fixed::decode(&mut encoded.as_slice()).expect("Should be possible to decode");
        assert_eq!(fixed, initial);
        let encoded = Fixed::ZERO.encode();
        let fixed: Fixed =
            Fixed::decode(&mut encoded.as_slice()).expect("Should be possible to decode");
        assert_eq!(fixed, Fixed::ZERO);
    }

    #[test]
    fn precision_loss() {
        assert_eq!(
            Fixed::try_from(0.2_f64).expect("Valid").to_float_lossy(),
            0.2_f64,
            "Compilers are cool, and this actually optimises to what you expect"
        );
        assert_eq!(
            fixed(0, 1234567891011).to_float_lossy(),
            1.234567891011e-7,
            "52 bits of precision"
        );
        assert_eq!(
            fixed(0, 12345678910111213141).to_float_lossy(),
            1.2345678910111213141,
            "A little more than 52 bits of precision, passes because of compiler magic"
        );
        assert_eq!(
            fixed(0, 12345678910111213141).to_float_lossy(),
            1.234_567_891_011_121_314 + 0.000_000_000_000_000_000_1,
            "A little more than 52 bits of precision, passes because of compiler magic"
        );
        assert_eq!(
            fixed(1, 020_000_000_000_000_000_0).checked_sub(fixed(1, 010_000_000_000_000_000_0)),
            Some(fixed(0, 010_000_000_000_000_000_0))
        );
        assert_eq!(
            fixed(1, 020_000_000_000_000_000_0).to_float_lossy()
                - (fixed(1, 010_000_000_000_000_000_0).to_float_lossy()),
            0.010000000000000009f64,
            "Note the nine. This is why we bother with this crate. It's also the case with non--binary-coded-decimal"
        )
    }
}
