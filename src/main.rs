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
fn guess(word: &str, word_list: &[String]) -> Result<u32, Box<CmdError>> {
    let mut quesses = 0_u32;
    let mut guessed: String = String::from("");
    let lowercase_word = &word.to_ascii_lowercase();
    let mut match_pattern = Pattern {
        pattern: (0..word.chars().count()).map(|_| ".").collect::<String>(),
        blanks: word.chars().count() as u32,
    };
    let mut words = word_list.to_owned();
    words.drain_filter(|v| v.chars().count() != word.chars().count());
    'outer: loop {
        let count = words.len() / 32;
        let freq = frequency(&mut words, count);
        let mut count_vec: Vec<(&char, &usize)> = freq.iter().collect();
        use std::cmp::Ordering;
        // make sort deterministic
        count_vec.sort_by(|a, b| {
            if b.1.cmp(a.1) == Ordering::Equal {
                a.0.cmp(b.0)
            } else {
                b.1.cmp(a.1)
            }
        });
        count_vec.drain_filter(|v| guessed.contains(*v.0));

        let mut iterator = count_vec.iter();
        loop {
            let letter;
            if let Some(lett) = iterator.next() {
                letter = *lett.0;
            } else {
                println!("{:?}", words);
                println!("{}", lowercase_word);
                println!("{}", match_pattern.pattern);
                panic!("Should've not happend");
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
                if match_pattern.blanks == 0 {
                    break 'outer;
                }
            } else {
                let reg = Regex::new(match_pattern.pattern.as_str())
                    .expect("Incorrect regex, logic of program failed");
                words.drain_filter(|v| !reg.is_match(v));
                //use not filtered words ...
                if words.is_empty() {
                    return Err(Box::new(CmdError::NoSuchWord));
                }
                if words.len() == 1 {
                    if word == words.iter().next().unwrap() {
                        return Ok(quesses);
                    } else {
                        return Err(Box::new(CmdError::NoSuchWord));
                    }
                }
                break;
            }
        }
    }

    Ok(quesses)
}
use std::fmt;
#[derive(Debug, Clone)]
enum CmdError {
    IterDefault,
    InputDefault,
    SingleWordDefault,
    IterWrongArgumentType,
    NoCmd,
    NoSuchWord,
}
impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CmdError::*;
        match self {
            IterDefault => write!(f, "Usage -I <num>"),
            InputDefault => write!(f, "Usage -i <input>"),
            SingleWordDefault => write!(f, "Usage -w <word>"),
            IterWrongArgumentType => write!(f, "Numbers of iteration must be a unsigned integer"),
            NoCmd => write!(f, "No such command"),
            NoSuchWord => write!(f, "No such word"),
        }
    }
}
impl std::error::Error for CmdError {}
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                            return Err(Box::new(CmdError::InputDefault));
                        }
                    }
                    "-I" => {
                        if let Some(val) = args.next() {
                            if let Ok(num_of_iter) = val.parse::<usize>() {
                                iterations = num_of_iter
                            } else {
                                return Err(Box::new(CmdError::IterWrongArgumentType));
                            }
                        } else {
                            return Err(Box::new(CmdError::IterDefault));
                        }
                    }
                    "-w" => {
                        if let Some(val) = args.next() {
                            single_word = val;
                            single_word_mode = true;
                        } else {
                            return Err(Box::new(CmdError::SingleWordDefault));
                        }
                    }
                    _ => {
                        return Err(Box::new(CmdError::NoCmd));
                    }
                }
            }
        }
    }
    if let Ok(lines) = read_lines(filename) {
        let words: Vec<String> = lines.collect::<Result<_, _>>().unwrap();
        let lines = words.to_owned();
        if single_word_mode {
            let amount = guess(&single_word, &lines)?;
            print!("Word {} took {} guesses", single_word, amount);
        } else {
            let mut max = 0;
            let mut hardest_word: String = String::new();
            let mut i: usize = 0;
            for word in lines {
                let new_max = guess(&word, &words)?;
                if new_max > max {
                    max = new_max;
                    hardest_word = word;
                }
                i += 1;
                print!(
                    "\r Worst to guess {: >12} with {: >2} guesses iteration {: >6} of {: >10}",
                    hardest_word,
                    max,
                    i,
                    words.len()
                );
                io::stdout().flush().unwrap();

                if i == iterations as usize {
                    break;
                }
            }
        }
    }
    println!();
    Ok(())
}
