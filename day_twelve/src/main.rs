use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Map {
    width: usize,
    height: usize,
    data: Vec<String>,
    regions: HashMap<char, Vec<Vec<(usize, usize)>>>,
}

impl Map {
    fn detect_regions(&mut self) {
        let mut visited = vec![vec![false; self.width]; self.height];
        self.regions.clear();
        for y in 0..self.height {
            for x in 0..self.width {
                if !visited[y][x] {
                    let c = self.data[y].chars().nth(x).unwrap();
                    let mut region = Vec::new();
                    let mut queue = Vec::new();
                    queue.push((x, y));

                    while let Some((c_x, c_y)) = queue.pop() {
                        if visited[c_y][c_x] {
                            continue;
                        }
                        if self.data[c_y].chars().nth(c_x).unwrap() != c {
                            continue;
                        }
                        visited[c_y][c_x] = true;
                        region.push((c_x, c_y));
                        for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                            let n_x = c_x as i32 + dx;
                            let n_y = c_y as i32 + dy;
                            if n_x >= 0
                                && n_x < self.width as i32
                                && n_y >= 0
                                && n_y < self.height as i32
                            {
                                queue.push((n_x as usize, n_y as usize));
                            }
                        }
                    }
                    self.regions.entry(c).or_insert(Vec::new()).push(region);
                }
            }
        }
    }

    fn calculate_price(&self) -> usize {
        let mut total = 0;
        for regions in self.regions.values() {
            for region in regions {
                let area = region.len();
                let perimeter = self.calculate_perimeter(region);
                total += area * perimeter;
            }
        }
        total
    }

    fn calculate_perimeter(&self, region: &Vec<(usize, usize)>) -> usize {
        let mut perimeter = 0;
        for &(x, y) in region {
            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;
                if new_x < 0
                    || new_x >= self.width as i32
                    || new_y < 0
                    || new_y >= self.height as i32
                {
                    perimeter += 1;
                    continue;
                }
                let current_char = self.data[y].chars().nth(x).unwrap();
                let adjacent_char = self.data[new_y as usize]
                    .chars()
                    .nth(new_x as usize)
                    .unwrap();
                if current_char != adjacent_char {
                    perimeter += 1;
                }
            }
        }

        perimeter
    }

    fn calculate_sides(&self) -> usize {
        let mut total_price = 0;
        for regions in self.regions.values() {
            for region in regions {
                let num_cells = region.len();
                let num_sides = self.count_region_sides(region);
                total_price += num_cells * num_sides;
            }
        }
        total_price
    }

    fn count_region_sides(&self, region: &Vec<(usize, usize)>) -> usize {
        let region_coords: std::collections::HashSet<_> = region.iter().cloned().collect();
        let mut sides = 0;

        let is_in_region = |x: i32, y: i32| -> bool {
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                return false;
            }
            region_coords.contains(&(x as usize, y as usize))
        };

        for &(x, y) in region {
            let x = x as i32;
            let y = y as i32;

            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;

                if !is_in_region(nx, ny) {
                    let mut is_new_side = true;

                    if *dx == 0 {
                        // Vertical edge
                        if is_in_region(x - 1, y) && !is_in_region(x - 1, ny) {
                            is_new_side = false;
                        }
                    } else {
                        // Horizontal edge
                        if is_in_region(x, y - 1) && !is_in_region(nx, y - 1) {
                            is_new_side = false;
                        }
                    }

                    if is_new_side {
                        sides += 1;
                    }
                }
            }
        }
        sides
    }
}

trait MapSource {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_>;
}

impl MapSource for File {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_> {
        let reader = BufReader::new(self);
        Box::new(reader.lines().map(|l| l.unwrap()))
    }
}

impl MapSource for Vec<String> {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.clone().into_iter())
    }
}

fn main() {
    let file = File::open("input.txt").expect("Failed to read file");
    let mut map = load_map(&file);
    println!("Loaded map: {}x{}", map.height, map.width);
    println!("Regions: {:?}", map.regions.keys());
    map.detect_regions();
    let total = map.calculate_price();
    println!("Total price: {}", total);
    let total_sides = map.calculate_sides();
    println!("Total sides price: {}", total_sides);
}

fn load_map(source: &impl MapSource) -> Map {
    let mut width = 0;
    let mut height = 0;
    let mut data = Vec::new();
    let mut regions = HashMap::new();

    for line in source.lines() {
        for c in line.chars() {
            regions.entry(c).or_insert(Vec::new());
        }
        width = line.len();
        height += 1;
        data.push(line);
    }
    Map {
        width,
        height,
        data,
        regions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_map_with_ox_pattern() {
        let input = vec![
            "OOOOO".to_string(),
            "OXOXO".to_string(),
            "OOOOO".to_string(),
            "OXOXO".to_string(),
            "OOOOO".to_string(),
        ];

        let mut map = load_map(&input);

        assert_eq!(map.width, 5);
        assert_eq!(map.height, 5);
        map.detect_regions();
        assert_eq!(map.regions.len(), 2);
        assert_eq!(map.regions.get(&'O').unwrap().len(), 1);
        assert_eq!(map.regions.get(&'X').unwrap().len(), 4);
        let total = map.calculate_price();
        assert_eq!(total, 772);
    }

    #[test]
    fn test_load_map_with_complex_pattern() {
        let input = vec![
            "RRRRIICCFF".to_string(),
            "RRRRIICCCF".to_string(),
            "VVRRRCCFFF".to_string(),
            "VVRCCCJFFF".to_string(),
            "VVVVCJJCFE".to_string(),
            "VVIVCCJJEE".to_string(),
            "VVIIICJJEE".to_string(),
            "MIIIIIJJEE".to_string(),
            "MIIISIJEEE".to_string(),
            "MMMISSJEEE".to_string(),
        ];

        let mut map = load_map(&input);

        // Test dimensions
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        map.detect_regions();
        let total = map.calculate_price();
        assert_eq!(total, 1930);
        let total_sides = map.calculate_sides();
        assert_eq!(total_sides, 1206);
    }
}
