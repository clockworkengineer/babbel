//! Fast zero-allocation numeric formatting abstractions.

use crate::io::traits::IDestination;

/// Formats an integer directly into an `IDestination` without heap allocation using `itoa`.
#[inline]
pub fn format_integer<T: itoa::Integer>(val: T, dest: &mut dyn IDestination) {
    let mut buf = itoa::Buffer::new();
    dest.add_bytes(buf.format(val));
}

/// Formats a floating-point number directly into an `IDestination` without heap allocation using `dtoa`.
#[inline]
pub fn format_float<T: dtoa::Float>(val: T, dest: &mut dyn IDestination) {
    let mut buf = dtoa::Buffer::new();
    dest.add_bytes(buf.format(val));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::StringDestination;

    #[test]
    fn test_format_numbers() {
        let mut dest = StringDestination::new();
        format_integer(123456, &mut dest);
        dest.add_byte(b',');
        format_integer(-987, &mut dest);
        dest.add_byte(b',');
        format_float(3.14159, &mut dest);
        assert_eq!(dest.into_string(), "123456,-987,3.14159");
    }
}
