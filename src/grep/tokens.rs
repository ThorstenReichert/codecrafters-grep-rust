use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(char),
    Backslash,
    OpenBracket,
    CloseBracket,
    OpenSquareBracket,
    CloseSquareBracket,
    Caret,
    Dollar,
    Plus,
    QuestionMark,
    Dot,
    Bar
}

pub fn tokenize_pattern(pattern: &str) -> Vec<Token> {
    pattern
        .chars()
        .map(|c| match c {
            '\\' => Token::Backslash,
            '(' => Token::OpenBracket,
            ')' => Token::CloseBracket,
            '[' => Token::OpenSquareBracket,
            ']' => Token::CloseSquareBracket,
            '^' => Token::Caret,
            '$' => Token::Dollar,
            '+' => Token::Plus,
            '?' => Token::QuestionMark,
            '.' => Token::Dot,
            '|' => Token::Bar,
            other => Token::Literal(other),
        })
        .collect()
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Backslash => write!(f, "\\"),
            Token::OpenBracket => write!(f, "("),
            Token::CloseBracket => write!(f, ")"),
            Token::OpenSquareBracket => write!(f, "["),
            Token::CloseSquareBracket => write!(f, "]"),
            Token::Caret => write!(f, "^"),
            Token::Dollar => write!(f, "$"),
            Token::Plus => write!(f, "+"),
            Token::QuestionMark => write!(f, "?"),
            Token::Dot => write!(f, "."),
            Token::Bar => write!(f, "|"),
            Token::Literal(c) => write!(f, "{}", c)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_pattern_backslash() {
        assert_eq!(tokenize_pattern("\\"), [Token::Backslash])
    }

    #[test]
    fn test_tokenize_pattern_open_bracket() {
        assert_eq!(tokenize_pattern("("), [Token::OpenBracket])
    }

    #[test]
    fn test_tokenize_pattern_closingbracket() {
        assert_eq!(tokenize_pattern(")"), [Token::CloseBracket])
    }

    #[test]
    fn test_tokenize_pattern_open_square_bracket() {
        assert_eq!(tokenize_pattern("["), [Token::OpenSquareBracket])
    }

    #[test]
    fn test_tokenize_pattern_closing_square_bracket() {
        assert_eq!(tokenize_pattern("]"), [Token::CloseSquareBracket])
    }

    #[test]
    fn test_tokenize_pattern_caret() {
        assert_eq!(tokenize_pattern("^"), [Token::Caret])
    }

    #[test]
    fn test_tokenize_pattern_dollar() {
        assert_eq!(tokenize_pattern("$"), [Token::Dollar]);
    }

    #[test]
    fn test_tokenize_pattern_plus() {
        assert_eq!(tokenize_pattern("+"), [Token::Plus]);
    }

    #[test]
    fn test_tokenize_pattern_question_mark() {
        assert_eq!(tokenize_pattern("?"), [Token::QuestionMark]);
    }

    #[test]
    fn test_tokenize_pattern_dot() {
        assert_eq!(tokenize_pattern("."), [Token::Dot]);
    }

    #[test]
    fn test_tokenize_pattern_bar() {
        assert_eq!(tokenize_pattern("|"), [Token::Bar]);
    }

    #[test]
    fn test_tokenize_pattern_complex_pattern() {
        assert_eq!(
            tokenize_pattern("[^abc]\\d\\d"),
            [
                Token::OpenSquareBracket,
                Token::Caret,
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::CloseSquareBracket,
                Token::Backslash,
                Token::Literal('d'),
                Token::Backslash,
                Token::Literal('d')
            ]
        )
    }
}
