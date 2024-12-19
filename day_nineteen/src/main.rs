use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};

struct TrieNode {
    is_end: bool,
    children: HashMap<char, TrieNode>,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            is_end: false,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, word: &str) {
        let mut current = self;
        for c in word.chars() {
            current = current.children.entry(c).or_insert(TrieNode::new())
        }
        current.is_end = true;
    }
}

fn find_valid_patterns(input: &str, root: &TrieNode) -> bool {
    let n = input.len();
    let mut dp = vec![false; n + 1];
    dp[0] = true;

    for i in 0..n {
        if !dp[i] {
            continue;
        }

        let mut current = root;
        let mut j = i;
        while j < n {
            if let Some(next) = current.children.get(&input[j..=j].chars().next().unwrap()) {
                if next.is_end {
                    dp[j + 1] = true;
                }
                current = next;
                j += 1;
            } else {
                break;
            }
        }
    }

    dp[n]
}

fn count_valid_designs(input: &str, root: &TrieNode) -> u64 {
    let n = input.len();
    let mut dp = vec![0u64; n + 1];
    dp[0] = 1;

    for i in 0..n {
        if dp[i] == 0 {
            continue;
        }
        let mut current = root;
        let mut j = i;
        while j < n {
            if let Some(next) = current.children.get(&input[j..=j].chars().next().unwrap()) {
                if next.is_end {
                    dp[j + 1] += dp[i];
                }
                current = next;
                j += 1;
            } else {
                break;
            }
        }
    }
    dp[n]
}

fn main() {
    let towels: Vec<String> = read_to_string("towels.txt")
        .expect("Could not read file")
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut trie = TrieNode::new();
    for towel in towels {
        trie.insert(&towel);
    }

    let input_file = File::open("input.txt").expect("Could not open file");
    let reader = BufReader::new(input_file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let valid_count = lines
        .iter()
        .filter(|line| find_valid_patterns(line, &trie))
        .count();

    let design_count = lines
        .iter()
        .map(|line| count_valid_designs(line, &trie))
        .sum::<u64>();

    println!("Valid patterns: {}", valid_count);
    println!("Valid designs: {}", design_count);
}
