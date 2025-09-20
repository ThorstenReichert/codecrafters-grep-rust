pub mod patterns;

use patterns::is_digit;

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else if pattern == "\\d" {
        return input_line.contains(is_digit)
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_pattern_single_char() {
        assert!(match_pattern("abcdefg", "e"))
    }

    #[test]
    fn test_match_pattern_single_char_not_contained() {
        assert!(!match_pattern("abcdefg", "x"))
    }

    #[test]
    fn test_match_pattern_digit() {
        assert!(match_pattern("ab1def", "\\d"))
    }

    #[test]
    fn test_match_pattern_digit_no_digit() {
        assert!(!match_pattern("abcdefg", "\\d"))
    }
}