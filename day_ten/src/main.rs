use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct TopoMap {
    height: i32,
    width: i32,
    trailheads: Vec<(i32, i32)>,
    grid: Vec<Vec<u8>>,
}

impl TopoMap {
    const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    #[allow(dead_code)] //used in testing
    fn new_from_string(str: Vec<String>) -> TopoMap {
        let mut trailheads = Vec::new();
        let mut grid = Vec::new();
        let height = str.len() as i32;
        let width = str[0].len() as i32;

        for (i, line) in str.iter().enumerate() {
            let mut row = Vec::new();
            for (j, c) in line.chars().enumerate() {
                let value = c.to_digit(10).unwrap() as u8;
                row.push(value);
                if c == '0' {
                    trailheads.push((j as i32, i as i32));
                }
            }
            grid.push(row);
        }
        TopoMap {
            height,
            width,
            trailheads,
            grid,
        }
    }
    fn new_from_file(file: File) -> TopoMap {
        let mut trailheads = Vec::new();
        let mut grid = Vec::new();
        let mut width = 0;

        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            let mut row = Vec::new();
            let line = line.unwrap();
            if width == 0 {
                width = line.len() as i32;
            }
            for (j, c) in line.chars().enumerate() {
                let value = c.to_digit(10).unwrap() as u8;
                row.push(value);
                if c == '0' {
                    trailheads.push((j as i32, i as i32));
                }
            }
            grid.push(row);
        }

        let height = grid.len() as i32;

        TopoMap {
            height,
            width,
            trailheads,
            grid,
        }
    }
    fn get_value(&self, x: i32, y: i32) -> Option<u8> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(self.grid[y as usize][x as usize])
        } else {
            None
        }
    }
    fn find_paths_from_start(&self, start: (i32, i32), valid_paths: &mut Vec<Vec<(i32, i32)>>) {
        let mut queue = VecDeque::new();
        queue.push_back((start, vec![start]));
        let mut visited = HashSet::new();
        visited.insert(start);

        while let Some((current_pos, current_path)) = queue.pop_front() {
            let current_value = self.get_value(current_pos.0, current_pos.1).unwrap();

            if current_value == 9 {
                valid_paths.push(current_path);
                continue;
            }

            for &(dx, dy) in &Self::DIRECTIONS {
                let next_pos = (current_pos.0 + dx, current_pos.1 + dy);

                if visited.contains(&next_pos) {
                    continue;
                }

                if let Some(next_value) = self.get_value(next_pos.0, next_pos.1) {
                    if next_value == current_value + 1 {
                        visited.insert(next_pos);
                        let mut new_path = current_path.clone();
                        new_path.push(next_pos);
                        queue.push_back((next_pos, new_path));
                    }
                }
            }
        }
    }
    fn find_paths_from_start_all(&self, start: (i32, i32), valid_paths: &mut Vec<Vec<(i32, i32)>>) {
        let mut queue = vec![(start, vec![start])];

        while let Some((current_pos, current_path)) = queue.pop() {
            let current_value = self.get_value(current_pos.0, current_pos.1).unwrap();

            if current_value == 9 {
                valid_paths.push(current_path);
                continue;
            }

            for &(dx, dy) in &Self::DIRECTIONS {
                let next_pos = (current_pos.0 + dx, current_pos.1 + dy);

                if let Some(next_value) = self.get_value(next_pos.0, next_pos.1) {
                    if next_value == current_value + 1 {
                        let mut new_path = current_path.clone();
                        new_path.push(next_pos);
                        queue.push((next_pos, new_path));
                    }
                }
            }
        }
    }

    fn calculate_scores(&self) -> Vec<((i32, i32), usize)> {
        self.trailheads
            .iter()
            .map(|&trailhead| {
                let mut paths = Vec::new();
                self.find_paths_from_start(trailhead, &mut paths);
                (trailhead, paths.len())
            })
            .collect()
    }
    fn calculate_all_scores(&self) -> Vec<((i32, i32), usize)> {
        self.trailheads
            .iter()
            .map(|&trailhead| {
                let mut paths = Vec::new();
                self.find_paths_from_start_all(trailhead, &mut paths);
                (trailhead, paths.len())
            })
            .collect()
    }
}

fn main() {
    let input = File::open("input.txt").expect("Could not open file");
    let map = TopoMap::new_from_file(input);
    println!(
        "Loaded map: {}x{}, Trailheads: {}",
        map.width,
        map.height,
        map.trailheads.len()
    );
    let scores = map.calculate_scores();
    let total: usize = scores.iter().map(|(_, score)| score).sum();
    println!("Total score (case 1) {}", total);

    let all_scores = map.calculate_all_scores();
    let all_total: usize = all_scores.iter().map(|(_, score)| score).sum();
    println!("Total score (case 2) {}", all_total);
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_test_map() -> TopoMap {
        let input = vec![
            String::from("89010123"),
            String::from("78121874"),
            String::from("87430965"),
            String::from("96549874"),
            String::from("45678903"),
            String::from("32019012"),
            String::from("01329801"),
            String::from("10456732"),
        ];
        TopoMap::new_from_string(input)
    }

    #[test]
    fn test_map_creation() {
        let map = create_test_map();
        assert_eq!(map.width, 8);
        assert_eq!(map.height, 8);
        assert_eq!(map.trailheads.len(), 9);
    }

    #[test]
    fn test_get_value() {
        let map = create_test_map();
        assert_eq!(map.get_value(0, 0), Some(8));
        assert_eq!(map.get_value(2, 1), Some(1));
        assert_eq!(map.get_value(-1, 0), None);
        assert_eq!(map.get_value(8, 0), None);
    }

    #[test]
    fn test_single_path() {
        let map = create_test_map();
        let mut paths = Vec::new();
        map.find_paths_from_start((3, 0), &mut paths);
        assert!(paths.len() > 0);
    }

    #[test]
    fn test_trailhead_scores() {
        let map = create_test_map();
        let scores = map.calculate_scores();
        assert_eq!(scores.len(), map.trailheads.len());

        let expected_scores = vec![5, 6, 5, 3, 1, 3, 5, 3, 5];
        let actual_scores: Vec<usize> = scores.iter().map(|(_, score)| *score).collect();

        assert_eq!(actual_scores, expected_scores);

        let all_scores = map.calculate_all_scores();
        let expected_all_scores = vec![20, 24, 10, 4, 1, 4, 5, 8, 5];
        let actual_all_scores: Vec<usize> = all_scores.iter().map(|(_, score)| *score).collect();
        assert_eq!(actual_all_scores, expected_all_scores);
    }

    #[test]
    fn test_total_score() {
        let map = create_test_map();
        let scores = map.calculate_scores();
        let all_scores = map.calculate_all_scores();
        let total: usize = scores.iter().map(|(_, score)| score).sum();
        let all_total: usize = all_scores.iter().map(|(_, score)| score).sum();
        assert!(total == 36);
        assert!(all_total == 81);
    }
}
