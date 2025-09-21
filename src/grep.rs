mod patterns;
mod str;
mod syntax;
mod tokens;

use str::StringUtils;
use syntax::Syntax;

fn is_match(char: char, pattern: &Syntax) -> bool {
    match pattern {
        Syntax::Literal { char: c } => *c == char,
        Syntax::Digit => patterns::is_digit(char),
        Syntax::Word => patterns::is_word(char),
        Syntax::CharacterClass {
            chars: cs,
            is_negated: true,
        } => !patterns::is_any_of(&cs, char),
        Syntax::CharacterClass {
            chars: cs,
            is_negated: false,
        } => patterns::is_any_of(&cs, char),
    }
}

fn match_here(text: &str, pattern: &[Syntax]) -> bool {
    let Some(syntax) = pattern.get(0) else {
        // The entire pattern matched, return success.
        return true;
    };

    let Some(c) = &text.chars().next() else {
        // No more text, but still pattern left to match, return non-success.
        return false;
    };

    if is_match(*c, syntax) {
        return match_here(&text.slice(1..), &pattern[1..]);
    }

    return false;
}

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = tokens::tokenize_pattern(pattern);
    let syntax = syntax::parse_pattern(&tokens);

    for start_index in 0..input_line.len() {
        if match_here(&input_line.slice(start_index..), &syntax) {
            return true;
        }
    }

    false
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

    #[test]
    fn test_match_pattern_combined_character_classes() {
        assert!(match_pattern("1 apple", "\\d apple"));
        assert!(!match_pattern("1 orange", "\\d apple"));

        assert!(match_pattern("100 apples", "\\d\\d\\d apple"));
        assert!(!match_pattern("1 apple", "\\d\\d\\d apple"));

        assert!(match_pattern("3 dogs", "\\d \\w\\w\\ws"));
        assert!(match_pattern("4 cats", "\\d \\w\\w\\ws"));
        assert!(!match_pattern("1 dog", "\\d \\w\\w\\ws"));
    }

    #[test]
    fn test_match_pattern_regression_tests() {
        assert!(!match_pattern("×-+=÷%", "\\w"));
        assert!(!match_pattern("sally has 12 apples", "\\d\\\\d\\\\d apples"));
    }
}
