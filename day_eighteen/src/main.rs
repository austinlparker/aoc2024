use std::collections::{HashSet, VecDeque};
use std::fs::read_to_string;

struct Grid {
    height: usize,
    width: usize,
    cells: Vec<Vec<bool>>,
}

impl Grid {
    fn new(height: usize, width: usize) -> Self {
        Grid {
            height,
            width,
            cells: vec![vec![false; width]; height],
        }
    }
    fn set_coordinates(&mut self, coordinates: &[(usize, usize)]) {
        for &(x, y) in coordinates {
            if y < self.height && x < self.width {
                self.cells[y][x] = true;
            }
        }
    }
    fn find_path(&self) -> Option<Vec<(usize, usize)>> {
        let start = (0, 0);
        let end = (self.width - 1, self.height - 1);

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((start.0, start.1, vec![start]));
        visited.insert(start);

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        while let Some((x, y, path)) = queue.pop_front() {
            if (x, y) == end {
                return Some(path);
            }
            for (dx, dy) in directions.iter() {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;
                if new_x >= 0
                    && new_x < self.width as i32
                    && new_y >= 0
                    && new_y < self.height as i32
                {
                    let new_x = new_x as usize;
                    let new_y = new_y as usize;
                    if !visited.contains(&(new_x, new_y)) && !self.cells[new_y][new_x] {
                        let mut new_path = path.clone();
                        new_path.push((new_x, new_y));
                        queue.push_back((new_x, new_y, new_path));
                        visited.insert((new_x, new_y));
                    }
                }
            }
        }
        None
    }
}

fn main() {
    let input = read_to_string("input.txt").expect("Failed to read input file.");
    let all_coordinates: Vec<(usize, usize)> = input
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(',');
            let x = parts.next()?.parse().ok()?;
            let y = parts.next()?.parse().ok()?;
            Some((x, y))
        })
        .collect();

    // Part A
    let mut grid = Grid::new(71, 71);
    grid.set_coordinates(&all_coordinates[..1024]);
    if let Some(path) = grid.find_path() {
        println!("Part A - Steps needed: {}", path.len() - 1);
    }

    // Part B
    for i in 1024..all_coordinates.len() {
        let mut test_grid = Grid::new(71, 71);
        test_grid.set_coordinates(&all_coordinates[..=i]);

        if test_grid.find_path().is_none() {
            println!("Path becomes impossible after {} coordinates", i + 1);
            println!("First blocking coordinate: {:?}", all_coordinates[i]);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_solution() {
        let mut grid = Grid::new(7, 7);
        let coordinates = vec![
            (5, 4),
            (4, 2),
            (4, 5),
            (3, 0),
            (2, 1),
            (6, 3),
            (2, 4),
            (1, 5),
            (0, 6),
            (3, 3),
            (2, 6),
            (5, 1),
        ];

        grid.set_coordinates(&coordinates);
        println!("Test maze layout:");

        let path = grid.find_path().expect("Should find a path");
        println!("Path found: {:?}", path);
        assert_eq!(path.len() - 1, 22, "Path should take 22 steps");
    }
}
