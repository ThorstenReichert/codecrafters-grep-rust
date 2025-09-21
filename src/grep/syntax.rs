use super::str::StringUtils;

#[derive(Debug, PartialEq)]
pub enum Syntax {
    /// Matches a single specified character.
    Literal { char: char },

    /// Matches a single digit. Equivalent to \[0-9\]
    Digit,

    /// Matches a single word character. Equivalent to \[a-zA-Z0-9_\].
    Word,

    /// Matches any one of the specified characters.
    CharacterClass { chars: Vec<char>, is_negated: bool },
}

pub fn parse_pattern(pattern: &str) -> Vec<Syntax> {
    let mut syntax: Vec<Syntax> = vec![];
    let mut remainder = pattern;

    while remainder.len() > 0 {
        let prev_len = remainder.len();

        if remainder.starts_with('[') {
            if let Some(end) = remainder.find(']') {
                let character_class = &pattern.slice(1..end);
                if character_class.starts_with('^') {
                    let negated_character_class = &character_class[1..];

                    syntax.push(Syntax::CharacterClass {
                        chars: negated_character_class.chars().collect(),
                        is_negated: true,
                    });
                    remainder = &remainder.slice(end + 1..);
                } else {
                    syntax.push(Syntax::CharacterClass {
                        chars: character_class.chars().collect(),
                        is_negated: false,
                    });
                    remainder = &remainder.slice(end + 1..);
                }
            } else {
                panic!(
                    "Incomplete character class '{}' (missing closing bracket)",
                    remainder
                );
            }
        } else if remainder.starts_with("\\d") {
            syntax.push(Syntax::Digit);
            remainder = &remainder.slice(2..);
        } else if remainder.starts_with("\\w") {
            syntax.push(Syntax::Word);
            remainder = &remainder.slice(2..);
        } else {
            syntax.push(Syntax::Literal {
                char: remainder.chars().next().unwrap(),
            });
            remainder = &remainder.slice(1..);
        }

        // Sanity check to ensure that progress is made.
        assert!(
            remainder.len() < prev_len,
            "Must consume at least one pattern char"
        )
    }

    syntax
}

#[cfg(test)]
mod tests {
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
}
