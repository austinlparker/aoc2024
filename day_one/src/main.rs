use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn main() {
    let mut column1: Vec<i32> = Vec::new();
    let mut column2: Vec<i32> = Vec::new();

    if let Ok(file) = File::open("input.txt") {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                let numbers: Vec<&str> = line.split_whitespace().collect();
                if numbers.len() >= 2 {
                    if let Ok(num1) = numbers[0].parse::<i32>() {
                        column1.push(num1);
                    }
                    if let Ok(num2) = numbers[1].parse::<i32>() {
                        column2.push(num2);
                    }
                }
            }
        }
    } else {
        println!("Error: Could not open input file");
        return;
    }

    column1.sort();
    column2.sort();

    let total_distance = calculate_total_distance(&column1, &column2);
    let total_distance_sum = total_distance.iter().sum::<i32>();
    println!("Distance: {}", total_distance_sum);

    let similarity_score = calculate_similarity_score(&column1, &column2);
    println!("Similarity: {}", similarity_score);
}

fn calculate_total_distance(left: &[i32], right: &[i32]) -> Vec<i32> {
    let mut results: Vec<i32> = Vec::new();

    for (i, j) in left.iter().zip(right.iter()) {
        let distance = i - j;
        results.push(distance.abs());
    }
    results
}

fn calculate_similarity_score(left: &[i32], right: &[i32]) -> i32 {
    let mut frequencies: HashMap<i32, i32> = HashMap::new();
    let mut appearances: Vec<i32> = Vec::new();
    for &value in right {
        *frequencies.entry(value).or_insert(0) += 1;
    }

    for &value in left {
        if let Some(&count) = frequencies.get(&value) {
            appearances.push(count * value)
        }
    }
    appearances.iter().sum()
}
