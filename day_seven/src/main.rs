use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn process_line(line: String) -> Option<u64> {
    let tokens: Vec<&str> = line.split(":").collect();
    let answer = tokens[0].trim().parse::<u64>().unwrap();
    let numbers: Vec<u64> = tokens[1]
        .trim()
        .split_whitespace()
        .map(|x| x.parse::<u64>().unwrap())
        .collect();

    let operations_count = numbers.len() - 1;
    let max_combinations = 3_u64.pow(operations_count as u32);

    'outer: for i in 0..max_combinations {
        let mut result = numbers[0];
        let mut temp_i = i;
        let mut ops = Vec::with_capacity(operations_count);

        for _ in 0..operations_count {
            ops.push(temp_i % 3);
            temp_i /= 3;
        }

        for (j, &op_code) in ops.iter().enumerate() {
            let next_num = numbers[j + 1];
            let new_result = match op_code {
                0 => result.checked_add(next_num),
                1 => result.checked_mul(next_num),
                2 => concatenate(result, next_num),
                _ => unreachable!(),
            };

            match new_result {
                Some(val) => result = val,
                None => continue 'outer,
            }
        }

        if result == answer {
            let mut expression = numbers[0].to_string();
            for (j, &op_code) in ops.iter().enumerate() {
                let op = match op_code {
                    0 => '+',
                    1 => '*',
                    2 => '|',
                    _ => unreachable!(),
                };
                expression.push(op);
                expression.push_str(&numbers[j + 1].to_string());
            }
            println!("{} = {}", expression, answer);
            return Some(answer);
        }
    }
    None
}

fn main() {
    let file = File::open("input.txt").expect("File not found");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let results: Vec<u64> = lines
        .par_iter()
        .filter_map(|line| process_line(line.clone()))
        .collect();

    println!("Result: {}", results.iter().sum::<u64>());
}

fn concatenate(a: u64, b: u64) -> Option<u64> {
    let b_str = b.to_string();
    let pow = 10_u64.checked_pow(b_str.len() as u32)?;
    a.checked_mul(pow)?.checked_add(b)
}
