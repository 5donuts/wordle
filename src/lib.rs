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

use std::collections::HashSet;

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;

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

        // ensure the guess is valid
        if self.guesses.contains(&word) {
            let answer = self.word.expect("Game not initialized");
            assert_eq!(answer.len(), 5, "Answer must have exactly 5 characters");

            let mut statuses = [LetterStatus::NotInWord; 5];
            let word = word.chars();
            for (i, c) in word.enumerate() {
                let status = check_letter(answer, c, i);
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
fn check_letter(word: &str, letter: char, idx: usize) -> LetterStatus {
    assert!(idx < 5, "idx must be in [0..5)");
    // letter is in word, need to check the position
    if word.contains(letter) {
        let word_letter_at_idx = word.chars().nth(idx).unwrap();
        // TODO: handle double letters
        if letter == word_letter_at_idx {
            LetterStatus::Correct
        } else {
            LetterStatus::InWord
        }
    }
    // letter not in word
    else {
        LetterStatus::NotInWord
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_letter() {
        // letter in word in correct position
        let word = "abcde";
        for i in 0..word.len() {
            let letter = word.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::Correct,
                check_letter(word, letter, i),
                "Letter in word in correct position"
            );
        }

        // letter in word
        let word = "fghij";
        let guesses = "ghijf"; // rotate the word
        for i in 0..word.len() {
            let letter = guesses.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::InWord,
                check_letter(word, letter, i),
                "Letter in word, not in correct position"
            );
        }

        // letter not in word
        let word = "klmno";
        let guesses = "abcde";
        for i in 0..word.len() {
            let letter = guesses.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::NotInWord,
                check_letter(word, letter, i),
                "Letter not in word"
            );
        }

        // double letters, both in correct position
        let word = "aabcd";
        let guess = "aabcd";
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::Correct,
                check_letter(word, letter, i),
                "Double letters, both in correct position"
            );
        }

        // double letters, both in wrong position
        let word = "aabcd";
        let guess = "bcdaa";
        for i in 0..word.len() {
            let letter = guess.chars().nth(i).unwrap();
            assert_eq!(
                LetterStatus::InWord,
                check_letter(word, letter, i),
                "Double letters, both in wrong position"
            );
        }

        // double letters, one in correct position
        let word = "aabcd";
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
                check_letter(word, letter, i),
                "Double letters, one in correct position"
            );
        }

        // double letters, only one guessed (correct position)
        let word = "aabcd";
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
                check_letter(word, letter, i),
                "Double letters, only one guessed (correct position)"
            );
        }

        // double letters, only one guessed (incorrect position)
        let word = "aabcd";
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
                check_letter(word, letter, i),
                "Double letters, only one guessed (incorrect position)"
            );
        }

        // double letters guessed, only one in word (one correct position)
        let word = "abcde";
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
                check_letter(word, letter, i),
                "Double letters guessed, only one in word (one correct position)"
            );
        }

        // double letters guessed, only one in word (both incorrect position)
        let word = "abcde";
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
                check_letter(word, letter, i),
                "Double letters guessed, only one in word (both incorrect position)"
            );
        }
    }

    // #[test]
    // fn test_guess() {
    //     assert!(false, "TODO: implement this test")
    // }
}
