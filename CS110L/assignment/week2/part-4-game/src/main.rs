use std::io::{self, Write};

fn main() {
    let secret = String::from("lobster");
    let mut guessed: Vec<char> = Vec::new();
    let mut guesses_left: usize = 5;

    println!("Welcome to Guess the Word!");

    while guesses_left > 0 {
        show_word_so_far(&guessed, &secret);
        println!("You have {} guesses left", guesses_left);

        let letter = guess_letter();

        if guessed.contains(&letter) {
            println!("You already guessed that letter.");
            continue;
        }

        guessed.push(letter);

        if !secret.contains(letter) {
            guesses_left -= 1;
        }

        if is_word_guessed(&guessed, &secret) {
            println!("Congratulations you guessed the secret word: {}!", secret);
            return;
        }
    }

    println!("Sorry, you ran out of guesses!");
}

fn show_word_so_far(guessed: &Vec<char>, secret: &String) {
    let mut show = String::new();

    for letter in secret.chars() {
        if guessed.contains(&letter) {
            show.push(letter);
        } else {
            show.push('-');
        }
    }

    println!("The word so far is {}", show);

    print!("You have guessed the following letters:");
    for letter in guessed {
        print!(" {}", letter);
    }
    println!();
}

fn guess_letter() -> char {
    print!("Please guess a letter: ");
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");

    input
        .trim()
        .chars()
        .next()
        .expect("please enter at least one character")
}

fn is_word_guessed(guessed: &Vec<char>, secret: &String) -> bool {
    for letter in secret.chars() {
        if !guessed.contains(&letter) {
            return false;
        }
    }

    true
}
