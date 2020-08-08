#![feature(drain_filter)]
mod errors;
mod frequency;
mod settings;

use errors::*;
use frequency::*;
use settings::*;

use std::fs::File;
// use std::io::prelude::*;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::Path;
use threadpool::ThreadPool;

fn is_match(word: &str, pattern: &String, negative_match: &String) -> bool {
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
fn inprint_letter_on_pattern(word: &String, letter: char, pattern: &String) -> String {
    let mut new_pattern = String::with_capacity(pattern.chars().count());
    let mut iter = pattern.chars();
    for i in word.chars() {
        let b = iter.next().unwrap();
        if i == letter {
            new_pattern.push(letter);
        } else {
            new_pattern.push(b);
        }
    }
    new_pattern
}
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn guess_all<'a>(word_list: Vec<&'static str>, pool: &ThreadPool, word_size: usize, iterations : u64, curr_iter : &mut  u64) -> Vec<(&'static str, u32)> {
    struct Guesses {
        pub pattern: String,
        pub matching_words: Vec<&'static str>,
        pub guessing_depth: usize,
        pub guessed: String,
    }
    let mut recording: Vec<(&'static str, u32)> = Vec::new();
    let mut chunks: Vec<Guesses> = Vec::new();
    let mut freq = Frequency {
        map: HashMap::<char, usize>::new(),
        pool: pool,
    };

    let pattern = (0..word_size).map(|_| '.').collect::<String>();
    chunks.push(Guesses {
        pattern,
        matching_words: Vec::from(word_list),
        guessing_depth: 0,
        guessed: "".to_owned(),
    });
    let max_iter = 10_000;
    let mut loop_iter = 0;
    loop {
        let mut new_chunks: Vec<Guesses> = Vec::new();
        if chunks.len() == 0 {
            break;
        }
        for chunk in &mut chunks {
            if chunk.matching_words.len() == 0 {
                continue;
            }
            freq.frequency(&chunk.matching_words);
            let mut sorted = freq.to_sorted_vec();
            // sorted.drain_filter(|v| chunk.guessed.contains(*v.0));
            if sorted.len() == 0 {
                continue;
            }
            let mut sort_iter = sorted.iter();
            let mut letter = *sort_iter.next().unwrap().0;

            let guessed = chunk.guessed.to_owned();
           // let owned = guessed.to_owned();
            'outer: while chunk.matching_words.len() > 0 {
                let matching = &mut chunk.matching_words;
                let word = matching.iter().to_owned().next().unwrap().to_owned();
                let mut curr =  guessed.clone();
                loop {
                    if !word.contains(letter) {
                        curr.push(letter);

                        if let Some(lett) = sort_iter.next() {
                            letter = *lett.0;
                            continue;
                        } else {
                            // let matching =  &mut matching;
                            *curr_iter += 1;
                            matching.remove(0);
                            recording.push((
                                word,
                                (chunk.guessing_depth + 1) as u32
                            ));
                            let size = matching.len();
                            if size == 0 {break 'outer;} else { println!("{}",size)}
                            println!("!{}",*curr_iter);
                            if *curr_iter >= iterations {return recording;}
                            continue 'outer; 
                        }
                    }
                    break;
                }
                let new_pattern = inprint_letter_on_pattern(&word.to_owned(), letter, &chunk.pattern);

                // let owned = guessed.to_owned();
                // let curr_word = word.to_owned();
                let matching_to_new_pattern = chunk
                    .matching_words
                    .drain_filter(|v| is_match(v, &new_pattern, &guessed))
                    .collect::<Vec<_>>();

                if matching_to_new_pattern.len() == 0 {
                    if let Some(lett) = sort_iter.next() {
                        letter = *lett.0;
                        curr.push(letter);
                        *curr_iter +=1;
                        if *curr_iter >= iterations {return recording;}
                        continue;
                    } else {
                        recording.push((word, (chunk.guessing_depth + 1) as u32));
                    }
                    break 'outer;
                }
                if matching_to_new_pattern.len() == 1 {
                    *curr_iter +=1;
                    recording.push((
                        matching_to_new_pattern.iter().next().unwrap().clone(),
                        (chunk.guessing_depth + 1) as u32,
                    ));
                    println!("!{}",*curr_iter);
                    if *curr_iter >= iterations {return recording;}
                    continue;
                } else {
                    new_chunks.push(Guesses {
                        pattern: new_pattern,
                        guessed: curr,
                        matching_words: matching_to_new_pattern,
                        guessing_depth: chunk.guessing_depth + 1,
                    });
                }
                
                let size = chunk.matching_words.len();
                if size == 0 {break 'outer;} else { println!("{}",size)}
                println!("!{}",*curr_iter);
                if *curr_iter >= iterations {return recording;}
                
            }
        }
        drop(chunks);
        chunks = new_chunks;
        if  loop_iter >= max_iter {
            return  recording;
        }
        loop_iter += 1;
        if *curr_iter >= iterations {return recording;}
    }
    recording
}
fn guess(
    word: &str,
    mut words: Vec<&'static str>,
    pool: &ThreadPool,
) -> Result<u32, Box<dyn std::error::Error>> {
    struct Pattern {
        pub pattern: String,
        pub blanks: u32,
    }

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
    if words.len() == 1 {
        return Ok(1);
    }
    'outer: loop {
        freq.frequency(&words);
        let mut count_vec = freq.to_sorted_vec();
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
                if words.is_empty() {
                    return Err(Box::new(CmdError::NoSuchWord));
                }
                if words.len() == 1 {
                    if word == *words.iter().next().unwrap() {
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
fn to_static(word : String)-> &'static str{
    Box::leak(word.into_boxed_str())
}
fn to_static_slice<T : 'static >(vector : Vec<T>) -> &'static [T]{
    Box::leak(vector.into_boxed_slice())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = settings_from_args()?;
    if let Ok(lines) = read_lines(settings.path) {
        let words: Vec<String> = lines.collect::<Result<Vec<String>, _>>().unwrap();
        let mut words = words.iter().map(|v| to_static(v.to_owned())).collect::<Vec<_>>();
        let word_count = words.len();
        if let Some(single_word) = settings.single_word {
            let pool = ThreadPool::new(WORKER_COUNT);
            words.split(|v| v.chars().count() != single_word.chars().count());
            let amount = guess(&single_word, words, &pool)?;
            print!("Word {} took {} guesses", single_word, amount);
        } else {
            let mut max = 0;
            let mut chunks: Vec<(Vec<&'static str>, usize)> = Vec::new();

            for word in &words {
                if word.chars().count() > max {
                    max = word.chars().count()
                }
            }
            for i in 1..max {
                let l = words
                    .drain_filter(|v| v.chars().count() == i)
                    .collect::<Vec<_>>();
                let owned = l.to_owned();
                chunks.push((owned, i as usize));
            }
            let workers = ThreadPool::new(chunks.len());
            let (tx, rx) = std::sync::mpsc::channel();
            println!("{}", chunks.len());
            let iterations = settings.iterations;
            let mut curr_iter = 0;
            for words in chunks {
                let sender = tx.clone();
                let pool = workers.clone();

                workers.execute(move || {
                    let mut hardest_word: String = String::new();
                    let mut max = 0;
                    let recording = guess_all(words.0, &pool, words.1,  iterations as u64, &mut curr_iter );
                    let len = recording.len();
                    for (word, geusses) in recording {
                        if geusses > max {
                            hardest_word = (*word).to_string();
                            max = geusses;
                        }
                    }

                    println!("{}: {:?}, {}", words.1, hardest_word, max);
                    sender.send((hardest_word, max, len)).unwrap();
                });
            }
            drop(tx);
            let mut max: (String, u32, usize) = (String::new(), 0, 0);
            let mut iterations = 0;
            for j in rx {
                iterations += j.2;
                if j.1 > max.1 {
                    max = j;
                }
            }
            println!(
                "Worst word {:?} took {} guesses and checked {} words of {}",
                max.0, max.1, iterations, word_count
            );
        }
    }
    println!();
    Ok(())
}
