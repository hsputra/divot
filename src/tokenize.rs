//! `imara-diff` only ships line-granularity tokenization out of the box
//! ([`TokenSource` for `&str`](imara_diff::TokenSource) splits on lines).
//! Word and char granularity are real, separate tokenizers implemented
//! here -- not something the underlying crate provides.

use imara_diff::TokenSource;

/// Splits `data` into alternating runs of "word" characters
/// (alphanumeric or `_`) and "non-word" characters (whitespace,
/// punctuation), each run becoming one token. Concatenating every token
/// losslessly reconstructs the original string -- same invariant
/// `imara-diff`'s own line tokenizer upholds by including the newline in
/// each line token.
#[derive(Clone, Copy)]
pub struct WordTokenizer<'a>(pub &'a str);

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> TokenSource for WordTokenizer<'a> {
    type Token = &'a str;
    type Tokenizer = WordTokens<'a>;

    fn tokenize(&self) -> Self::Tokenizer {
        WordTokens { remaining: self.0 }
    }

    fn estimate_tokens(&self) -> u32 {
        // Rough capacity hint, not required to be exact -- assume an
        // average run length of ~4 bytes (mixed word/whitespace runs).
        (self.0.len() as u32 / 4).max(1)
    }
}

#[derive(Clone, Copy)]
pub struct WordTokens<'a> {
    remaining: &'a str,
}

impl<'a> Iterator for WordTokens<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.remaining.is_empty() {
            return None;
        }
        let mut char_indices = self.remaining.char_indices();
        let (_, first_char) = char_indices.next().expect("checked non-empty above");
        let first_is_word = is_word_char(first_char);
        let split_at = char_indices
            .find(|&(_, c)| is_word_char(c) != first_is_word)
            .map_or(self.remaining.len(), |(idx, _)| idx);
        let (token, rest) = self.remaining.split_at(split_at);
        self.remaining = rest;
        Some(token)
    }
}

/// Splits `data` into one token per Unicode scalar value (`char`).
#[derive(Clone, Copy)]
pub struct CharTokenizer<'a>(pub &'a str);

impl<'a> TokenSource for CharTokenizer<'a> {
    type Token = char;
    type Tokenizer = std::str::Chars<'a>;

    fn tokenize(&self) -> Self::Tokenizer {
        self.0.chars()
    }

    fn estimate_tokens(&self) -> u32 {
        // Byte length over-estimates for multi-byte UTF-8 input, which is
        // fine -- this is only a `Vec` capacity hint, not a correctness
        // requirement.
        self.0.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_tokenizer_round_trips_losslessly() {
        let input = "hello, world!  foo_bar 123";
        let tokens: Vec<&str> = WordTokenizer(input).tokenize().collect();
        assert_eq!(tokens.concat(), input);
    }

    #[test]
    fn word_tokenizer_groups_word_chars_together() {
        let tokens: Vec<&str> = WordTokenizer("foo_bar123 baz").tokenize().collect();
        assert_eq!(tokens, vec!["foo_bar123", " ", "baz"]);
    }

    #[test]
    fn char_tokenizer_round_trips_losslessly() {
        let input = "héllo wörld";
        let tokens: Vec<char> = CharTokenizer(input).tokenize().collect();
        let rebuilt: String = tokens.into_iter().collect();
        assert_eq!(rebuilt, input);
    }
}
