// this_file: backends/typf-orge/src/fixed.rs

//! F26Dot6 fixed-point arithmetic for scan conversion.
//!
//! Uses 26.6 format: 26 bits integer, 6 bits fractional.
//! This provides 1/64 pixel precision for sub-pixel positioning.

use std::ops::{Add, Neg, Sub};

/// 26.6 fixed-point number.
///
/// Internal representation: 32-bit signed integer where the lower 6 bits
/// represent the fractional part (1/64 units).
///
/// # Examples
///
/// ```
/// use typf_orge::fixed::F26Dot6;
///
/// let x = F26Dot6::from_int(5);
/// assert_eq!(x.to_int(), 5);
///
/// let y = F26Dot6::from_float(5.5);
/// assert_eq!(y.to_int_round(), 6);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct F26Dot6(i32);

impl F26Dot6 {
    /// Fractional bits (6 bits = 64 units per integer).
    pub const FRAC_BITS: u32 = 6;

    /// Fractional mask (0b111111 = 63).
    pub const FRAC_MASK: i32 = (1 << Self::FRAC_BITS) - 1;

    /// One unit (1.0 in 26.6 format = 64).
    pub const ONE: F26Dot6 = F26Dot6(1 << Self::FRAC_BITS);

    /// Zero.
    pub const ZERO: F26Dot6 = F26Dot6(0);

    /// Half (0.5 in 26.6 format = 32).
    pub const HALF: F26Dot6 = F26Dot6(1 << (Self::FRAC_BITS - 1));

    /// Create from integer value.
    #[inline]
    pub const fn from_int(x: i32) -> Self {
        F26Dot6(x << Self::FRAC_BITS)
    }

    /// Create from floating-point value.
    #[inline]
    pub fn from_float(x: f32) -> Self {
        F26Dot6((x * 64.0) as i32)
    }

    /// Convert to integer (truncate fractional part).
    #[inline]
    pub const fn to_int(self) -> i32 {
        // Right shift with arithmetic shift (rounds toward negative infinity)
        // For scan conversion, this is actually fine
        self.0 >> Self::FRAC_BITS
    }

    /// Convert to integer (round to nearest).
    #[inline]
    pub const fn to_int_round(self) -> i32 {
        // Add half and truncate (works for both positive and negative)
        (self.0 + Self::HALF.0) >> Self::FRAC_BITS
    }

    /// Get fractional part (0-63).
    #[inline]
    pub const fn frac(self) -> i32 {
        self.0 & Self::FRAC_MASK
    }

    /// Convert to floating-point.
    #[inline]
    pub fn to_float(self) -> f32 {
        self.0 as f32 / 64.0
    }

    /// Multiply two F26Dot6 values.
    ///
    /// Result = (a * b) / 64
    #[inline]
    pub const fn mul(self, other: F26Dot6) -> F26Dot6 {
        F26Dot6(((self.0 as i64 * other.0 as i64) >> Self::FRAC_BITS) as i32)
    }

    /// Divide two F26Dot6 values.
    ///
    /// Result = (a * 64) / b
    #[inline]
    pub const fn div(self, other: F26Dot6) -> F26Dot6 {
        F26Dot6((((self.0 as i64) << Self::FRAC_BITS) / other.0 as i64) as i32)
    }

    /// Absolute value.
    #[inline]
    pub const fn abs(self) -> F26Dot6 {
        F26Dot6(self.0.abs())
    }

    /// Floor (round down to nearest integer).
    #[inline]
    pub const fn floor(self) -> F26Dot6 {
        // Masking clears the fractional bits, which gives floor for positive
        // For negative, masking already rounds down (toward negative infinity)
        F26Dot6(self.0 & !(Self::FRAC_MASK))
    }

    /// Ceiling (round up to nearest integer).
    #[inline]
    pub const fn ceil(self) -> F26Dot6 {
        if self.0 & Self::FRAC_MASK == 0 {
            self
        } else {
            F26Dot6((self.0 & !(Self::FRAC_MASK)) + Self::ONE.0)
        }
    }

    /// Get raw internal value.
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Create from raw internal value.
    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        F26Dot6(raw)
    }
}

impl Add for F26Dot6 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        F26Dot6(self.0 + other.0)
    }
}

impl Sub for F26Dot6 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        F26Dot6(self.0 - other.0)
    }
}

impl Neg for F26Dot6 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        F26Dot6(-self.0)
    }
}

impl From<i32> for F26Dot6 {
    #[inline]
    fn from(x: i32) -> Self {
        Self::from_int(x)
    }
}

impl From<f32> for F26Dot6 {
    #[inline]
    fn from(x: f32) -> Self {
        Self::from_float(x)
    }
}

impl From<F26Dot6> for f32 {
    #[inline]
    fn from(x: F26Dot6) -> f32 {
        x.to_float()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(F26Dot6::ZERO.raw(), 0);
        assert_eq!(F26Dot6::ONE.raw(), 64);
        assert_eq!(F26Dot6::HALF.raw(), 32);
    }

    #[test]
    fn test_from_int() {
        assert_eq!(F26Dot6::from_int(0).raw(), 0);
        assert_eq!(F26Dot6::from_int(1).raw(), 64);
        assert_eq!(F26Dot6::from_int(5).raw(), 320);
        assert_eq!(F26Dot6::from_int(-3).raw(), -192);
    }

    #[test]
    fn test_from_float() {
        assert_eq!(F26Dot6::from_float(0.0).raw(), 0);
        assert_eq!(F26Dot6::from_float(1.0).raw(), 64);
        assert_eq!(F26Dot6::from_float(0.5).raw(), 32);
        assert_eq!(F26Dot6::from_float(5.25).raw(), 336);
        assert_eq!(F26Dot6::from_float(-2.5).raw(), -160);
    }

    #[test]
    fn test_to_int() {
        assert_eq!(F26Dot6::from_int(5).to_int(), 5);
        assert_eq!(F26Dot6::from_float(5.75).to_int(), 5);
        // Arithmetic right shift rounds toward negative infinity
        assert_eq!(F26Dot6::from_float(-3.25).to_int(), -4);
    }

    #[test]
    fn test_to_int_round() {
        assert_eq!(F26Dot6::from_float(5.25).to_int_round(), 5);
        assert_eq!(F26Dot6::from_float(5.5).to_int_round(), 6);
        assert_eq!(F26Dot6::from_float(5.75).to_int_round(), 6);
        // Negative rounding (note: 0.5 ties round toward positive infinity)
        assert_eq!(F26Dot6::from_float(-3.25).to_int_round(), -3);
        assert_eq!(F26Dot6::from_float(-3.5).to_int_round(), -3); // Tie breaks up
        assert_eq!(F26Dot6::from_float(-3.75).to_int_round(), -4);
    }

    #[test]
    fn test_frac() {
        assert_eq!(F26Dot6::from_float(5.0).frac(), 0);
        assert_eq!(F26Dot6::from_float(5.5).frac(), 32);
        assert_eq!(F26Dot6::from_float(5.25).frac(), 16);
        assert_eq!(F26Dot6::from_float(5.75).frac(), 48);
    }

    #[test]
    fn test_to_float() {
        assert!((F26Dot6::from_float(5.5).to_float() - 5.5).abs() < 0.01);
        assert!((F26Dot6::from_float(-3.25).to_float() + 3.25).abs() < 0.01);
    }

    #[test]
    fn test_add() {
        let a = F26Dot6::from_int(3);
        let b = F26Dot6::from_int(2);
        assert_eq!((a + b).to_int(), 5);

        let a = F26Dot6::from_float(3.5);
        let b = F26Dot6::from_float(2.25);
        assert!((a + b - F26Dot6::from_float(5.75)).abs().raw() < 2);
    }

    #[test]
    fn test_sub() {
        let a = F26Dot6::from_int(5);
        let b = F26Dot6::from_int(3);
        assert_eq!((a - b).to_int(), 2);

        let a = F26Dot6::from_float(5.75);
        let b = F26Dot6::from_float(2.25);
        assert!((a - b - F26Dot6::from_float(3.5)).abs().raw() < 2);
    }

    #[test]
    fn test_neg() {
        let a = F26Dot6::from_int(5);
        assert_eq!((-a).to_int(), -5);

        let a = F26Dot6::from_float(3.5);
        assert!(((-a) + F26Dot6::from_float(3.5)).abs().raw() < 2);
    }

    #[test]
    fn test_mul() {
        let a = F26Dot6::from_int(3);
        let b = F26Dot6::from_int(4);
        assert_eq!(a.mul(b).to_int(), 12);

        let a = F26Dot6::from_float(2.5);
        let b = F26Dot6::from_float(4.0);
        assert!((a.mul(b) - F26Dot6::from_float(10.0)).abs().raw() < 2);
    }

    #[test]
    fn test_div() {
        let a = F26Dot6::from_int(12);
        let b = F26Dot6::from_int(4);
        assert_eq!(a.div(b).to_int(), 3);

        let a = F26Dot6::from_float(10.0);
        let b = F26Dot6::from_float(4.0);
        assert!((a.div(b) - F26Dot6::from_float(2.5)).abs().raw() < 2);
    }

    #[test]
    fn test_abs() {
        assert_eq!(F26Dot6::from_int(5).abs().to_int(), 5);
        assert_eq!(F26Dot6::from_int(-5).abs().to_int(), 5);
        assert_eq!(F26Dot6::from_float(-3.5).abs().to_float(), 3.5);
    }

    #[test]
    fn test_floor() {
        assert_eq!(F26Dot6::from_float(5.75).floor().to_int(), 5);
        assert_eq!(F26Dot6::from_float(5.0).floor().to_int(), 5);
        // Floor rounds down (toward negative infinity)
        assert_eq!(F26Dot6::from_float(-3.25).floor().to_int(), -4);
    }

    #[test]
    fn test_ceil() {
        assert_eq!(F26Dot6::from_float(5.25).ceil().to_int(), 6);
        assert_eq!(F26Dot6::from_float(5.0).ceil().to_int(), 5);
        assert_eq!(F26Dot6::from_float(-3.75).ceil().to_int(), -3);
    }

    #[test]
    fn test_comparison() {
        let a = F26Dot6::from_int(5);
        let b = F26Dot6::from_int(3);
        assert!(a > b);
        assert!(b < a);
        assert_eq!(a, a);
    }

    #[test]
    fn test_from_trait() {
        let a: F26Dot6 = 5i32.into();
        assert_eq!(a.to_int(), 5);

        let b: F26Dot6 = 3.5f32.into();
        assert!((b.to_float() - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_into_f32() {
        let a = F26Dot6::from_float(5.5);
        let f: f32 = a.into();
        assert!((f - 5.5).abs() < 0.01);
    }

    #[test]
    fn test_precision() {
        // Test 1/64 pixel precision
        for i in 0..64 {
            let f = F26Dot6::from_raw(i);
            assert_eq!(f.frac(), i);
        }
    }

    #[test]
    fn test_overflow_safety() {
        // Ensure we don't overflow in normal operations
        let large = F26Dot6::from_int(10000);
        let small = F26Dot6::from_int(2);
        let result = large.mul(small);
        assert_eq!(result.to_int(), 20000);
    }
}
