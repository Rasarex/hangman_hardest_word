use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

const MULTI_THRESHOLD: usize = 32;

pub fn frequency(input: &mut Vec<String>, worker_count: usize) -> HashMap<char, usize> {
    if worker_count < 2 || input.len() / worker_count < MULTI_THRESHOLD {
        return single_thread(input);
    }
    let (tx, rx) = mpsc::channel();
    let size = input.len();
    let each = size / worker_count;

    for slice in input.chunks(each) {
        let clone = tx.clone();
        let slice = slice.to_owned();
        thread::spawn(move || {
            let mut map = HashMap::new();
            for line in slice {
                for c in line.chars() {
                    push_map(&mut map, c, 1);
                }
            }
            if let Err(e) = clone.send(map) {
                println!("{}", e);
            };
        });
    }
    drop(tx);
    let mut map = HashMap::new();
    for j in rx {
        merge(&mut map, &j);
    }

    map
}

fn single_thread(input: &mut Vec<String>) -> HashMap<char, usize> {
    let mut map = HashMap::new();
    for line in input.iter() {
        for c in line.chars() {
            push_map(&mut map, c, 1);
        }
    }
    map
}

fn merge(first: &mut HashMap<char, usize>, second: &HashMap<char, usize>) {
    for (key, value) in second.iter() {
        push_map(first, *key, *value);
    }
}
fn push_map(first: &mut HashMap<char, usize>, key: char, value: usize) {
    let key = key.to_ascii_lowercase();
    first
        .entry(key)
        .and_modify(|v| *v += value)
        .or_insert(value);
}
