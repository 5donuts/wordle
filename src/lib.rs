//! A _Wordle_ clone

// Copyright (C) 2022 Charles German <5donuts@protonmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;

/// Count the occurrences of letters in the given string
macro_rules! letter_count {
    ($word:ident) => {{
        let mut letter_counts: HashMap<char, u8> = HashMap::new();
        for letter in $word.chars() {
            let count = *letter_counts.get(&letter).unwrap_or(&0);
            letter_counts.insert(letter, count + 1);
        }
        letter_counts
    }};
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LetterStatus {
    /// The guessed letter is in the correct position in the word (i.e., the green square)
    Correct,
    /// The guessed letter is in the incorrect position, but is in the word (i.e., the yellow square)
    InWord,
    /// The guessed letter is not in the word (i.e., the gray/black square)
    NotInWord,
}

#[derive(Debug)]
pub struct Wordle<'a> {
    /// (Pseudo-) Random Number Generator
    rand: Lazy<rand::rngs::ThreadRng>,
    /// Acceptable guesses
    guesses: HashSet<&'a str>,
    /// Answer list
    answers: &'a [&'a str],
    /// The currently selected word to play against
    word: Option<&'a str>,
}

impl<'a> Wordle<'a> {
    /// Initialize a new Wordle game
    pub fn new(guesses: &'a [&str], answers: &'a [&str]) -> Self {
        assert!(!guesses.is_empty());
        assert!(!answers.is_empty());

        Self {
            rand: Lazy::new(|| rand::thread_rng()),
            guesses: guesses.iter().map(|&s| s).collect(),
            answers,
            word: None,
        }
    }

    /// Choose the next word to play against
    pub fn choose_word(&mut self) {
        let word = self.answers.choose(&mut *self.rand).unwrap();
        self.word = Some(&word);
    }

    /// Guess a word and get back information about the guess.
    /// If the guess is not in the list of valid guesses, return `Err(())`.
    pub fn guess(&self, word: &str) -> Result<[LetterStatus; 5], ()> {
        assert_eq!(
            word.split_whitespace().count(),
            1,
            "Guess cannot contain whitespace characters"
        );
        assert_eq!(word.len(), 5, "Guess must have exactly 5 characters");

        let answer = self.word.expect("Game not initialized");
        assert_eq!(answer.len(), 5, "Answer must have exactly 5 characters");

        // keep track of the number of occurrences of letters in the word
        let mut letter_counts = letter_count!(answer);

        // ensure the guess is valid
        if self.guesses.contains(&word) {
            let mut statuses = [LetterStatus::NotInWord; 5];
            let word = word.chars();
            for (i, c) in word.enumerate() {
                let status = check_letter(answer, c, i, &mut letter_counts);
                statuses[i] = status;
            }
            Ok(statuses)
        } else {
            Err(())
        }
    }
}

/// Check a letter in a guess against the actual word
///
/// # Arguments
/// `word` - The word being guessed against
/// `letter` - The letter to check against `word`
/// `idx` - The position of `letter` in the guess
/// `remaining` - The count of remaining unguessed letters in the word
fn check_letter(
    word: &str,
    letter: char,
    idx: usize,
    remaining: &mut HashMap<char, u8>,
) -> LetterStatus {
    assert!(idx < 5, "idx must be in [0..5)");

    // if there is at least one remaining unguessed occurrence of letter in the word,
    // we need to check the position
    if let Some(count) = remaining.get(&letter) {
        if *count == 0 {
            return LetterStatus::NotInWord; // no occurrences remaining
        }

        // decrement count of unguessed occurrences
        let count = count - 1;
        remaining.insert(letter, count);

        // check the letter against the answer
        let word_letter_at_idx = word.chars().nth(idx).unwrap();
        if letter == word_letter_at_idx {
            LetterStatus::Correct
        } else {
            LetterStatus::InWord
        }
    }
    // if there are no remaining unguessed occurrences of the letter, then this
    // letter is a duplicate and is thus not in the word
    else {
        LetterStatus::NotInWord
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_count() {
        let word = "abcde";
        let mut expected: HashMap<char, u8> = HashMap::new();
        expected.insert('a', 1);
        expected.insert('b', 1);
        expected.insert('c', 1);
        expected.insert('d', 1);
        expected.insert('e', 1);
        let actual = letter_count!(word);
        assert_eq!(expected, actual, "Letter counts built improperly");
    }

    #[test]
    fn test_check_letter() {
        // letter in word in correct position
        let word = "abcde";
        let mut letter_counts = letter_count!(word);
        for i in 0..word.len() {
            let letter = word.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::Correct,
                check_letter(word, letter, i, &mut letter_counts),
                "Letter in word in correct position"
            );
        }

        // letter in word
        let word = "fghij";
        let mut letter_counts = letter_count!(word);
        let guesses = "ghijf"; // rotate the word
        for i in 0..word.len() {
            let letter = guesses.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::InWord,
                check_letter(word, letter, i, &mut letter_counts),
                "Letter in word, not in correct position"
            );
        }

        // letter not in word
        let word = "klmno";
        let mut letter_counts = letter_count!(word);
        let guesses = "abcde";
        for i in 0..word.len() {
            let letter = guesses.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::NotInWord,
                check_letter(word, letter, i, &mut letter_counts),
                "Letter not in word"
            );
        }

        // double letters, both in correct position
        let word = "aabcd";
        let mut letter_counts = letter_count!(word);
        let guess = "aabcd";
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::Correct,
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters, both in correct position"
            );
        }

        // double letters, both in wrong position
        let word = "aabcd";
        let mut letter_counts = letter_count!(word);
        let guess = "bcdaa";
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::InWord,
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters, both in wrong position"
            );
        }

        // double letters, one in correct position
        let word = "aabcd";
        let mut letter_counts = letter_count!(word);
        let guess = "abacd";
        let expected = vec![
            LetterStatus::Correct,
            LetterStatus::InWord,
            LetterStatus::InWord,
            LetterStatus::Correct,
            LetterStatus::Correct,
        ];
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                expected[i],
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters, one in correct position"
            );
        }

        // double letters, only one guessed (correct position)
        let word = "aabcd";
        let mut letter_counts = letter_count!(word);
        let guess = "axbcd";
        let expected = vec![
            LetterStatus::Correct,
            LetterStatus::NotInWord,
            LetterStatus::Correct,
            LetterStatus::Correct,
            LetterStatus::Correct,
        ];
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                expected[i],
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters, only one guessed (correct position)"
            );
        }

        // double letters, only one guessed (incorrect position)
        let word = "aabcd";
        let mut letter_counts = letter_count!(word);
        let guess = "xxacd";
        let expected = vec![
            LetterStatus::NotInWord,
            LetterStatus::NotInWord,
            LetterStatus::InWord,
            LetterStatus::Correct,
            LetterStatus::Correct,
        ];
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                expected[i],
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters, only one guessed (incorrect position)"
            );
        }

        // double letters guessed, only one in word (one correct position)
        let word = "abcde";
        let mut letter_counts = letter_count!(word);
        let guess = "aacde";
        let expected = vec![
            LetterStatus::Correct,
            LetterStatus::NotInWord,
            LetterStatus::Correct,
            LetterStatus::Correct,
            LetterStatus::Correct,
        ];
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                expected[i],
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters guessed, only one in word (one correct position)"
            );
        }

        // double letters guessed, only one in word (both incorrect position)
        let word = "abcde";
        let mut letter_counts = letter_count!(word);
        let guess = "xbcaa";
        let expected = vec![
            LetterStatus::NotInWord,
            LetterStatus::Correct,
            LetterStatus::Correct,
            LetterStatus::InWord,
            LetterStatus::NotInWord,
        ];
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                expected[i],
                check_letter(word, letter, i, &mut letter_counts),
                "Double letters guessed, only one in word (both incorrect position)"
            );
        }
    }

    // #[test]
    // fn test_guess() {
    //     assert!(false, "TODO: implement this test")
    // }
}
