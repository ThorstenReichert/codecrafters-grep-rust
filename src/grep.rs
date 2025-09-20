use crate::grep::patterns::is_any_of;

pub mod patterns;

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else if pattern == "\\d" {
        return input_line.contains(patterns::is_digit);
    } else if pattern == "\\w" {
        return input_line.contains(patterns::is_word);
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        let char_group: Vec<char> = pattern[1..pattern.len() - 1].chars().collect();

        if char_group.starts_with(&['^']) {
            let negative_char_group = &char_group[1..];
            return input_line.contains(|c| !is_any_of(&negative_char_group, c))
        } else {
            return input_line.contains(|c| is_any_of(&char_group, c));
        }
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

    #[test]
    fn test_match_pattern_word() {
        assert!(match_pattern("fool101", "\\w"))
    }

    #[test]
    fn test_match_pattern_word_no_word() {
        assert!(!match_pattern("$!?", "\\w"))
    }

    #[test]
    fn test_match_pattern_character_group() {
        assert!(match_pattern("apple", "[abc]"));
        assert!(match_pattern("apple", "[cba]"));
    }

    #[test]
    fn test_match_pattern_character_group_no_match() {
        assert!(!match_pattern("apple", "[]"));
        assert!(!match_pattern("apple", "[b]"));
        assert!(!match_pattern("apple", "[_xy]"));
    }

    #[test]
    fn test_match_pattern_negative_character_group() {
        assert!(match_pattern("cat", "[^abc]"))
    }

    #[test]
    fn test_match_pattern_negative_character_group_match() {
        assert!(!match_pattern("cab", "[^abc]"));
    }
}
