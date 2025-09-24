use std::collections::VecDeque;

use crate::grep::tokens::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum Syntax {
    /// Matches a single specified character.
    Literal { char: char },

    /// Matches a single digit. Equivalent to \[0-9\]
    Digit,

    /// Matches a single word character. Equivalent to \[a-zA-Z0-9_\].
    Word,

    /// Matches any single character.
    Wildcard,

    /// Matches any one of the specified characters.
    CharacterClass { chars: Vec<char>, is_negated: bool },

    /// Matches the start of a line.
    StartOfLineAnchor,

    /// Matches the end of a line.
    EndOfLineAnchor,

    /// Matches the contained syntax one or more times.
    OneOrMore { syntax: Box<Syntax> },

    /// Matches the contained syntax zero or more times.
    ZeroOrOne { syntax: Box<Syntax> },

    /// Matches either of the contained syntax options.
    CaptureGroup { options: Vec<Vec<Syntax>>, id: u32 },

    /// Artificial syntax to finalize capture groups.
    CaptureGroupEnd { text: String, id: u32 },

    /// References an already matched capture group by id.
    BackReference { id: u32 },
}

fn into_character_class(tokens: &[Token], is_negated: bool) -> Syntax {
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

#[derive(PartialEq)]
enum BracketKind {
    Bracket,
    SquareBracket,
}

fn is_opening_bracket(token: &Token) -> Option<BracketKind> {
    match token {
        Token::OpenBracket => Some(BracketKind::Bracket),
        Token::OpenSquareBracket => Some(BracketKind::SquareBracket),
        _ => None,
    }
}

fn is_closing_bracket(token: &Token) -> Option<BracketKind> {
    match token {
        Token::CloseBracket => Some(BracketKind::Bracket),
        Token::CloseSquareBracket => Some(BracketKind::SquareBracket),
        _ => None,
    }
}

fn find_closing_bracket(pattern: &[Token]) -> Option<usize> {
    let first = pattern.get(0).expect("Pattern must not be empty");
    let Some(kind) = is_opening_bracket(first) else {
        panic!("First token must be an opening bracket");
    };

    let mut index = 1;
    let mut brackets = VecDeque::from([]);

    for token in pattern[1..].iter() {
        if let Some(open_kind) = is_opening_bracket(token) {
            brackets.push_back(open_kind);
        }

        if let Some(close_kind) = is_closing_bracket(token) {
            let last_open_bracket = brackets.pop_back();

            if let Some(open_kind) = last_open_bracket {
                if open_kind != close_kind {
                    // Open/closed bracket types do not match, fail search;
                    return None;
                }
            } else if close_kind == kind {
                return Some(index);
            } else {
                // Open/closed bracket types do not match, fail search;
                return None;
            }
        }

        index += 1;
    }

    return None;
}

fn parse_pattern_core(pattern: &[Token], capture_group_id: &mut u32) -> Vec<Syntax> {
    let mut syntax: Vec<Syntax> = vec![];
    let mut remainder = pattern;

    if remainder.starts_with(&[Token::Caret]) {
        syntax.push(Syntax::StartOfLineAnchor);
        remainder = &remainder[1..];
    }

    while remainder.len() > 0 {
        let prev_len = remainder.len();

        if remainder.starts_with(&[Token::OpenSquareBracket]) {
            let Some(end) = find_closing_bracket(remainder)
            else {
                panic!("Incomplete character class (missing closing bracket)");
            };

            let character_class = &remainder[1..end];
            if character_class.starts_with(&[Token::Caret]) {
                let negated_character_class = &character_class[1..];

                syntax.push(into_character_class(negated_character_class, true));
                remainder = &remainder[end + 1..];
            } else {
                syntax.push(into_character_class(character_class, false));
                remainder = &remainder[end + 1..];
            }
        } else if remainder.starts_with(&[Token::OpenBracket]) {
            let Some(end) = find_closing_bracket(remainder) else {
                panic!("Incomplete alternation (missing closing bracket)");
            };

            *capture_group_id += 1;
            let id = *capture_group_id;
            let options: Vec<Vec<Syntax>> = remainder[1..end]
                .split(|t| *t == Token::Bar)
                .map(|o| parse_pattern_core(o, capture_group_id))
                .collect();

            syntax.push(Syntax::CaptureGroup {
                options: options,
                id: id,
            });
            remainder = &remainder[end + 1..];
        } else if remainder.starts_with(&[Token::Backslash, Token::Backslash]) {
            syntax.push(Syntax::Literal { char: '\\' });
            remainder = &remainder[2..];
        } else if remainder.starts_with(&[Token::Backslash, Token::Literal('d')]) {
            syntax.push(Syntax::Digit);
            remainder = &remainder[2..];
        } else if remainder.starts_with(&[Token::Backslash, Token::Literal('w')]) {
            syntax.push(Syntax::Word);
            remainder = &remainder[2..];
        } else if remainder.starts_with(&[Token::Backslash]) {
            let Some(escapee) = remainder.get(1) else {
                panic!("Incomplete escape sequence");
            };

            if let Token::Literal(l) = escapee {
                if let Some(d) = char::to_digit(*l, 10) {
                    syntax.push(Syntax::BackReference { id: d });
                    remainder = &remainder[2..];
                } else {
                    panic!("Unrecognized escape sequence '\\{}'", l);
                }
            } else {
                panic!("Unrecognized token type following backslash");
            }
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
            syntax.push(Syntax::ZeroOrOne {
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

pub fn parse_pattern(pattern: &[Token]) -> Vec<Syntax> {
    let mut capture_group_id = 0;
    parse_pattern_core(pattern, &mut capture_group_id)
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
            Syntax::ZeroOrOne {
                syntax: Box::new(Syntax::Literal { char: 'a' }),
            },
        )
    }

    #[test]
    fn test_parse_pattern_wildcard() {
        assert_single(parse_pattern(&[Token::Dot]), Syntax::Wildcard);
    }

    #[test]
    fn test_parse_pattern_alternation() {
        assert_single(
            parse_pattern(&[
                Token::OpenBracket,
                Token::Literal('a'),
                Token::Backslash,
                Token::Literal('d'),
                Token::Bar,
                Token::Literal('b'),
                Token::CloseBracket,
            ]),
            Syntax::CaptureGroup {
                options: vec![
                    vec![Syntax::Literal { char: 'a' }, Syntax::Digit],
                    vec![Syntax::Literal { char: 'b' }],
                ],
                id: 1,
            },
        );
    }

    #[test]
    fn test_parse_pattern_capture_group_ids() {
        let items = parse_pattern(&[
            Token::OpenBracket,
            Token::Literal('a'),
            Token::CloseBracket,
            Token::OpenBracket,
            Token::Literal('b'),
            Token::CloseBracket,
        ]);

        assert_eq!(
            items.get(0).unwrap(),
            &Syntax::CaptureGroup {
                options: vec![vec![Syntax::Literal { char: 'a' }]],
                id: 1
            }
        );
        assert_eq!(
            items.get(1).unwrap(),
            &Syntax::CaptureGroup {
                options: vec![vec![Syntax::Literal { char: 'b' }]],
                id: 2
            }
        );
    }

    #[test]
    fn test_parse_pattern_backreference() {
        assert_single(
            parse_pattern(&[Token::Backslash, Token::Literal('1')]),
            Syntax::BackReference { id: 1 },
        )
    }
}
