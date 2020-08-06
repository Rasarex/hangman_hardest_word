#![feature(drain_filter)]
use std::fs::File;
// use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
mod errors;
use errors::*;
mod frequency;
use frequency::*;
use std::collections::HashMap;
use threadpool::ThreadPool;

fn is_match(word: &String, pattern: &String, negative_match: &String) -> bool {
    let mut pattern_iter = pattern.chars();
    for letter in word.chars() {
        let pattern_letter = pattern_iter.next().unwrap();
        if pattern_letter == '.' {
            if negative_match.contains(letter) {
                return false;
            }
        } else {
            if pattern_letter != letter {
                return false;
            }
        }
    }
    true
}

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

fn guess(
    word: &str,
    word_list: &[String],
    pool: &ThreadPool,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut quesses = 0_u32;
    let mut guessed: String = String::from("");
    let mut freq: Frequency = Frequency {
        map: HashMap::<char, usize>::new(),
        pool: pool,
    };
    let lowercase_word = &word.to_ascii_lowercase();
    let mut match_pattern = Pattern {
        pattern: (0..word.chars().count()).map(|_| ".").collect::<String>(),
        blanks: word.chars().count() as u32,
    };
    let mut words = word_list.to_owned();
    if words.len() == 1{
        return Ok(1);
    }
    'outer: loop {
        freq.frequency(&mut words);
        let mut count_vec: Vec<(&char, &usize)> = freq.map.iter().collect();
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
                return Err(Box::new(CmdError::NoSuchWord));
            }

            guessed.push(letter);
            // println!("{} {:?}",letter, match_pattern.pattern);
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
                words.drain_filter(|v| !is_match(v, &match_pattern.pattern, &guessed));
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = &mut std::env::args();
    let mut single_word_mode = false;
    let mut single_word = String::new();
    let mut filename = String::from("words.txt");
    let mut iterations: usize = 1_000_000;

    let size = args.into_iter().len();
    if size > 1 {
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
        let mut words: Vec<String> = lines.collect::<Result<_, _>>().unwrap();
        if single_word_mode {
            let pool = ThreadPool::new(WORKER_COUNT);
            words.drain_filter(|v| v.chars().count() != single_word.chars().count());
            let amount = guess(&single_word, &words, &pool)?;
            print!("Word {} took {} guesses", single_word, amount);
        } else {
            let mut max = 0;
            let mut i: usize = 0;
            let mut chunks: Vec<(Vec<String>,usize)> = Vec::new();

            for word in &words {
                if word.chars().count() > max {
                    max = word.chars().count()
                }
            }
            for i in 1..max{
                let l = words.drain_filter(|v| v.chars().count()  == i).collect::<Vec<_>>();
                let owned = l.to_owned();
                chunks.push((owned,i as usize));
            }
            let workers = ThreadPool::new(chunks.len());
            let (tx,rx) = std::sync::mpsc::channel();
            println!("{}",chunks.len());
            for words in chunks {
                let sender = tx.clone();
                let pool = workers.clone();
                let lines = words.to_owned();
                workers.execute( move || {
                    // let pool = ThreadPool::new(WORKER_COUNT);
                    let mut hardest_word: String = String::new();
                    let mut max = 0;
                    for word in lines.0 {

                        if let Ok(new_max) = guess(&word, &words.0, &pool) {
                            if new_max > max {
                                max = new_max;
                                hardest_word = word;
                            }
                            i += 1;
                            

                            if i == iterations as usize {
                                break;
                            }
                        }
                    }
                    // println!("{}: {:?}, {}",lines.1,hardest_word,max);
                    sender.send((hardest_word,max)).unwrap();
                });
            }
            drop(tx);
            let mut max : (String,u32) = (String::new(),0);
            for j in rx{
                if j.1 > max.1{
                    max = j;
                }
            }
            println!("Worst word {:?} took {} guesses ",max.0,max.1);
        }
    }
    println!();
    Ok(())
}
