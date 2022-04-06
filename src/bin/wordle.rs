//! Runner for the Wordle game

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

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use wordle::LetterStatus;

/// Letter is in word in the correct position
const GREEN_SQ: &'static str = "🟩";
/// Letter is in word, but has incorrect position
const YELLOW_SQ: &'static str = "🟨";
/// Letter is not in word
const BLACK_SQ: &'static str = "⬛";

fn main() {
    // load the word lists
    let guess_list: Vec<&'static str> = read_word_list("./guesses.txt")
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str())) // String to &'static mut str
        .map(|s| &*s) // mut to immut
        .collect();
    let answer_list: Vec<&'static str> = read_word_list("./answers.txt")
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str())) // String to &'static mut str
        .map(|s| &*s) // mut to immut
        .collect();

    // initialize the game
    let mut game = wordle::Wordle::new(guess_list.as_slice(), answer_list.as_slice());

    let mut counter = 0;
    loop {
        game.choose_word();
        counter += 1;
        println!("--- Game {} started ---", counter);

        for i in 1..=6 {
            // get the user's guess & validate it against the allowed guesses list
            let (guess, guess_info) = loop {
                print!("Guess {}/{}: ", i, 6);
                std::io::stdout().flush().expect("Could not flush stdout"); // flush output

                let mut guess = String::new();
                io::stdin()
                    .read_line(&mut guess)
                    .expect("Failed to read line");
                let guess: String = guess.trim().into();

                let guess_res = game.guess(&guess);

                if guess_res.is_err() {
                    println!("Guess '{}' is not valid.", &guess);
                    continue; // keep making guesses
                } else {
                    break (guess.clone(), guess_res.unwrap()); // return the guess & guess info
                }
            };

            let info_str = guess_info
                .iter()
                .map(|status| status_to_str(status))
                .collect::<Vec<&str>>()
                .join("");

            println!("Guess:  {}\nResult: {}", &guess, &info_str);

            // check if the game is over
            if guess_info == [LetterStatus::Correct; 5] {
                println!("Congratulations!");
                break; // advance to the next game
            }
        }
    }
}

/// Read a word list from a file
fn read_word_list<P: AsRef<Path> + TryInto<String> + Copy>(path: P) -> Vec<String> {
    let words = fs::read_to_string(path).expect(
        format!(
            "Error reading file '{}'",
            path.try_into()
                .unwrap_or("Err: Could not display path".into())
        )
        .as_str(),
    );
    words
        .split_terminator("\n")
        .map(|s| {
            s.trim() // trim any '\r', ' ', etc
                .to_owned()
        })
        .collect()
}

/// Get the colored square to represent a [`LetterStatus`]
fn status_to_str(status: &LetterStatus) -> &'static str {
    match status {
        LetterStatus::Correct => GREEN_SQ,
        LetterStatus::InWord => YELLOW_SQ,
        LetterStatus::NotInWord => BLACK_SQ,
    }
}
