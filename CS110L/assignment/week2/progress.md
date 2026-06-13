link:https://web.stanford.edu/class/cs110l/assignments/week-2-exercises/
## Part1 hello world
```terminal of wsl
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2$ cargo new part-1-hello-world
    Creating binary (application) `part-1-hello-world` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2$ cd part-1-hello-world
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-1-hello-world$ cargo build
   Compiling part-1-hello-world v0.1.0 (/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-1-hello-world)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-1-hello-world$ ./target/debug/hello-world
-bash: ./target/debug/hello-world: No such file or directory
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-1-hello-world$ ./target/debug/part-1-he
llo-world
Hello, world!
```

## part2 shopping list.
```wsl
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-2-shopping-list$ cargo run
   Compiling part-2-shopping-list v0.1.0 (/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-2-shopping-list)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.27s
     Running `target/debug/part-2-shopping-list`
Enter an item to add to the list: nigga
Enter an item to add to the list: fucker
Enter an item to add to the list: SHittt
Enter an item to add to the list: bakaakaka
Enter an item to add to the list: doNE
Remember to buy:
* nigga
* fucker
* SHittt
* bakaakaka

```
learnt std , io method. maybe i shall learn more? but they seems basic.
whats more, i v encountered much errors while `cargo run`.It explains where I wrote wrong and really legible.

## part3 

## part4
```BASH
chiparon@chiparon:/mnt/e/Eproject/rustlearn-my-work/CS110L/assignment/week2/part-4-game$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/part-4-game`
Welcome to Guess the Word!
The word so far is -------
You have guessed the following letters:
You have 5 guesses left
Please guess a letter: h
The word so far is -------
You have guessed the following letters: h
You have 4 guesses left
Please guess a letter: a
The word so far is -------
You have guessed the following letters: h a
You have 3 guesses left
Please guess a letter: b
The word so far is --b----
You have guessed the following letters: h a b
You have 3 guesses left
Please guess a letter: o
The word so far is -ob----
You have guessed the following letters: h a b o
You have 3 guesses left
Please guess a letter: l
The word so far is lob----
You have guessed the following letters: h a b o l
You have 3 guesses left
Please guess a letter: s
The word so far is lobs---
You have guessed the following letters: h a b o l s
You have 3 guesses left
Please guess a letter: r
The word so far is lobs--r
You have guessed the following letters: h a b o l s r
You have 3 guesses left
Please guess a letter: t
The word so far is lobst-r
You have guessed the following letters: h a b o l s r t
You have 3 guesses left
Please guess a letter: e
Congratulations you guessed the secret word: lobster!
```