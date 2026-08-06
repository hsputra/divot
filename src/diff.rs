use imara_diff::{Algorithm, Diff, InternedInput, Token};

use crate::tokenize::{CharTokenizer, WordTokenizer};

/// One contiguous run of tokens in a diff result: unchanged (present
/// identically in both `before` and `after`), added, or removed. Mirrors
/// jsdiff's `Change` shape (`{value, added, removed, count}`) so the npm
/// binding can expose an API shaped like the incumbent it's meant to be
/// an easy migration from.
///
/// `value` borrows directly from the original `before`/`after` input --
/// every run is, by construction, a contiguous byte range of the input
/// it came from, so there is no need to copy it into an owned `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change<'a> {
    pub value: &'a str,
    pub added: bool,
    pub removed: bool,
    pub count: usize,
}

impl<'a> Change<'a> {
    fn unchanged(value: &'a str, count: usize) -> Self {
        Change { value, added: false, removed: false, count }
    }

    fn added(value: &'a str, count: usize) -> Self {
        Change { value, added: true, removed: false, count }
    }

    fn removed(value: &'a str, count: usize) -> Self {
        Change { value, added: false, removed: true, count }
    }
}

/// Walks a computed [`Diff`]'s hunks and the full token sequences to
/// produce the complete list of [`Change`]s -- unchanged runs *and*
/// changed ones, not just the hunks. `imara-diff`'s own `Diff` only
/// tracks what changed; reconstructing the interleaved full sequence
/// (what jsdiff's `diffLines`/`diffWords`/`diffChars` return) is this
/// crate's own work on top of it.
///
/// Every run is sliced directly out of `before_text`/`after_text` by
/// byte range -- computed by summing token byte-lengths as the token-
/// index cursor advances, since tokens are contiguous and
/// order-preserving by construction. No per-token copying.
fn hunks_to_changes<'a, T>(
    diff: &Diff,
    input: &InternedInput<T>,
    before_text: &'a str,
    after_text: &'a str,
    token_len: impl Fn(&T) -> usize,
) -> Vec<Change<'a>> {
    let run_byte_len = |tokens: &[Token], range: std::ops::Range<u32>| -> usize {
        tokens[range.start as usize..range.end as usize]
            .iter()
            .map(|&tok| token_len(&input.interner[tok]))
            .sum()
    };

    let mut changes = Vec::new();
    let mut pos_before: u32 = 0;
    let mut byte_before: usize = 0;
    let mut byte_after: usize = 0;

    for hunk in diff.hunks() {
        if hunk.before.start > pos_before {
            let len = run_byte_len(&input.before, pos_before..hunk.before.start);
            let count = (hunk.before.start - pos_before) as usize;
            changes.push(Change::unchanged(&before_text[byte_before..byte_before + len], count));
            byte_before += len;
            // An unchanged run has identical content (hence identical
            // byte length) on both sides by definition.
            byte_after += len;
        }

        if !hunk.before.is_empty() {
            let len = run_byte_len(&input.before, hunk.before.clone());
            let count = (hunk.before.end - hunk.before.start) as usize;
            changes.push(Change::removed(&before_text[byte_before..byte_before + len], count));
            byte_before += len;
        }
        if !hunk.after.is_empty() {
            let len = run_byte_len(&input.after, hunk.after.clone());
            let count = (hunk.after.end - hunk.after.start) as usize;
            changes.push(Change::added(&after_text[byte_after..byte_after + len], count));
            byte_after += len;
        }

        pos_before = hunk.before.end;
    }

    if (pos_before as usize) < input.before.len() {
        let len = run_byte_len(&input.before, pos_before..input.before.len() as u32);
        let count = input.before.len() - pos_before as usize;
        changes.push(Change::unchanged(&before_text[byte_before..byte_before + len], count));
    }

    changes
}

/// Diffs `before`/`after` line by line (the newline is included in each
/// line's token, same convention `imara-diff` uses, so changing the final
/// line's trailing newline is itself detected as a change).
pub fn line_diff<'a>(before: &'a str, after: &'a str, algorithm: Algorithm) -> Vec<Change<'a>> {
    let input = InternedInput::new(before, after);
    let mut diff = Diff::compute(algorithm, &input);
    diff.postprocess_lines(&input);
    hunks_to_changes(&diff, &input, before, after, |tok: &&str| tok.len())
}

/// Diffs `before`/`after` word by word. "Word" means a maximal run of
/// alphanumeric/`_` characters; runs of whitespace/punctuation between
/// words are their own tokens, so e.g. a single added space is reported
/// as its own small change rather than merged into a neighboring word.
pub fn word_diff<'a>(before: &'a str, after: &'a str, algorithm: Algorithm) -> Vec<Change<'a>> {
    let input = InternedInput::new(WordTokenizer(before), WordTokenizer(after));
    let diff = Diff::compute(algorithm, &input);
    hunks_to_changes(&diff, &input, before, after, |tok: &&str| tok.len())
}

/// Diffs `before`/`after` character by character.
pub fn char_diff<'a>(before: &'a str, after: &'a str, algorithm: Algorithm) -> Vec<Change<'a>> {
    let input = InternedInput::new(CharTokenizer(before), CharTokenizer(after));
    let diff = Diff::compute(algorithm, &input);
    hunks_to_changes(&diff, &input, before, after, |tok: &char| tok.len_utf8())
}

/// Renders a unified diff (`git diff`/`diff -u` style patch text) for
/// `before`/`after`. Thin wrapper over `imara-diff`'s own `unified_diff`
/// feature -- no reimplementation needed here, unlike the tokenizers
/// above.
pub fn unified_diff(before: &str, after: &str, algorithm: Algorithm) -> String {
    use imara_diff::{BasicLineDiffPrinter, UnifiedDiffConfig};

    let input = InternedInput::new(before, after);
    let mut diff = Diff::compute(algorithm, &input);
    diff.postprocess_lines(&input);
    diff.unified_diff(&BasicLineDiffPrinter(&input.interner), UnifiedDiffConfig::default(), &input)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_diff_reconstructs_before_and_after() {
        let before = "abc\ndef\nghi\n";
        let after = "abc\nDEF\nghi\n";
        let changes = line_diff(before, after, Algorithm::Histogram);

        let reconstructed_before: String = changes.iter().filter(|c| !c.added).map(|c| c.value).collect();
        let reconstructed_after: String = changes.iter().filter(|c| !c.removed).map(|c| c.value).collect();

        assert_eq!(reconstructed_before, before);
        assert_eq!(reconstructed_after, after);
    }

    #[test]
    fn line_diff_identical_input_is_all_unchanged() {
        let text = "same\nlines\n";
        let changes = line_diff(text, text, Algorithm::Histogram);
        assert_eq!(changes.len(), 1);
        assert!(!changes[0].added && !changes[0].removed);
        assert_eq!(changes[0].value, text);
    }

    #[test]
    fn word_diff_reconstructs_before_and_after() {
        let before = "the quick brown fox";
        let after = "the slow brown fox";
        let changes = word_diff(before, after, Algorithm::Histogram);

        let reconstructed_before: String = changes.iter().filter(|c| !c.added).map(|c| c.value).collect();
        let reconstructed_after: String = changes.iter().filter(|c| !c.removed).map(|c| c.value).collect();

        assert_eq!(reconstructed_before, before);
        assert_eq!(reconstructed_after, after);
        // "quick" replaced by "slow"; "the ", " brown fox" unchanged.
        assert!(changes.iter().any(|c| c.removed && c.value == "quick"));
        assert!(changes.iter().any(|c| c.added && c.value == "slow"));
    }

    #[test]
    fn char_diff_reconstructs_before_and_after() {
        let before = "cat";
        let after = "cot";
        let changes = char_diff(before, after, Algorithm::Histogram);

        let reconstructed_before: String = changes.iter().filter(|c| !c.added).map(|c| c.value).collect();
        let reconstructed_after: String = changes.iter().filter(|c| !c.removed).map(|c| c.value).collect();

        assert_eq!(reconstructed_before, before);
        assert_eq!(reconstructed_after, after);
    }

    #[test]
    fn char_diff_handles_multibyte_utf8() {
        let before = "héllo wörld";
        let after = "héllo wArld";
        let changes = char_diff(before, after, Algorithm::Histogram);

        let reconstructed_before: String = changes.iter().filter(|c| !c.added).map(|c| c.value).collect();
        let reconstructed_after: String = changes.iter().filter(|c| !c.removed).map(|c| c.value).collect();

        assert_eq!(reconstructed_before, before);
        assert_eq!(reconstructed_after, after);
    }

    #[test]
    fn unified_diff_contains_hunk_header() {
        let before = "a\nb\nc\n";
        let after = "a\nB\nc\n";
        let patch = unified_diff(before, after, Algorithm::Histogram);
        assert!(patch.contains("@@"));
        assert!(patch.contains("-b\n") || patch.contains("-b"));
        assert!(patch.contains("+B\n") || patch.contains("+B"));
    }

    #[test]
    fn myers_and_histogram_both_reconstruct_correctly() {
        let before = "one\ntwo\nthree\nfour\n";
        let after = "one\nTWO\nthree\nFOUR\n";
        for algorithm in [Algorithm::Histogram, Algorithm::Myers] {
            let changes = line_diff(before, after, algorithm);
            let reconstructed_before: String = changes.iter().filter(|c| !c.added).map(|c| c.value).collect();
            let reconstructed_after: String = changes.iter().filter(|c| !c.removed).map(|c| c.value).collect();
            assert_eq!(reconstructed_before, before);
            assert_eq!(reconstructed_after, after);
        }
    }
}
