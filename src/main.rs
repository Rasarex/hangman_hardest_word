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
fn guess(word: &String, word_list: &Vec<String>) -> u32 {
    let mut quesses = 0_u32;
    let mut guessed: String = String::from("");
    let lowercase_word = &word.to_ascii_lowercase();
    let mut match_pattern = Pattern {
        pattern: (0..word.chars().count()).map(|_| ".").collect::<String>(),
        blanks: word.chars().count() as u32,
    };
    let mut words = word_list.clone();
    words = words
        .drain_filter(|v| v.chars().count() == word.chars().count())
        .collect::<_>();
    'outer : loop {
        let freq = frequency(&mut words, 32);
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
                    println!("{}",match_pattern.pattern);
                    panic!("Should've not happend");
                }
            }
            guessed.push(letter);
            quesses += 1;
            if lowercase_word.contains(letter) {
                let mut new_pattern = String::with_capacity(match_pattern.pattern.chars().count());
                let mut iter  = match_pattern.pattern.chars().into_iter();
                for i in word.chars() {
                    let b = iter.next().unwrap();
                    if i == letter {
                        new_pattern.push(letter);
                        match_pattern.blanks -= 1;
                    }
                    else{
                        new_pattern.push(b);
                    }
                }
                match_pattern.pattern = new_pattern;
                // println!("{}",match_pattern.pattern);
                if match_pattern.blanks == 0 {
                    break 'outer;
                }
            } else {
                match Regex::new(match_pattern.pattern.as_str()){
                    Ok(reg) => { 
                        let filtered_words = words
                            .drain_filter(|v| !reg.is_match(v))
                            .collect::<Vec<_>>();
                        //use not filtered words ...
                        words = filtered_words;
                        if words.is_empty() {
                            break 'outer;
                        }
                        // println!("{:?}",match_pattern.pattern);
                        break;
                    }
                    Err(e) =>{
                        println!("{:?}",e);
                    }
                }
            }
            
        }
    }
    quesses
}
fn main() -> std::io::Result<()> {
    if let Ok(lines) = read_lines("words.txt") {
        let words: Vec<String> = lines.collect::<Result<_, _>>().unwrap();
        let lines = words.to_owned();
        let mut max = 0;
        let mut hardest_word : String = String::new();
        let mut i : usize = 0;
        // guess(&"alkylbenzenesulfonate".to_owned(), &lines);
        for word in lines {
            let new_max = guess(&word, &words);
        
            if new_max > max {
                max = new_max;
                hardest_word = word;
            }
            i +=1;
            print!("\r Worst to guess {: >10} with {} guesses iteration {: >6} of {: >10}", hardest_word, max, i, words.len());
            io::stdout().flush().unwrap();
        }
    }
    Ok(())
}
