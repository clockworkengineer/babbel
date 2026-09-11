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

/// Represents different numeric types that can be stored in AST nodes across format representations.
#[derive(Clone, Debug, PartialEq)]
pub enum Numeric {
    Integer(i64),
    Float(f64),
    UInteger(u64),
    Byte(u8),
    Int32(i32),
    UInt32(u32),
    Int16(i16),
    UInt16(u16),
    Int8(i8),
    UInt8(u8),
}

impl From<i64> for Numeric {
    #[inline]
    fn from(value: i64) -> Self {
        Numeric::Integer(value)
    }
}

impl From<f64> for Numeric {
    #[inline]
    fn from(value: f64) -> Self {
        Numeric::Float(value)
    }
}

impl From<u64> for Numeric {
    #[inline]
    fn from(value: u64) -> Self {
        Numeric::UInteger(value)
    }
}

impl From<u8> for Numeric {
    #[inline]
    fn from(value: u8) -> Self {
        Numeric::Byte(value)
    }
}

impl From<i32> for Numeric {
    #[inline]
    fn from(value: i32) -> Self {
        Numeric::Int32(value)
    }
}

impl From<u32> for Numeric {
    #[inline]
    fn from(value: u32) -> Self {
        Numeric::UInt32(value)
    }
}

impl From<i16> for Numeric {
    #[inline]
    fn from(value: i16) -> Self {
        Numeric::Int16(value)
    }
}

impl From<u16> for Numeric {
    #[inline]
    fn from(value: u16) -> Self {
        Numeric::UInt16(value)
    }
}

impl From<i8> for Numeric {
    #[inline]
    fn from(value: i8) -> Self {
        Numeric::Int8(value)
    }
}

impl core::fmt::Display for Numeric {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Numeric::Integer(n) => write!(f, "{}", n),
            Numeric::Float(n) => write!(f, "{}", n),
            Numeric::UInteger(n) => write!(f, "{}", n),
            Numeric::Byte(n) => write!(f, "{}", n),
            Numeric::Int32(n) => write!(f, "{}", n),
            Numeric::UInt32(n) => write!(f, "{}", n),
            Numeric::Int16(n) => write!(f, "{}", n),
            Numeric::UInt16(n) => write!(f, "{}", n),
            Numeric::Int8(n) => write!(f, "{}", n),
            Numeric::UInt8(n) => write!(f, "{}", n),
        }
    }
}

impl Numeric {
    /// Lossy string conversion for all numeric variants.
    #[inline]
    pub fn to_string_lossy(&self) -> alloc::string::String {
        match self {
            Numeric::Integer(i) => alloc::string::ToString::to_string(i),
            Numeric::Float(f) => alloc::string::ToString::to_string(f),
            Numeric::UInteger(u) => alloc::string::ToString::to_string(u),
            Numeric::Byte(b) => alloc::string::ToString::to_string(b),
            Numeric::Int32(i) => alloc::string::ToString::to_string(i),
            Numeric::UInt32(u) => alloc::string::ToString::to_string(u),
            Numeric::Int16(i) => alloc::string::ToString::to_string(i),
            Numeric::UInt16(u) => alloc::string::ToString::to_string(u),
            Numeric::Int8(i) => alloc::string::ToString::to_string(i),
            Numeric::UInt8(u) => alloc::string::ToString::to_string(u),
        }
    }

    /// Convert to i32, recommended for embedded systems
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Numeric::Integer(v) => i32::try_from(*v).ok(),
            Numeric::Float(v) => {
                if v.is_finite() && *v >= i32::MIN as f64 && *v <= i32::MAX as f64 {
                    Some(*v as i32)
                } else {
                    None
                }
            }
            Numeric::UInteger(v) => i32::try_from(*v).ok(),
            Numeric::Byte(v) => Some(*v as i32),
            Numeric::Int32(v) => Some(*v),
            Numeric::UInt32(v) => i32::try_from(*v).ok(),
            Numeric::Int16(v) => Some(*v as i32),
            Numeric::UInt16(v) => Some(*v as i32),
            Numeric::Int8(v) => Some(*v as i32),
            Numeric::UInt8(v) => Some(*v as i32),
        }
    }

    /// Convert to f32, recommended for embedded systems
    pub fn to_f32(&self) -> f32 {
        match self {
            Numeric::Integer(v) => *v as f32,
            Numeric::Float(v) => *v as f32,
            Numeric::UInteger(v) => *v as f32,
            Numeric::Byte(v) => *v as f32,
            Numeric::Int32(v) => *v as f32,
            Numeric::UInt32(v) => *v as f32,
            Numeric::Int16(v) => *v as f32,
            Numeric::UInt16(v) => *v as f32,
            Numeric::Int8(v) => *v as f32,
            Numeric::UInt8(v) => *v as f32,
        }
    }

    /// Check if this numeric value fits in i32 range
    #[inline]
    pub fn fits_in_i32(&self) -> bool {
        self.to_i32().is_some()
    }

    /// Memory size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            Numeric::Integer(_) | Numeric::Float(_) | Numeric::UInteger(_) => 8,
            Numeric::Int32(_) | Numeric::UInt32(_) => 4,
            Numeric::Int16(_) | Numeric::UInt16(_) => 2,
            Numeric::Byte(_) | Numeric::Int8(_) | Numeric::UInt8(_) => 1,
        }
    }

    /// Formats this numeric value directly into an `IDestination` without heap allocation.
    pub fn format_to(&self, dest: &mut dyn IDestination) {
        match self {
            Numeric::Integer(n) => format_integer(*n, dest),
            Numeric::Float(n) => format_float(*n, dest),
            Numeric::UInteger(n) => format_integer(*n, dest),
            Numeric::Byte(n) => format_integer(*n, dest),
            Numeric::Int32(n) => format_integer(*n, dest),
            Numeric::UInt32(n) => format_integer(*n, dest),
            Numeric::Int16(n) => format_integer(*n, dest),
            Numeric::UInt16(n) => format_integer(*n, dest),
            Numeric::Int8(n) => format_integer(*n, dest),
            Numeric::UInt8(n) => format_integer(*n, dest),
        }
    }
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
