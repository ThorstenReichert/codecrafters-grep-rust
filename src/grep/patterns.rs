pub fn is_digit(char: char) -> bool {
    return '0' <= char && char <= '9';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_digit() {
        assert!(is_digit('0'));
        assert!(is_digit('1'));
        assert!(is_digit('2'));
        assert!(is_digit('3'));
        assert!(is_digit('4'));
        assert!(is_digit('5'));
        assert!(is_digit('6'));
        assert!(is_digit('7'));
        assert!(is_digit('8'));
        assert!(is_digit('9'));
    }

    #[test]
    fn test_is_digit_no_digit() {
        assert!(!is_digit('a'));
        assert!(!is_digit('%'));
        assert!(!is_digit('_'));
    }
}
