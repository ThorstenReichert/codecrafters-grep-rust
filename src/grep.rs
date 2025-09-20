pub mod patterns;
pub mod syntax;

use syntax::Syntax;

fn parse_pattern(pattern: &str) -> Vec<syntax::Syntax> {
    let mut syntax: Vec<syntax::Syntax> = vec![];
    let mut remainder = pattern;

    while remainder.len() > 0 {
        let prev_len = remainder.len();

        if remainder.starts_with('[') {
            if let Some(end) = remainder.find(']') {
                let character_class = &pattern[1..end];
                if character_class.starts_with('^') {
                    let negated_character_class = &character_class[1..];

                    syntax.push(Syntax::CharacterClass {
                        chars: negated_character_class.chars().collect(),
                        is_negated: true,
                    });
                    remainder = &remainder[end + 1..];
                } else {
                    syntax.push(Syntax::CharacterClass {
                        chars: character_class.chars().collect(),
                        is_negated: false,
                    });
                    remainder = &remainder[end + 1..];
                }
            } else {
                panic!(
                    "Incomplete character class '{}' (missing closing bracket)",
                    remainder
                );
            }
        } else if remainder.starts_with("\\d") {
            syntax.push(Syntax::Digit);
            remainder = &remainder[2..];
        } else if remainder.starts_with("\\w") {
            syntax.push(Syntax::Word);
            remainder = &remainder[2..];
        } else {
            syntax.push(Syntax::Literal {
                char: remainder.chars().next().unwrap(),
            });
            remainder = &remainder[1..];
        }

        // Sanity check to ensure that progress is made.
        assert!(
            remainder.len() < prev_len,
            "Must consume at least one pattern char"
        )
    }

    syntax
}

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let syntax = parse_pattern(pattern);

    // Supports only single syntax item for now.
    assert_eq!(syntax.len(), 1);

    match &syntax[0] {
        Syntax::Literal { char } => input_line.contains(*char),
        Syntax::Digit => input_line.contains(patterns::is_digit),
        Syntax::Word => input_line.contains(patterns::is_word),
        Syntax::CharacterClass {
            chars,
            is_negated: true,
        } => input_line.contains(|c| !patterns::is_any_of(&chars, c)),
        Syntax::CharacterClass {
            chars,
            is_negated: false,
        } => input_line.contains(|c| patterns::is_any_of(&chars, c)),
    }
}

#[cfg(test)]
mod tests {
    use super::syntax::Syntax;
    use super::*;

    fn assert_single<T: std::fmt::Debug + PartialEq>(items: Vec<T>, expected: T) {
        assert_eq!(
            1,
            items.len(),
            "Collection must contain exactly one element"
        );
        assert_eq!(expected, items[0]);
    }

    #[test]
    fn test_parse_pattern_literal() {
        assert_single(parse_pattern("a"), Syntax::Literal { char: 'a' });
    }

    #[test]
    fn test_parse_pattern_digit() {
        assert_single(parse_pattern("\\d"), Syntax::Digit);
    }

    #[test]
    fn test_parse_pattern_word() {
        assert_single(parse_pattern("\\w"), Syntax::Word);
    }

    #[test]
    fn test_parse_pattern_character_class() {
        assert_single(
            parse_pattern("[abc]"),
            Syntax::CharacterClass {
                chars: vec!['a', 'b', 'c'],
                is_negated: false,
            },
        )
    }

    #[test]
    fn test_parse_pattern_negated_character_class() {
        assert_single(
            parse_pattern("[^abc]"),
            Syntax::CharacterClass {
                chars: vec!['a', 'b', 'c'],
                is_negated: true,
            },
        )
    }

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
