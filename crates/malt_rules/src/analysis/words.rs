//! # Whole words in the current file, memoised
//!
//! A cheap pre-filter for checks that can only fire when the file names some
//! word, such as a call to `fi` or a `parfor` loop: when the word occurs nowhere
//! in the source, the check returns at once instead of walking the tree. The
//! answer is remembered per file, so asking it for every node costs one scan.

use std::cell::RefCell;

use crate::analysis::symbols::FileKey;

/// The words asked about in the current file, with their answers.
type WordMemo = Option<(FileKey, Vec<(&'static str, bool)>)>;

thread_local! {
    static MEMO: RefCell<WordMemo> = const { RefCell::new(None) };
}

/// Whether `word` occurs in `source` as a whole word: not preceded by a letter
/// or `_`, and not followed by a letter, digit or `_`. Every identifier token
/// with the text `word` is such an occurrence (comments and strings count too,
/// so the answer errs towards `true`).
pub(crate) fn has_word(source: &str, word: &'static str) -> bool {
    let key = FileKey::of(source);
    MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        let (memo_key, answers) = memo.get_or_insert_with(|| (key, Vec::new()));
        if *memo_key != key {
            *memo_key = key;
            answers.clear();
        }
        if let Some(&(_, found)) = answers.iter().find(|(w, _)| *w == word) {
            return found;
        }
        let found = scan(source, word);
        answers.push((word, found));
        found
    })
}

/// [`has_word`] without the memo.
fn scan(source: &str, word: &str) -> bool {
    let bytes = source.as_bytes();
    source.match_indices(word).any(|(at, _)| {
        let before = at.checked_sub(1).map(|i| bytes[i]);
        let after = bytes.get(at + word.len()).copied();
        !before.is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            && !after.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only() {
        assert!(scan("y = fi(0, 1, 16);", "fi"));
        assert!(scan("fi", "fi"));
        assert!(scan("x=1;fi(2)", "fi"));
        assert!(!scan("f = find(x); file = 1; xfi = 2; fi_2 = 3;", "fi"));
        assert!(scan("parfor i = 1:3", "parfor"));
        assert!(!scan("myparfor = 1;", "parfor"));
    }

    #[test]
    fn the_memo_follows_the_file() {
        malt_core::note_parse();
        let a = String::from("y = fi(1);");
        assert!(has_word(&a, "fi"));
        malt_core::note_parse();
        let b = String::from("y = f(1);");
        assert!(!has_word(&b, "fi"));
        malt_core::note_parse();
        assert!(has_word(&a, "fi"));
    }
}
