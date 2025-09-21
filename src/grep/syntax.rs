use crate::grep::tokens::Token;

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

    /// Matches the start of a line.
    StartOfLineAnchor,

    /// Matches the end of a line.
    EndOfLineAnchor,

    /// Matches the contained syntax one or more times.
    OneOrMore { syntax: Box<Syntax> },

    /// Matches the contained syntax zero or more times.
    ZeroOrMore { syntax: Box<Syntax> },

    /// Matches any single character.
    Wildcard,
}

pub fn into_character_class(tokens: &[Token], is_negated: bool) -> Syntax {
    Syntax::CharacterClass {
        chars: tokens
            .iter()
            .map(|t| match t {
                Token::Literal(c) => *c,
                other => panic!("Invalid token '{}' in character class", other),
            })
            .collect(),
        is_negated: is_negated,
    }
}

pub fn parse_pattern(pattern: &[Token]) -> Vec<Syntax> {
    let mut syntax: Vec<Syntax> = vec![];
    let mut remainder = pattern;

    if remainder.starts_with(&[Token::Caret]) {
        syntax.push(Syntax::StartOfLineAnchor);
        remainder = &remainder[1..];
    }

    while remainder.len() > 0 {
        let prev_len = remainder.len();

        if remainder.starts_with(&[Token::OpenSquareBracket]) {
            if let Some(end) = remainder
                .iter()
                .position(|t| *t == Token::CloseSquareBracket)
            {
                let character_class = &pattern[1..end];
                if character_class.starts_with(&[Token::Caret]) {
                    let negated_character_class = &character_class[1..];

                    syntax.push(into_character_class(negated_character_class, true));
                    remainder = &remainder[end + 1..];
                } else {
                    syntax.push(into_character_class(character_class, false));
                    remainder = &remainder[end + 1..];
                }
            } else {
                panic!("Incomplete character class (missing closing bracket)");
            }
        } else if remainder.starts_with(&[Token::Backslash, Token::Backslash]) {
            syntax.push(Syntax::Literal { char: '\\' });
            remainder = &remainder[2..]
        } else if remainder.starts_with(&[Token::Backslash, Token::Literal('d')]) {
            syntax.push(Syntax::Digit);
            remainder = &remainder[2..];
        } else if remainder.starts_with(&[Token::Backslash, Token::Literal('w')]) {
            syntax.push(Syntax::Word);
            remainder = &remainder[2..];
        } else if remainder.starts_with(&[Token::Dot]) {
            syntax.push(Syntax::Wildcard);
            remainder = &remainder[1..];
        } else if remainder.starts_with(&[Token::Dollar]) {
            syntax.push(Syntax::EndOfLineAnchor);
            remainder = &remainder[1..];
        } else if remainder.starts_with(&[Token::Plus]) {
            let contained_syntax = syntax
                .pop()
                .expect("The one or more modifier can only appear after another token");
            syntax.push(Syntax::OneOrMore {
                syntax: Box::from(contained_syntax),
            });
            remainder = &remainder[1..];
        } else if remainder.starts_with(&[Token::QuestionMark]) {
            let contained_syntax = syntax
                .pop()
                .expect("The zero or more modifier can only appear after another token");
            syntax.push(Syntax::ZeroOrMore {
                syntax: Box::from(contained_syntax),
            });
            remainder = &remainder[1..];
        } else if let Some(Token::Literal(c)) = remainder.get(0) {
            syntax.push(Syntax::Literal { char: *c });
            remainder = &remainder[1..];
        } else {
            panic!("Malformed pattern, cannot parse token");
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
        assert_single(
            parse_pattern(&[Token::Literal('a')]),
            Syntax::Literal { char: 'a' },
        );
    }

    #[test]
    fn test_parse_pattern_digit() {
        assert_single(
            parse_pattern(&[Token::Backslash, Token::Literal('d')]),
            Syntax::Digit,
        );
    }

    #[test]
    fn test_parse_pattern_word() {
        assert_single(
            parse_pattern(&[Token::Backslash, Token::Literal('w')]),
            Syntax::Word,
        );
    }

    #[test]
    fn test_parse_pattern_character_class() {
        assert_single(
            parse_pattern(&[
                Token::OpenSquareBracket,
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::CloseSquareBracket,
            ]),
            Syntax::CharacterClass {
                chars: vec!['a', 'b', 'c'],
                is_negated: false,
            },
        )
    }

    #[test]
    fn test_parse_pattern_negated_character_class() {
        assert_single(
            parse_pattern(&[
                Token::OpenSquareBracket,
                Token::Caret,
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::CloseSquareBracket,
            ]),
            Syntax::CharacterClass {
                chars: vec!['a', 'b', 'c'],
                is_negated: true,
            },
        )
    }

    #[test]
    fn test_parse_pattern_start_of_line_anchor() {
        assert_single(parse_pattern(&[Token::Caret]), Syntax::StartOfLineAnchor);
    }

    #[test]
    fn test_parse_pattern_end_of_line_anchor() {
        assert_single(parse_pattern(&[Token::Dollar]), Syntax::EndOfLineAnchor);
    }

    #[test]
    fn test_parse_pattern_one_or_more_modifier() {
        assert_single(
            parse_pattern(&[Token::Literal('a'), Token::Plus]),
            Syntax::OneOrMore {
                syntax: Box::new(Syntax::Literal { char: 'a' }),
            },
        )
    }

    #[test]
    fn test_parse_pattern_zero_or_more_modifier() {
        assert_single(
            parse_pattern(&[Token::Literal('a'), Token::QuestionMark]),
            Syntax::ZeroOrMore {
                syntax: Box::new(Syntax::Literal { char: 'a' }),
            },
        )
    }

    #[test]
    fn test_parse_pattern_wildcard() {
        assert_single(parse_pattern(&[Token::Dot]), Syntax::Wildcard);
    }
}
