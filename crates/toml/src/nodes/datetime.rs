//! RFC 3339 Date and Time representations for TOML v1.0.0.

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// The specific variant of TOML date/time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DatetimeKind {
    /// Full date and time with UTC offset (e.g. `1979-05-27T07:32:00Z`).
    OffsetDateTime,
    /// Date and time without timezone offset (e.g. `1979-05-27T07:32:00`).
    LocalDateTime,
    /// Date only (e.g. `1979-05-27`).
    LocalDate,
    /// Time only (e.g. `07:32:00.999999`).
    LocalTime,
}

/// High-fidelity representation of a TOML RFC 3339 date/time.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TomlDatetime {
    pub kind: DatetimeKind,
    pub raw: String,
}

impl TomlDatetime {
    /// Attempts to parse an RFC 3339 date/time string (supporting TOML v1.0.0 and v1.1.0).
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Check for Local Time:
        //    TOML 1.1.0: HH:MM (len == 5)
        //    TOML 1.0.0: HH:MM:SS[.fraction] (len >= 8)
        if (trimmed.len() == 5 || trimmed.len() >= 8)
            && trimmed.len() <= 32
            && trimmed.as_bytes().get(2) == Some(&b':')
            && trimmed.as_bytes()[0].is_ascii_digit()
            && trimmed.as_bytes()[1].is_ascii_digit()
            && trimmed.as_bytes()[3].is_ascii_digit()
            && trimmed.as_bytes()[4].is_ascii_digit()
        {
            if trimmed.len() == 5 {
                // HH:MM (TOML 1.1.0 optional seconds)
                return Some(Self {
                    kind: DatetimeKind::LocalTime,
                    raw: trimmed.to_string(),
                });
            } else if trimmed.as_bytes().get(5) == Some(&b':')
                && trimmed.as_bytes()[6].is_ascii_digit()
                && trimmed.as_bytes()[7].is_ascii_digit()
            {
                // HH:MM:SS[.fraction]
                if trimmed.len() == 8
                    || (trimmed.len() > 8
                        && trimmed.as_bytes()[8] == b'.'
                        && trimmed[9..].bytes().all(|b| b.is_ascii_digit()))
                {
                    return Some(Self {
                        kind: DatetimeKind::LocalTime,
                        raw: trimmed.to_string(),
                    });
                }
            }
        }

        // 2. Check for Date: YYYY-MM-DD
        if trimmed.len() >= 10
            && trimmed.as_bytes()[4] == b'-'
            && trimmed.as_bytes()[7] == b'-'
            && trimmed.as_bytes()[0..4].iter().all(u8::is_ascii_digit)
            && trimmed.as_bytes()[5..7].iter().all(u8::is_ascii_digit)
            && trimmed.as_bytes()[8..10].iter().all(u8::is_ascii_digit)
        {
            if trimmed.len() == 10 {
                return Some(Self {
                    kind: DatetimeKind::LocalDate,
                    raw: trimmed.to_string(),
                });
            }

            // Has time component: separator can be 'T', 't', or ' '
            let sep = trimmed.as_bytes()[10];
            if (sep == b'T' || sep == b't' || sep == b' ') && trimmed.len() >= 16 {
                // Must have HH:MM at [11..16]
                if trimmed.as_bytes()[11].is_ascii_digit()
                    && trimmed.as_bytes()[12].is_ascii_digit()
                    && trimmed.as_bytes()[13] == b':'
                    && trimmed.as_bytes()[14].is_ascii_digit()
                    && trimmed.as_bytes()[15].is_ascii_digit()
                {
                    // Check if has offset (Z, z, +, - after index 10)
                    let time_part = &trimmed[11..];
                    let has_offset = time_part.ends_with('Z')
                        || time_part.ends_with('z')
                        || time_part.contains('+')
                        || time_part.contains('-');

                    let kind = if has_offset {
                        DatetimeKind::OffsetDateTime
                    } else {
                        DatetimeKind::LocalDateTime
                    };

                    return Some(Self {
                        kind,
                        raw: trimmed.to_string(),
                    });
                }
            }
        }

        None
    }

    /// Access the underlying raw RFC 3339 string.
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl core::fmt::Display for TomlDatetime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}
