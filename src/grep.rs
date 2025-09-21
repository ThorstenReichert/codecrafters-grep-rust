mod patterns;
mod str;
mod syntax;
mod tokens;

use std::ops::Deref;

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

        Syntax::StartOfLineAnchor => panic!(
            "Only one-character matching syntax expected here, but found start of line anchor"
        ),

        Syntax::EndOfLineAnchor => {
            panic!("Only one-character matching syntax expected here, but found end of line anchor")
        }

        Syntax::OneOrMore { syntax: _ } => panic!(
            "Only one-character matching syntax expected here, but found one or more quantifier"
        ),

        Syntax::ZeroOrMore { syntax: _ } => panic!(
            "Only one-character matching syntax expected here, but found zero or more quantifier"
        ),
    }
}

fn match_at_least(text: &str, syntax: &Syntax, pattern_remainder: &[Syntax], count: usize) -> bool {
    if let Syntax::OneOrMore { syntax: _ } = syntax {
        panic!("Nested quantifiers are not supported");
    }

    if text.len() < count || text.chars().take(count).any(|c| !is_match(c, &syntax)) {
        return false;
    }

    let mut text_remainder = &text[count..];
    loop {
        if match_here(text_remainder, pattern_remainder) {
            return true;
        }

        let Some(c) = text_remainder.chars().next() else {
            return false;
        };

        if !is_match(c, &syntax) {
            return false;
        }

        text_remainder = &text_remainder[1..];
    }
}

fn match_here(text: &str, pattern: &[Syntax]) -> bool {
    let Some(syntax) = pattern.get(0) else {
        // The entire pattern matched, return success.
        return true;
    };

    if let Syntax::OneOrMore { syntax: s } = syntax {
        return match_at_least(text, &s.deref(), &pattern[1..], 1);
    }

    if let Syntax::ZeroOrMore { syntax: s } = syntax {
        return match_at_least(text, &s.deref(), &pattern[1..], 0);
    }

    if let Syntax::EndOfLineAnchor = syntax {
        return pattern.len() == 1 && text.len() == 0;
    }

    if let Some(c) = text.chars().next() {
        return is_match(c, syntax) && match_here(&text.slice(1..), &pattern[1..]);
    }

    return false;
}

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = tokens::tokenize_pattern(pattern);
    let syntax = syntax::parse_pattern(&tokens);

    if let Some(Syntax::StartOfLineAnchor) = syntax.get(0) {
        return match_here(input_line, &syntax[1..]);
    }

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
    fn test_match_pattern_start_of_line_anchor() {
        assert!(match_pattern("log", "^log"));
        assert!(!match_pattern("slog", "^log"));
    }

    #[test]
    fn test_match_pattern_end_of_line_anchor() {
        assert!(match_pattern("dog", "dog$"));
        assert!(!match_pattern("dogs", "dog$"));
    }

    #[test]
    fn test_match_pattern_empty_anchors() {
        assert!(match_pattern("", "^$"));
        assert!(!match_pattern("x", "^$"));
    }

    #[test]
    fn test_match_pattern_one_or_more_quantifier() {
        assert!(match_pattern("caats", "ca+ts"));
        assert!(match_pattern("caaaaa", "ca+"));
        assert!(!match_pattern("cts", "ca+ts"));
    }

    #[test]
    fn test_match_pattern_zero_or_more_quantifier() {
        assert!(match_pattern("dogs", "dogs?"));
        assert!(match_pattern("dog", "dogs?"));
        assert!(!match_pattern("cat", "dogs?"));
    }

    #[test]
    fn test_match_pattern_regression_tests() {
        assert!(!match_pattern("×-+=÷%", "\\w"));
        assert!(!match_pattern(
            "sally has 12 apples",
            "\\d\\\\d\\\\d apples"
        ));
    }
}
