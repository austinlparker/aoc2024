use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct FrequencyMap {
    height: i32,
    width: i32,
    antennas: HashMap<String, Vec<(i32, i32)>>,
    antinode_counter: i32,
}

impl FrequencyMap {
    pub fn new(height: i32, width: i32, antennas: HashMap<String, Vec<(i32, i32)>>) -> Self {
        Self {
            height,
            width,
            antennas,
            antinode_counter: 0,
        }
    }

    fn is_in_bounds(&self, point: (i32, i32)) -> bool {
        point.0 >= 0 && point.0 < self.width && point.1 >= 0 && point.1 < self.height
    }

    fn calculate_vector(&self, start: (i32, i32), end: (i32, i32)) -> (i32, i32) {
        (end.0 - start.0, end.1 - start.1)
    }

    fn is_collinear(&self, p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)) -> bool {
        (p2.1 - p1.1) * (p3.0 - p1.0) == (p3.1 - p1.1) * (p2.0 - p1.0)
    }

    pub fn count_collinear_antinodes(&mut self) -> i32 {
        let mut antinode_positions = HashSet::new();
        for (_, positions) in &self.antennas {
            if positions.len() > 1 {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let point = (x, y);
                        let mut collinear_count = 0;
                        for i in 0..positions.len() {
                            for j in i + 1..positions.len() {
                                if self.is_collinear(positions[i], positions[j], point) {
                                    collinear_count += 1;
                                    break;
                                }
                            }
                        }

                        if collinear_count > 0 {
                            antinode_positions.insert(point);
                        }
                    }
                }
            }
        }

        antinode_positions.len() as i32
    }

    pub fn count_antinodes(&mut self) {
        let mut antinode_positions = HashSet::new();

        for positions in self.antennas.values() {
            for i in 0..positions.len() {
                for j in i + 1..positions.len() {
                    let start = positions[i];
                    let end = positions[j];
                    let vector = self.calculate_vector(start, end);
                    let antinode_before = (start.0 - vector.0, start.1 - vector.1);
                    let antinode_after = (end.0 + vector.0, end.1 + vector.1);

                    if self.is_in_bounds(antinode_before) {
                        antinode_positions.insert(antinode_before);
                    }
                    if self.is_in_bounds(antinode_after) {
                        antinode_positions.insert(antinode_after);
                    }
                }
            }
        }

        self.antinode_counter = antinode_positions.len() as i32;
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let mut map = parse_frequency_map(&lines);
    map.count_antinodes();
    println!("Antinode count: {}", map.antinode_counter);
    let cnc = map.count_collinear_antinodes();
    println!("Collinear antinode count: {}", cnc);

    Ok(())
}

fn parse_frequency_map(lines: &[String]) -> FrequencyMap {
    let mut antennas = HashMap::new();
    let height = lines.len() as i32;
    let width = lines.first().map_or(0, |line| line.len() as i32);

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch != '.' {
                antennas
                    .entry(ch.to_string())
                    .or_insert_with(Vec::new)
                    .push((x as i32, y as i32));
            }
        }
    }

    FrequencyMap::new(height, width, antennas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collinear_antinodes() {
        let input = vec![
            "............".to_string(),
            "........0...".to_string(),
            ".....0......".to_string(),
            ".......0....".to_string(),
            "....0.......".to_string(),
            "......A.....".to_string(),
            "............".to_string(),
            "............".to_string(),
            "........A...".to_string(),
            ".........A..".to_string(),
            "............".to_string(),
            "............".to_string(),
        ];

        let mut map = parse_frequency_map(&input);

        println!("Initial antenna positions:");
        for (symbol, positions) in &map.antennas {
            println!("{}: {:?}", symbol, positions);
        }

        let collinear_count = map.count_collinear_antinodes();

        let mut antinode_positions = HashSet::new();
        for (_, positions) in &map.antennas {
            if positions.len() > 1 {
                for y in 0..map.height {
                    for x in 0..map.width {
                        let point = (x, y);
                        for i in 0..positions.len() {
                            for j in i + 1..positions.len() {
                                if map.is_collinear(positions[i], positions[j], point) {
                                    antinode_positions.insert(point);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("Found {} collinear antinodes", collinear_count);
        assert_eq!(collinear_count, 34);
    }

    #[test]
    fn test_antinode_counting() {
        let input = vec![
            "............".to_string(),
            "........0...".to_string(),
            ".....0......".to_string(),
            ".......0....".to_string(),
            "....0.......".to_string(),
            "......A.....".to_string(),
            "............".to_string(),
            "............".to_string(),
            "........A...".to_string(),
            ".........A..".to_string(),
            "............".to_string(),
            "............".to_string(),
        ];

        let mut map = parse_frequency_map(&input);

        for positions in map.antennas.values() {
            for i in 0..positions.len() {
                for j in i + 1..positions.len() {
                    let start = positions[i];
                    let end = positions[j];
                    let vector = map.calculate_vector(start, end);
                    let antinode_before = (start.0 - vector.0, start.1 - vector.1);
                    let antinode_after = (end.0 + vector.0, end.1 + vector.1);
                    println!("Ray from {:?} to {:?}", start, end);
                    println!(
                        "  Potential antinode before: {:?} (in bounds: {})",
                        antinode_before,
                        map.is_in_bounds(antinode_before)
                    );
                    println!(
                        "  Potential antinode after: {:?} (in bounds: {})",
                        antinode_after,
                        map.is_in_bounds(antinode_after)
                    );
                }
            }
        }

        map.count_antinodes();
        assert_eq!(
            map.antinode_counter, 14,
            "Expected 14 antinodes but got {}",
            map.antinode_counter
        );
    }
}
