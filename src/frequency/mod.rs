use std::collections::HashMap;
use std::sync::mpsc;
use threadpool::ThreadPool;
extern crate lazy_static;

pub const WORKER_COUNT: usize = 4;
pub struct Frequency<'a> {
    pub map: HashMap<char, usize>,
    pub pool: &'a ThreadPool,
}
impl<'a> Frequency<'a> {
    pub fn frequency(&mut self, input: &mut Vec<String>) {
        self.map.clear();
        let (tx, rx) = mpsc::channel();
        let size = input.len();
        let each = size / WORKER_COUNT;
        if each == 0 {
            self.map = single_thread(input);
            return;
        }

        for slice in input.chunks(each) {
            let clone = tx.clone();
            let slice = slice.to_owned();
            self.pool.execute(move || {
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
        for j in rx {
            merge(&mut self.map, &j);
        }
    }
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
