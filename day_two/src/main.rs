use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let mut safe_count = 0;
    let mut safe_ish_count = 0;
    let data = load_input("input.txt")?;
    println!("Loaded {} reports.", data.len());
    for report in &data {
        if calculate_safety_strict(report) {
            safe_count += 1;
            safe_ish_count += 1;
            println!("Vector {:?} is strictly safe", report);
        } else if check_safety_ish(report) {
            println!("Vector {:?} is safe-ish", report);
            safe_ish_count += 1;
        } else {
            println!("Vector {:?} is not safe even after removals", report);
        }
    }
    println!("Safe count: {}", safe_count);
    println!("Safe-ish count: {}", safe_ish_count);
    Ok(())
}

fn load_input(file_path: &str) -> io::Result<Vec<Vec<i32>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut data = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let numbers: Vec<i32> = line
            .split_whitespace()
            .map(|n| n.parse().unwrap())
            .collect();
        data.push(numbers);
    }
    Ok(data)
}

fn calculate_safety_strict(report: &[i32]) -> bool {
    if report.len() < 2 {
        return true;
    }

    let is_increasing_monotonically = report.windows(2).all(|pair| pair[0] < pair[1]);
    let is_decreasing_monotonically = report.windows(2).all(|pair| pair[0] > pair[1]);
    let condition_one = is_increasing_monotonically || is_decreasing_monotonically;

    let condition_two = report.windows(2).all(|pair| {
        let (a, b) = (pair[0], pair[1]);
        ((b - a).abs() >= 1) && ((b - a).abs() <= 3)
    });

    condition_one && condition_two
}

fn check_safety_ish(report: &[i32]) -> bool {
    if calculate_safety_strict(report) {
        return true;
    }

    for i in 0..report.len() {
        let mut mod_report: Vec<i32> = report.to_vec();
        mod_report.remove(i);
        if calculate_safety_strict(&mod_report) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_DATA: &[[i32; 5]] = &[
        [7, 6, 4, 2, 1],
        [1, 2, 7, 8, 9],
        [9, 7, 6, 2, 1],
        [1, 3, 2, 4, 5],
        [8, 6, 4, 4, 1],
        [1, 3, 6, 7, 9],
    ];

    #[test]
    fn test_safety_strict() {
        assert!(
            calculate_safety_strict(&TEST_DATA[0]),
            "Expected {:?} to be safe",
            TEST_DATA[0]
        );
        assert!(
            !calculate_safety_strict(&TEST_DATA[1]),
            "Expected {:?} to be unsafe",
            TEST_DATA[1]
        );
        assert!(
            !calculate_safety_strict(&TEST_DATA[2]),
            "Expected {:?} to be unsafe",
            TEST_DATA[2]
        );
        assert!(
            !calculate_safety_strict(&TEST_DATA[3]),
            "Expected {:?} to be unsafe",
            TEST_DATA[3]
        );
        assert!(
            !calculate_safety_strict(&TEST_DATA[4]),
            "Expected {:?} to be unsafe",
            TEST_DATA[4]
        );
        assert!(
            calculate_safety_strict(&TEST_DATA[5]),
            "Expected {:?} to be safe",
            TEST_DATA[5]
        );
    }

    #[test]
    fn test_safety_ish() {
        assert!(
            check_safety_ish(&TEST_DATA[0]),
            "Expected {:?} to be safe-ish",
            TEST_DATA[0]
        );
        assert!(
            !check_safety_ish(&TEST_DATA[1]),
            "Expected {:?} to be unsafe even after removals",
            TEST_DATA[1]
        );
        assert!(
            !check_safety_ish(&TEST_DATA[2]),
            "Expected {:?} to be unsafe even after removals",
            TEST_DATA[2]
        );
        assert!(
            check_safety_ish(&TEST_DATA[3]),
            "Expected {:?} to be safe-ish",
            TEST_DATA[3]
        );
        assert!(
            check_safety_ish(&TEST_DATA[4]),
            "Expected {:?} to be safe-ish",
            TEST_DATA[4]
        );
        assert!(
            check_safety_ish(&TEST_DATA[5]),
            "Expected {:?} to be safe-ish",
            TEST_DATA[5]
        );
    }
}
