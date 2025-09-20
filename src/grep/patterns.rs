fn is_in_range(lower_inclusive: char, upper_inclusive: char, char: char) -> bool {
    lower_inclusive <= char && char <= upper_inclusive
}

pub fn is_digit(char: char) -> bool {
    is_in_range('0', '9', char)
}

pub fn is_lower_case_letter(char: char) -> bool {
    is_in_range('a', 'z', char)
}

pub fn is_upper_case_letter(char: char) -> bool {
    is_in_range('A', 'Z', char)
}

pub fn is_word(char: char) -> bool {
    is_digit(char) || is_lower_case_letter(char) || is_upper_case_letter(char) || char == '_'
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

    #[test]
    fn test_is_lower_case_letter() {
        assert!(is_lower_case_letter('a'));
        assert!(is_lower_case_letter('q'));
        assert!(is_lower_case_letter('z'));
    }

    #[test]
    fn test_is_lower_case_letter_upper_case_letter() {
        assert!(!is_lower_case_letter('A'));
        assert!(!is_lower_case_letter('Q'));
        assert!(!is_lower_case_letter('Z'));
    }

    #[test]
    fn test_is_lower_case_letter_digit() {
        assert!(!is_lower_case_letter('3'))
    }

    #[test]
    fn test_is_lower_case_letter_symbol() {
        assert!(!is_lower_case_letter('_'))
    }

    #[test]
    fn test_is_upper_case_letter() {
        assert!(is_upper_case_letter('A'));
        assert!(is_upper_case_letter('Q'));
        assert!(is_upper_case_letter('Z'));
    }

    #[test]
    fn test_is_upper_case_letter_upper_case_letter() {
        assert!(!is_upper_case_letter('a'));
        assert!(!is_upper_case_letter('q'));
        assert!(!is_upper_case_letter('z'));
    }

    #[test]
    fn test_is_upper_case_letter_digit() {
        assert!(!is_upper_case_letter('3'))
    }

    #[test]
    fn test_is_upper_case_letter_symbol() {
        assert!(!is_upper_case_letter('_'))
    }

    #[test]
    fn test_is_word_lower_case_letter() {
        assert!(is_word('d'))
    }

    #[test]
    fn test_is_word_upper_case_letter() {
        assert!(is_word('G'))
    }

    #[test]
    fn test_is_word_digit() {
        assert!(is_word('7'))
    }

    #[test]
    fn test_is_word_underscore() {
        assert!(is_word('_'))
    }

    #[test]
    fn test_is_word_other_symbol(){
        assert!(!is_word('$'))
    }
}
