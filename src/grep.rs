mod patterns;
mod str;
mod syntax;
mod tokens;

use std::{collections::HashMap, ops::Deref};
use str::StringUtils;
use syntax::Syntax;

#[derive(Clone, Debug)]
struct Match {
    text: Vec<char>,
}

impl Match {
    /// Creates a match for the empty string.
    fn empty() -> Match {
        Match { text: vec![] }
    }

    /// Creates a Match from a single char that matched a single syntax item.
    fn from_char(text: char) -> Match {
        Match { text: vec![text] }
    }

    fn from_str(text: &str) -> Match {
        Match {
            text: text.chars().collect(),
        }
    }

    /// Merges two Matches, creating a new instance.
    fn merge(head: Match, tail: Match) -> Match {
        Match {
            text: [head.text, tail.text].concat(),
        }
    }

    /// Merges this Match instance with another one, mutating this instance.
    fn merge_with(&mut self, other: Match) {
        self.text.extend(other.text);
    }
}

fn is_match(char: char, pattern: &Syntax) -> Option<Match> {
    let is_match = match pattern {
        Syntax::Wildcard => true,
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

        Syntax::OneOrMore { .. } => panic!(
            "Only one-character matching syntax expected here, but found one or more quantifier"
        ),

        Syntax::ZeroOrOne { .. } => panic!(
            "Only one-character matching syntax expected here, but found zero or more quantifier"
        ),

        Syntax::CaptureGroup { .. } => panic!(
            "Only one-character matching syntax expected here, but found capture group quantifier"
        ),

        Syntax::CaptureGroupEnd { .. } => {
            panic!("Only one-character matching syntax expected here, but found capture group end")
        }

        Syntax::BackReference { .. } => {
            panic!("Only one-character matching syntax expected here, but found backreference")
        }
    };

    if is_match {
        Some(Match::from_char(char))
    } else {
        None
    }
}

fn match_star(
    text: &str,
    syntax: &Syntax,
    remainder: &[Syntax],
    cgroups: &mut HashMap<u32, Match>,
) -> Option<Match> {
    let mut match_head = Match::empty();
    let mut text_remainder = text;
    loop {
        if let Some(match_tail) = match_here(text_remainder, remainder, cgroups) {
            match_head.merge_with(match_tail);
            return Some(match_head);
        };

        let char = text_remainder.chars().next()?;
        let match_char = is_match(char, &syntax)?;

        match_head.merge_with(match_char);
        text_remainder = &text_remainder.slice(1..);
    }
}

fn match_question_mark(
    text: &str,
    syntax: &Syntax,
    pattern: &[Syntax],
    cgroups: &mut HashMap<u32, Match>
) -> Option<Match> {
    let pattern_once: Vec<Syntax> = [&[syntax.clone()], pattern].concat();

    if let Some(match_once) = match_here(text, &pattern_once, cgroups) {
        return Some(match_once);
    } else {
        return match_here(text, pattern, cgroups);
    }
}

fn match_here(text: &str, pattern: &[Syntax], cgroups: &mut HashMap<u32, Match>) -> Option<Match> {
    let Some(syntax) = pattern.get(0) else {
        // The entire pattern matched, return success.
        return Some(Match::empty());
    };

    if let Syntax::OneOrMore { syntax: s } = syntax {
        let match_head = match_here(text, &[(**s).clone()], cgroups)?;
        let match_tail = match_star(
            text.slice(match_head.text.len()..),
            s,
            &pattern[1..],
            cgroups,
        )?;

        return Some(Match::merge(match_head, match_tail));
    }

    if let Syntax::ZeroOrOne { syntax: s } = syntax {
        return match_question_mark(text, &s.deref(), &pattern[1..], cgroups);
    }

    if let Syntax::CaptureGroup { options: os, id } = syntax {
        let pattern_remainder = &pattern[1..];

        for option in os {
            let end = Syntax::CaptureGroupEnd {
                text: text.chars().collect(),
                id: *id,
            };
            let pattern_total = [option.as_slice(), &[end], pattern_remainder].concat();

            if let Some(match_total) = match_here(text, &pattern_total, cgroups) {
                return Some(match_total);
            }
        }

        return None;
    }

    if let Syntax::CaptureGroupEnd {
        text: text_original,
        id,
    } = syntax
    {
        let match_len = text_original.len() - text.len();
        let match_group = Match::from_str(text_original.slice(..match_len));

        let None = cgroups.insert(*id, match_group) else {
            panic!("Duplicate capture group result '{}'", id);
        };

        if let Some(match_remainder) = match_here(text, &pattern[1..], cgroups) {
            return Some(match_remainder);
        } else {
            // If the remainder does not match, we continue with the next option,
            // but the capture group result has to be discarded again.
            // Ignore the result here, since the capture group matching might or
            // might not have been successful.
            cgroups.remove(id).expect("Unable to remove capture group");
            return None;
        }
    }

    if let Syntax::BackReference { id } = syntax {
        let Some(match_original) = cgroups.get(id) else {
            panic!("No capture group with id '{}' has been matched yet", id);
        };

        let search_string: String = match_original.text.iter().collect();
        if text.starts_with(search_string.as_str()) {
            let match_ref = match_original.clone();
            let match_remainder = match_here(
                text.slice(match_original.text.len()..),
                &pattern[1..],
                cgroups,
            )?;

            return Some(Match::merge(match_ref, match_remainder));
        } else {
            return None;
        }
    }

    if let Syntax::EndOfLineAnchor = syntax {
        return (pattern.len() == 1 && text.len() == 0).then(|| Match::empty());
    }

    if let Some(c) = text.chars().next() {
        let match_char = is_match(c, syntax)?;
        let match_remainder = match_here(&text.slice(1..), &pattern[1..], cgroups)?;

        return Some(Match::merge(match_char, match_remainder));
    }

    return None;
}

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = tokens::tokenize_pattern(pattern);
    let syntax = syntax::parse_pattern(&tokens);
    let mut capture_groups = HashMap::new();

    if let Some(Syntax::StartOfLineAnchor) = syntax.get(0) {
        return match match_here(input_line, &syntax[1..], &mut capture_groups) {
            Some(_) => true,
            None => false,
        };
    }

    for start_index in 0..input_line.len() {
        if let Some(m) = match_here(
            &input_line.slice(start_index..),
            &syntax,
            &mut capture_groups,
        ) {
            println!("Match = {:?}", m.text.iter().collect::<String>());
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
    fn test_match_pattern_wildcard() {
        assert!(match_pattern("dog", "d.g"));
        assert!(!match_pattern("cat", "d.g"));
    }

    #[test]
    fn test_match_pattern_alternation() {
        assert!(match_pattern("cat", "(cat|dog)"));
        assert!(match_pattern("dog", "(cat|dog)"));
        assert!(!match_pattern("apple", "(cat|dog)"));
    }

    #[test]
    fn test_match_pattern_backreference() {
        assert!(match_pattern("cat and cat", "(cat) and \\1"));
        assert!(!match_pattern("cat and dog", "(cat) and \\1"));
        assert!(match_pattern("cat and cat", "(\\w+) and \\1"));
        assert!(match_pattern("dog and dog", "(\\w+) and \\1"));
        assert!(!match_pattern("cat and dog", "(\\w+) and \\1"));
        assert!(match_pattern("3 red squares and 3 red circles", "(\\d+) (\\w+) squares and \\1 \\2 circles"));
        assert!(!match_pattern("3 red squares and 4 red circles", "(\\d+) (\\w+) squares and \\1 \\2 circles"));
        assert!(match_pattern("'cat and cat' is the same as 'cat and cat'", "('(cat) and \\2') is the same as \\1"));
    }

    #[test]
    fn test_match_pattern_regression_tests() {
        assert!(!match_pattern("×-+=÷%", "\\w"));
        assert!(!match_pattern(
            "sally has 12 apples",
            "\\d\\\\d\\\\d apples"
        ));
        assert!(match_pattern("goøö0Ogol", "g.+gol"));
        assert!(match_pattern("a cat", "a (cat|dog)"));
        assert!(!match_pattern("once a dreaaamer, alwayszzz a dreaaamer", "once a (drea+mer), alwaysz? a \\1"));
        assert!(match_pattern("cat and fish, cat with fish, cat and fish", "((c.t|d.g) and (f..h|b..d)), \\2 with \\3, \\1"));
    }
}
