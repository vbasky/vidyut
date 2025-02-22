//! Utility functions for checking Sanskrit sounds.

use std::sync::OnceLock;

/// A set of Sanskrit sounds.
///
/// This implementation is copied directly from `vidyut_prakriya::sounds`. For details, see the
/// comments there.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Set([u8; 256]);

impl Set {
    /// Creates an empty set.
    pub fn new() -> Self {
        Set([0; 256])
    }

    /// Creates a set whose members are the characters in `string`.
    pub fn from(string: impl AsRef<str>) -> Self {
        let mut res = Self::new();
        for c in string.as_ref().chars() {
            res.0[c as usize] = 1;
        }
        res
    }

    /// Returns whether the set contains the given sound.
    pub fn contains(&self, c: char) -> bool {
        self.0[c as usize] == 1
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns whether the given character is a Sanskrit sound or *avagraha*.
///
/// We use this function to find boundaries between Sanskrit words. Non-Sanskrit sounds include:
/// - spaces
/// - other punctuation characters (|, ||, numbers)
/// - characters or symbols from non-SLP1 encodings
pub fn is_sanskrit(c: char) -> bool {
    static CHARS: OnceLock<Set> = OnceLock::new();
    CHARS.get_or_init(|| Set::from("aAiIuUfFxXeEoOMHkKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzshL'"));
    CHARS.get().unwrap().contains(c)
}

/// Returns whether the given sound is a vowel.
///
/// `ac` is the Paninian name for the Sanskrit vowels.
#[allow(dead_code)]
pub fn is_ac(c: char) -> bool {
    static AC: OnceLock<Set> = OnceLock::new();
    AC.get_or_init(|| Set::from("aAiIuUfFxXeEoO"));
    AC.get().unwrap().contains(c)
}

/// Returns whether the given sound is voiced.
#[allow(dead_code)]
pub fn is_ghosha(c: char) -> bool {
    static GHOSHA: OnceLock<Set> = OnceLock::new();
    GHOSHA.get_or_init(|| Set::from("aAiIuUfFxXeEoOgGNjJYqQRdDnbBmyrlvh"));
    GHOSHA.get().unwrap().contains(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sanskrit() {
        for c in "aAiIuUfFxXeEoOMHkKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzshL'".chars() {
            assert!(is_sanskrit(c));
        }
        for c in "0123456789,.![]|".chars() {
            assert!(!is_sanskrit(c));
        }
    }

    #[test]
    fn test_is_ac() {
        for c in "aAiIuUfFxXeEoO".chars() {
            assert!(is_ac(c));
        }
        for c in "kKgGnSzsh0123456789 '+".chars() {
            assert!(!is_ac(c));
        }
    }

    #[test]
    fn test_is_ghosha() {
        for c in "aAiIuUfFxXeEoOgGnjJYqQRdDnbBmyrlvh".chars() {
            assert!(is_ghosha(c));
        }
        for c in "kKcCwWtTpPSzs".chars() {
            assert!(!is_ghosha(c));
        }
    }
}
