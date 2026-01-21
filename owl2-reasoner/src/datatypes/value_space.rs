//! Value space utilities for numeric datatypes
//!
//! This module provides utilities for working with discrete value spaces
//! of numeric datatypes, particularly IEEE 754 floating-point numbers.

/// Get the next representable float value after the given value
///
/// This function returns the smallest float that is strictly greater than
/// the input value. It handles special cases including:
/// - Zero (returns smallest positive float)
/// - Negative numbers (returns value closer to zero)
/// - Positive numbers (returns next larger value)
/// - Infinity (returns infinity)
/// - NaN (returns NaN)
///
/// # Examples
///
/// ```
/// use owl2_reasoner::datatypes::value_space::next_float;
///
/// let result = next_float(0.0);
/// assert!(result > 0.0);
/// assert!(result < f32::MIN_POSITIVE);
/// assert!(next_float(1.0) > 1.0);
/// assert_eq!(next_float(f32::INFINITY), f32::INFINITY);
/// ```
pub fn next_float(value: f32) -> f32 {
    // Handle special cases
    if value.is_nan() {
        return f32::NAN;
    }

    if value == f32::INFINITY {
        return f32::INFINITY;
    }

    if value == f32::NEG_INFINITY {
        return -f32::MAX;
    }

    // Convert to bits for manipulation
    let bits = value.to_bits();

    // Handle zero specially
    if value == 0.0 {
        // Return smallest positive float (subnormal minimum)
        return f32::from_bits(0x0000_0001);
    }

    // For positive numbers, increment bit pattern
    if value > 0.0 {
        return f32::from_bits(bits + 1);
    }

    // For negative numbers, decrement bit pattern (moves toward zero)
    f32::from_bits(bits - 1)
}

/// Get the previous representable float value before the given value
///
/// This is the inverse of `next_float()`.
pub fn prev_float(value: f32) -> f32 {
    // Handle special cases
    if value.is_nan() {
        return f32::NAN;
    }

    if value == f32::NEG_INFINITY {
        return f32::NEG_INFINITY;
    }

    if value == f32::INFINITY {
        return f32::MAX;
    }

    // Convert to bits for manipulation
    let bits = value.to_bits();

    // Handle zero specially
    if value == 0.0 {
        // Return largest negative float (subnormal minimum)
        return f32::from_bits(0x8000_0001);
    }

    // For positive numbers, decrement bit pattern
    if value > 0.0 {
        return f32::from_bits(bits - 1);
    }

    // For negative numbers, increment bit pattern (moves away from zero)
    f32::from_bits(bits + 1)
}

/// Check if a float range is empty (contains no representable values)
///
/// A range is considered empty if there are no float values strictly between
/// the min and max bounds, taking into account the exclusive/inclusive nature
/// of the bounds.
///
/// # Arguments
///
/// * `min` - The minimum bound value
/// * `min_inclusive` - Whether the minimum bound is inclusive
/// * `max` - The maximum bound value
/// * `max_inclusive` - Whether the maximum bound is inclusive
///
/// # Returns
///
/// `true` if the range contains no representable float values, `false` otherwise
///
/// # Examples
///
/// ```
/// use owl2_reasoner::datatypes::is_float_range_empty;
///
/// // Range (0.0, MIN_POSITIVE) is empty - no floats between them
/// assert!(is_float_range_empty(0.0, false, f32::from_bits(0x00000001), false));
///
/// // Range [0.0, 0.0] contains one value
/// assert!(!is_float_range_empty(0.0, true, 0.0, true));
///
/// // Range (0.0, 0.0) is empty
/// assert!(is_float_range_empty(0.0, false, 0.0, false));
/// ```
pub fn is_float_range_empty(min: f32, min_inclusive: bool, max: f32, max_inclusive: bool) -> bool {
    // Handle NaN
    if min.is_nan() || max.is_nan() {
        return true;
    }

    // If min > max, range is definitely empty
    if min > max {
        return true;
    }

    // If min == max
    if min == max {
        // Range is empty unless both bounds are inclusive
        return !(min_inclusive && max_inclusive);
    }

    // Determine the effective minimum value in the range
    let effective_min = if min_inclusive { min } else { next_float(min) };

    // Determine the effective maximum value in the range
    let effective_max = if max_inclusive { max } else { prev_float(max) };

    // Range is empty if effective_min > effective_max
    effective_min > effective_max
}

/// Check if a float range with exclusive bounds is empty
///
/// This is a convenience function for the common case of exclusive bounds.
///
/// # Examples
///
/// ```
/// use owl2_reasoner::datatypes::is_float_range_empty_exclusive;
///
/// // The test case: (0.0, 1.401298464324817e-45)
/// // 1.401298464324817e-45 is the smallest positive float
/// assert!(is_float_range_empty_exclusive(0.0, f32::from_bits(0x00000001)));
/// ```
pub fn is_float_range_empty_exclusive(min: f32, max: f32) -> bool {
    is_float_range_empty(min, false, max, false)
}
