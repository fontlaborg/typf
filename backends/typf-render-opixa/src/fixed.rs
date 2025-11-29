//! Precision beyond pixels: the mathematics of sub-pixel perfection
//!
//! When 1/64th of a pixel matters, fixed-point arithmetic delivers.
//! Our 26.6 format gives us 6 bits of fractional precision—enough to
//! position text with surgical accuracy while keeping calculations fast.
//! This is the secret sauce that makes rasterized text look smooth at any size.

use std::ops::{Add, Neg, Sub};

/// The perfect balance: integer speed with fraction precision
///
/// F26Dot6 is our compromise between floating-point overhead and integer
/// limitations. We pack 26 bits of integer precision with 6 bits of fractional
/// detail into a single 32-bit integer. The result? Calculations that are
/// both fast and precise enough for professional text rendering.
///
/// # The Precision Dance
///
/// ```rust
/// use typf_render_opixa::fixed::F26Dot6;
///
/// let x = F26Dot6::from_int(5);      // Exactly 5.0
/// let y = F26Dot6::from_float(5.5); // 5 + 32/64 = 5.5
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct F26Dot6(i32);

impl F26Dot6 {
    /// The magic number: 6 fractional bits = 64 precision units
    pub const FRAC_BITS: u32 = 6;

    /// Our fractional extractor: isolates the 6 lowest bits
    pub const FRAC_MASK: i32 = (1 << Self::FRAC_BITS) - 1;

    /// The fundamental unit: exactly 1.0 in our fixed world
    pub const ONE: F26Dot6 = F26Dot6(1 << Self::FRAC_BITS);

    /// Nothingness itself: perfectly zero
    pub const ZERO: F26Dot6 = F26Dot6(0);

    /// The midpoint: exactly 0.5, perfect for rounding
    pub const HALF: F26Dot6 = F26Dot6(1 << (Self::FRAC_BITS - 1));

    /// From whole number to fixed-point precision
    #[inline]
    pub const fn from_int(x: i32) -> Self {
        F26Dot6(x << Self::FRAC_BITS)
    }

    /// Transform浮动点 precision into our fixed world
    #[inline]
    pub fn from_float(x: f32) -> Self {
        F26Dot6((x * 64.0) as i32)
    }

    /// Shed the fractional baggage: pure integer result
    #[inline]
    pub const fn to_int(self) -> i32 {
        self.0 >> Self::FRAC_BITS
    }

    /// Mathematical justice: round to the nearest whole number
    #[inline]
    pub const fn to_int_round(self) -> i32 {
        (self.0 + Self::HALF.0) >> Self::FRAC_BITS
    }

    /// Extract the essence: the fractional detail that makes us precise
    #[inline]
    pub const fn frac(self) -> i32 {
        self.0 & Self::FRAC_MASK
    }

    /// Return to the floating world: when you need decimal representation
    #[inline]
    pub fn to_float(self) -> f32 {
        self.0 as f32 / 64.0
    }

    /// Precision multiplication: where fractions meet fractions
    #[inline]
    pub const fn mul(self, other: F26Dot6) -> F26Dot6 {
        F26Dot6(((self.0 as i64 * other.0 as i64) >> Self::FRAC_BITS) as i32)
    }

    /// Careful division: maintaining precision through the quotient
    #[inline]
    pub const fn div(self, other: F26Dot6) -> F26Dot6 {
        F26Dot6((((self.0 as i64) << Self::FRAC_BITS) / other.0 as i64) as i32)
    }

    /// Distance from zero: magnitude without direction
    #[inline]
    pub const fn abs(self) -> F26Dot6 {
        F26Dot6(self.0.abs())
    }

    /// Gravity's pull: always round down toward the earth
    #[inline]
    pub const fn floor(self) -> F26Dot6 {
        F26Dot6(self.0 & !(Self::FRAC_MASK))
    }

    /// Reach for the sky: always round up to greater heights
    #[inline]
    pub const fn ceil(self) -> F26Dot6 {
        if self.0 & Self::FRAC_MASK == 0 {
            self
        } else {
            F26Dot6((self.0 & !(Self::FRAC_MASK)) + Self::ONE.0)
        }
    }

    /// Peer under the hood: access the raw 32-bit representation
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// From raw bits to meaningful numbers: the inverse of peeking inside
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
        assert_eq!(F26Dot6::from_float(-3.25).to_int(), -4);
    }

    #[test]
    fn test_to_int_round() {
        assert_eq!(F26Dot6::from_float(5.25).to_int_round(), 5);
        assert_eq!(F26Dot6::from_float(5.5).to_int_round(), 6);
        assert_eq!(F26Dot6::from_float(5.75).to_int_round(), 6);
        assert_eq!(F26Dot6::from_float(-3.25).to_int_round(), -3);
        assert_eq!(F26Dot6::from_float(-3.5).to_int_round(), -3);
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
        for i in 0..64 {
            let f = F26Dot6::from_raw(i);
            assert_eq!(f.frac(), i);
        }
    }

    #[test]
    fn test_overflow_safety() {
        let large = F26Dot6::from_int(10000);
        let small = F26Dot6::from_int(2);
        let result = large.mul(small);
        assert_eq!(result.to_int(), 20000);
    }
}
