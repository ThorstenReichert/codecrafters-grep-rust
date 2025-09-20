#[derive(Debug, PartialEq)]
pub enum Syntax {
    
    /// Matches a single specified character.
    Literal { char: char },

    /// Matches a single digit. Equivalent to \[0-9\]
    Digit,

    /// Matches a single word character. Equivalent to \[a-zA-Z0-9_\].
    Word,

    /// Matches any one of the specified characters.
    CharacterClass { chars: Vec<char>, is_negated: bool }
}