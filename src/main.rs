#![feature(drain_filter)]
extern crate regex;
use regex::Regex;
use std::fs::File;
// use std::io::prelude::*;
use std::io::{self, BufRead, Write};
use std::path::Path;
mod frequency;
use frequency::*;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
struct Pattern {
    pub pattern: String,
    pub blanks: u32,
}
fn guess(word: &str, word_list: &[String]) -> Result<u32, String> {
    let mut quesses = 0_u32;
    let mut guessed: String = String::from("");
    let lowercase_word = &word.to_ascii_lowercase();
    let mut match_pattern = Pattern {
        pattern: (0..word.chars().count()).map(|_| ".").collect::<String>(),
        blanks: word.chars().count() as u32,
    };
    let mut words = word_list.to_owned();
    let size_filtered = words
        .drain_filter(|v| v.chars().count() == word.chars().count())
        .collect::<_>();
    words = size_filtered;
    'outer: loop {
        let count = words.len() / 32;
        let freq = frequency(&mut words, count);
        let mut count_vec: Vec<(&char, &usize)> = freq.iter().collect();
        count_vec.sort_by(|a, b| b.1.cmp(a.1));

        let mut iterator = count_vec.iter();
        loop {
            let mut letter = *iterator.next().unwrap().0;
            while guessed.contains(letter) {
                if let Some(lett) = iterator.next() {
                    letter = *lett.0;
                } else {
                    println!("{:?}", words);
                    println!("{}", lowercase_word);
                    println!("{}", match_pattern.pattern);
                    panic!("Should've not happend");
                }
            }
            guessed.push(letter);
            quesses += 1;
            if lowercase_word.contains(letter) {
                let mut new_pattern = String::with_capacity(match_pattern.pattern.chars().count());
                let mut iter = match_pattern.pattern.chars();
                for i in word.chars() {
                    let b = iter.next().unwrap();
                    if i == letter {
                        new_pattern.push(letter);
                        match_pattern.blanks -= 1;
                    } else {
                        new_pattern.push(b);
                    }
                }
                match_pattern.pattern = new_pattern;
                // println!("{}",match_pattern.pattern);
                if match_pattern.blanks == 0 {
                    break 'outer;
                }
            } else {
                let reg = Regex::new(match_pattern.pattern.as_str())
                    .expect("Incorrect regex, logic of program failed");
                let filtered_words = words.drain_filter(|v| reg.is_match(v)).collect::<Vec<_>>();
                //use not filtered words ...
                words = filtered_words;
                if words.is_empty() {
                    return Err("Not such word".to_string());
                }
                break;
            }
        }
    }

    Ok(quesses)
}
fn main() -> std::io::Result<()> {
    let args = &mut std::env::args();
    let mut single_word_mode = false;
    let mut single_word = String::new();
    let mut filename = String::from("words.txt");
    let mut iterations: usize = 1_000_000;

    let size = args.into_iter().len();
    if size > 2 {
        if let Some(_) = args.next() {
            while let Some(val) = args.next() {
                let mval: &str = &val.to_string();
                match mval {
                    "-i" => {
                        if let Some(val) = args.next() {
                            filename = val.to_owned();
                        } else {
                            eprintln!("Usage: -i <path>");
                        }
                    }
                    "-I" => {
                        if let Some(val) = args.next() {
                            iterations = val
                                .parse::<usize>()
                                .expect("Numbers of iterations must be a number");
                        } else {
                            eprintln!("Usage: -I <num>");
                        }
                    }
                    "-w" => {
                        if let Some(val) = args.next() {
                            single_word = val;
                            single_word_mode = true;
                        } else {
                            eprintln!("Usage -w <word>");
                        }
                    }
                    _ => {
                        eprintln!("No such command");
                    }
                }
            }
        }
    }
    if let Ok(lines) = read_lines(filename) {
        let words: Vec<String> = lines.collect::<Result<_, _>>().unwrap();
        let lines = words.to_owned();
        if single_word_mode {
            match guess(&single_word, &lines) {
                Ok(amount) => {
                    print!("Word {} took {} guesses",single_word, amount);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        } else {
            let mut max = 0;
            let mut hardest_word: String = String::new();
            let mut i: usize = 0;
            for word in lines {
                match guess(&word, &words) {
                    Ok(new_max) => {
                        if new_max > max {
                            max = new_max;
                            hardest_word = word;
                        }
                        i += 1;
                        print!(
                        "\r Worst to guess {: >10} with {: >2} guesses iteration {: >6} of {: >10}",
                        hardest_word,
                        max,
                        i,
                        words.len()
                    );
                        io::stdout().flush().unwrap();
                    }
                    Err(e) => {
                        io::stdout().flush().unwrap();
                        eprintln!("\n{} {}\n", e, word);
                    }
                }
                if i == iterations as usize {
                    break;
                }
            }
        }
    }
    println!();
    Ok(())
}
