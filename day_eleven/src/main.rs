use std::collections::HashMap;

fn main() {
    let input = "70949 6183 4 3825336 613971 0 15 182";
    let mut number_counts = input_to_map(input);

    for i in 0..75 {
        number_counts = process_numbers(number_counts);
        println!("Iteration {}: Count = {}", i, count_total(&number_counts));
    }
    println!("Final count: {}", count_total(&number_counts));
}

fn input_to_map(input: &str) -> HashMap<i64, i64> {
    let mut map = HashMap::new();
    input
        .split_whitespace()
        .map(|x| x.parse::<i64>().unwrap())
        .for_each(|n| *map.entry(n).or_insert(0) += 1);
    map
}

fn process_numbers(map: HashMap<i64, i64>) -> HashMap<i64, i64> {
    let mut new_map = HashMap::new();

    for (num, count) in map {
        if num == 0 {
            *new_map.entry(1).or_insert(0) += count;
        } else if count_digits(num) % 2 == 0 {
            let (left, right) = split_number(num);
            *new_map.entry(left).or_insert(0) += count;
            *new_map.entry(right).or_insert(0) += count;
        } else {
            *new_map.entry(num * 2024).or_insert(0) += count;
        }
    }

    new_map
}

fn count_total(map: &HashMap<i64, i64>) -> i64 {
    map.values().sum()
}

fn count_digits(mut n: i64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    n = n.abs();
    while n > 0 {
        n /= 10;
        count += 1;
    }
    count
}

fn split_number(n: i64) -> (i64, i64) {
    let digit_count = count_digits(n);
    let mid = digit_count / 2;
    let divisor = 10_i64.pow(mid as u32);
    let right = n % divisor;
    let left = n / divisor;
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_iterations() {
        let input = "125 17";
        let mut number_counts = input_to_map(input);
        for _ in 0..6 {
            number_counts = process_numbers(number_counts);
        }
        assert_eq!(count_total(&number_counts), 22);
    }

    #[test]
    fn twenty_five_iterations() {
        let input = "125 17";
        let mut number_counts = input_to_map(input);
        for _ in 0..25 {
            number_counts = process_numbers(number_counts);
        }
        assert_eq!(count_total(&number_counts), 55312);
    }
}
