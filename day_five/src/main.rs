use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct PageOrder {
    graph: HashMap<u32, HashSet<u32>>,
    reverse_graph: HashMap<u32, HashSet<u32>>,
}

impl PageOrder {
    fn new() -> Self {
        PageOrder {
            graph: HashMap::new(),
            reverse_graph: HashMap::new(),
        }
    }

    fn add_rule(&mut self, before: u32, after: u32) {
        self.graph
            .entry(before)
            .or_insert_with(HashSet::new)
            .insert(after);
        self.reverse_graph
            .entry(after)
            .or_insert_with(HashSet::new)
            .insert(before);
    }

    fn is_valid(&self, pages: &[u32]) -> bool {
        for i in 0..pages.len() {
            for j in (i + 1)..pages.len() {
                let before = pages[i];
                let after = pages[j];
                if let Some(rule) = self.graph.get(&after) {
                    if rule.contains(&before) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn fix_order(&self, pages: &[u32]) -> Vec<u32> {
        let mut fixed = pages.to_vec();
        let mut swapped = true;
        while swapped {
            swapped = false;
            for i in 0..fixed.len() - 1 {
                let before = fixed[i];
                let after = fixed[i + 1];
                if let Some(rule) = self.graph.get(&after) {
                    if rule.contains(&before) {
                        fixed.swap(i, i + 1);
                        swapped = true;
                    }
                }
            }
        }
        fixed
    }
}

fn main() {
    let mut page_order = PageOrder::new();
    let mut result = 0;
    let mut fixed_result = 0;
    let rules_file = File::open("rules.txt").unwrap();
    let books_file = File::open("books.txt").unwrap();
    let rules_reader = BufReader::new(rules_file);
    for line in rules_reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split('|').collect();
        let before = parts[0].parse::<u32>().unwrap();
        let after = parts[1].parse::<u32>().unwrap();
        page_order.add_rule(before, after);
    }
    let books_reader = BufReader::new(books_file);
    for line in books_reader.lines() {
        let line = line.unwrap();
        let pages: Vec<u32> = line.split(',').map(|s| s.parse().unwrap()).collect();
        if page_order.is_valid(&pages) {
            let middle = pages.len() / 2;
            result += pages[middle] as u64;
            println!("Valid: {:?}", pages);
        } else {
            println!("Invalid: {:?}, fixing.", pages);
            let fixed = page_order.fix_order(&pages);
            let middle = fixed.len() / 2;
            fixed_result += fixed[middle] as u64;
        }
    }
    println!("Result: {}", result);
    println!("Fixed result: {}", fixed_result);
}
