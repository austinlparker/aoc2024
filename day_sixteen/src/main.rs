use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::{BinaryHeap, HashMap};
use std::fs::read_to_string;

#[derive(Debug)]
struct Maze {
    grid: Vec<Vec<Cell>>,
    start: (i32, i32),
    end: (i32, i32),
}

#[derive(Debug, PartialEq, Clone)]
enum Cell {
    Wall,
    Path,
    Start,
    End,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Node {
    position: (i32, i32),
    direction: Direction,
    f_score: i32,
    g_score: i32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn turn_cost(&self, new_direction: Direction) -> i32 {
        if *self == new_direction {
            0
        } else {
            let directions = [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ];
            let current_idx = directions.iter().position(|&d| d == *self).unwrap();
            let new_idx = directions.iter().position(|&d| d == new_direction).unwrap();

            let diff = (new_idx as i32 - current_idx as i32).abs();
            let turns = std::cmp::min(diff, 4 - diff);
            turns * 1000
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Maze {
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = read_to_string(path)?;
        let mut grid = Vec::new();
        let mut start = None;
        let mut end = None;

        for (row, line) in contents.lines().enumerate() {
            let mut row_cells = Vec::new();
            for (col, ch) in line.chars().enumerate() {
                let cell = match ch {
                    '#' => Cell::Wall,
                    '.' => Cell::Path,
                    'S' => {
                        start = Some((row as i32, col as i32));
                        Cell::Start
                    }
                    'E' => {
                        end = Some((row as i32, col as i32));
                        Cell::End
                    }
                    _ => return Err("Invalid character in maze".into()),
                };
                row_cells.push(cell);
            }
            grid.push(row_cells);
        }

        Ok(Maze {
            grid,
            start: start.ok_or("No start found")?,
            end: end.ok_or("No end found")?,
        })
    }

    fn is_valid_position(&self, pos: (i32, i32)) -> bool {
        if pos.0 < 0 || pos.1 < 0 {
            return false;
        }
        let row = pos.0 as usize;
        let col = pos.1 as usize;

        row < self.grid.len() && col < self.grid[0].len() && self.grid[row][col] != Cell::Wall
    }

    fn find_optimal_path(&self, start_direction: Direction) -> Option<(Vec<(i32, i32)>, i32)> {
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<((i32, i32), Direction), ((i32, i32), Direction)> =
            HashMap::new();
        let mut g_scores: HashMap<((i32, i32), Direction), i32> = HashMap::new();

        let start_node = Node {
            position: self.start,
            direction: start_direction,
            f_score: manhattan_distance(self.start, self.end),
            g_score: 0,
        };

        g_scores.insert((self.start, start_direction), 0);
        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            if current.position == self.end {
                return Some((
                    self.reconstruct_path(&came_from, (current.position, current.direction)),
                    current.g_score,
                ));
            }

            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let next_pos = (current.position.0 + dx, current.position.1 + dy);

                if !self.is_valid_position(next_pos) {
                    continue;
                }

                let new_direction = match (*dx, *dy) {
                    (0, 1) => Direction::East,
                    (1, 0) => Direction::South,
                    (0, -1) => Direction::West,
                    (-1, 0) => Direction::North,
                    _ => unreachable!(),
                };

                let turn_cost = current.direction.turn_cost(new_direction);
                let movement_cost = 1;
                let total_move_cost = turn_cost + movement_cost;

                let tentative_g_score =
                    g_scores[&(current.position, current.direction)] + total_move_cost;

                if !g_scores.contains_key(&(next_pos, new_direction))
                    || tentative_g_score < g_scores[&(next_pos, new_direction)]
                {
                    came_from.insert(
                        (next_pos, new_direction),
                        (current.position, current.direction),
                    );
                    g_scores.insert((next_pos, new_direction), tentative_g_score);

                    open_set.push(Node {
                        position: next_pos,
                        direction: new_direction,
                        g_score: tentative_g_score,
                        f_score: tentative_g_score + manhattan_distance(next_pos, self.end),
                    });
                }
            }
        }

        None
    }

    fn find_all_optimal_paths(&self, start_direction: Direction) -> Vec<Vec<(i32, i32)>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<((i32, i32), Direction), ((i32, i32), Direction)> =
            HashMap::new();
        let mut g_scores: HashMap<((i32, i32), Direction), i32> = HashMap::new();
        let mut optimal_paths = Vec::new();
        let mut min_cost = std::i32::MAX;

        let start_node = Node {
            position: self.start,
            direction: start_direction,
            f_score: manhattan_distance(self.start, self.end),
            g_score: 0,
        };

        g_scores.insert((self.start, start_direction), 0);
        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            // If we've found a path and this one is more expensive, we can stop
            if !optimal_paths.is_empty() && current.g_score > min_cost {
                break;
            }

            if current.position == self.end {
                let path = self.reconstruct_path(&came_from, (current.position, current.direction));
                if optimal_paths.is_empty() || current.g_score == min_cost {
                    min_cost = current.g_score;
                    optimal_paths.push(path);
                }
                continue; // Continue searching for other paths of same cost
            }

            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let next_pos = (current.position.0 + dx, current.position.1 + dy);

                if !self.is_valid_position(next_pos) {
                    continue;
                }

                let new_direction = match (*dx, *dy) {
                    (0, 1) => Direction::East,
                    (1, 0) => Direction::South,
                    (0, -1) => Direction::West,
                    (-1, 0) => Direction::North,
                    _ => unreachable!(),
                };

                let turn_cost = current.direction.turn_cost(new_direction);
                let movement_cost = 1;
                let total_move_cost = turn_cost + movement_cost;

                let tentative_g_score =
                    g_scores[&(current.position, current.direction)] + total_move_cost;

                if !g_scores.contains_key(&(next_pos, new_direction))
                    || tentative_g_score < g_scores[&(next_pos, new_direction)]
                {
                    came_from.insert(
                        (next_pos, new_direction),
                        (current.position, current.direction),
                    );
                    g_scores.insert((next_pos, new_direction), tentative_g_score);

                    open_set.push(Node {
                        position: next_pos,
                        direction: new_direction,
                        g_score: tentative_g_score,
                        f_score: tentative_g_score + manhattan_distance(next_pos, self.end),
                    });
                }
            }
        }

        optimal_paths
    }

    fn reconstruct_path(
        &self,
        came_from: &HashMap<((i32, i32), Direction), ((i32, i32), Direction)>,
        mut current: ((i32, i32), Direction),
    ) -> Vec<(i32, i32)> {
        let mut path = vec![current.0];
        while let Some(&prev) = came_from.get(&current) {
            path.push(prev.0);
            current = prev;
        }
        path.reverse();
        path
    }

    fn visualize_path(&self, path: &[(i32, i32)]) -> String {
        let mut result = String::new();
        let path_set: std::collections::HashSet<_> = path.iter().collect();

        for (row, grid_row) in self.grid.iter().enumerate() {
            for (col, cell) in grid_row.iter().enumerate() {
                let char = if path_set.contains(&(row as i32, col as i32)) {
                    'O'
                } else {
                    match cell {
                        Cell::Wall => '#',
                        Cell::Path => '.',
                        Cell::Start => 'S',
                        Cell::End => 'E',
                    }
                };
                result.push(char);
            }
            result.push('\n');
        }
        result
    }

    fn count_adjacent_tiles(&self, path: &[(i32, i32)]) -> usize {
        let path_set: std::collections::HashSet<_> = path.iter().cloned().collect();
        let mut adjacent_set = std::collections::HashSet::new();
        for &(row, col) in path {
            for (dx, dy) in &[(-1, 0), (0, -1), (0, 1), (1, 0)] {
                let new_row = row + dx;
                let new_col = col + dy;

                if new_row >= 0
                    && new_row < self.grid.len() as i32
                    && new_col >= 0
                    && new_col < self.grid[0].len() as i32
                {
                    let pos = (new_row, new_col);
                    if self.grid[new_row as usize][new_col as usize] != Cell::Wall
                        && !path_set.contains(&pos)
                    {
                        adjacent_set.insert(pos);
                    }
                }
            }
        }
        adjacent_set.len()
    }
}

fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maze = Maze::from_file("input.txt")?;

    let initial_direction = Direction::East;
    let directions = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    let mut best_path = None;
    let mut best_cost = std::i32::MAX;

    for &try_direction in &directions {
        let initial_turn_cost = initial_direction.turn_cost(try_direction);

        if let Some((path, path_cost)) = maze.find_optimal_path(try_direction) {
            let total_cost = initial_turn_cost + path_cost;
            println!(
                "Trying direction: {:?}, Turn cost: {}, Path cost: {}, Total: {}",
                try_direction, initial_turn_cost, path_cost, total_cost
            );

            if total_cost < best_cost {
                best_cost = total_cost;
                best_path = Some(path);
            }
        }
    }

    if let Some(path) = best_path {
        println!("\nBest path found with cost {}!", best_cost);
        println!("Number of tiles in path: {}", path.len());
        println!("\nPath visualization:");
        println!("{}", maze.visualize_path(&path));
    } else {
        println!("No path found!");
    }

    Ok(())
}
