use colored::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::Error;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn turn_right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
        }
    }

    fn get_vector(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

struct LabMap {
    length: i32,
    width: i32,
    obstacles: Vec<(i32, i32)>,
    guard_history: Vec<(i32, i32)>,
    guard_pos: (i32, i32),
    guard_direction: Direction,
    direction_history: Vec<Direction>,
}

impl LabMap {
    pub fn new(file_name: File) -> Result<LabMap, Error> {
        let reader = BufReader::new(file_name);
        let mut obstacles = Vec::new();
        let mut guard_history = Vec::new();
        let mut length = 0;
        let mut width = 0;
        let mut guard_pos = (0, 0);

        for (y, line) in reader.lines().enumerate() {
            let line = line?;
            if y == 0 {
                width = line.len() as i32;
            }
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' => obstacles.push((x as i32, y as i32)),
                    '^' => guard_pos = (x as i32, y as i32),
                    _ => (),
                }
            }
            length = (y + 1) as i32;
        }

        Ok(LabMap {
            length,
            width,
            obstacles,
            guard_pos,
            guard_history,
            guard_direction: Direction::North,
        })
    }

    pub fn display(&self, viewport_size: i32) {
        print!("\x1B[2J\x1B[1;1H");
        let half_size = viewport_size / 2;
        let view_start_x = (self.guard_pos.0 - half_size).max(0);
        let view_start_y = (self.guard_pos.1 - half_size).max(0);
        let view_end_x = (self.guard_pos.0 + half_size).min(self.width - 1);
        let view_end_y = (self.guard_pos.1 + half_size).min(self.length - 1);
        let direction_symbol = match self.guard_direction {
            Direction::North => "↑",
            Direction::East => "→",
            Direction::South => "↓",
            Direction::West => "←",
        };
        println!(
            "Viewing: ({}, {}) to ({}, {})",
            view_start_x, view_start_y, view_end_x, view_end_y
        );
        println!("╔{}╗", "═".repeat((view_end_x - view_start_x + 1) as usize));
        for y in view_start_y..=view_end_y {
            print!("║");
            for x in view_start_x..=view_end_x {
                if self.guard_pos == (x, y) {
                    print!("{}", "G".bright_green().bold());
                } else if self.obstacles.contains(&(x, y)) {
                    print!("{}", "#".bright_red());
                } else if self.guard_history.contains(&(x, y)) {
                    print!("{}", ".".bright_yellow().dimmed());
                } else {
                    print!("{}", ".".bright_blue().dimmed());
                }
            }
            println!("║");
        }
        println!("╚{}╝", "═".repeat((view_end_x - view_start_x + 1) as usize));
        println!("Guard at: {:?} facing {}", self.guard_pos, direction_symbol);
    }

    fn is_in_bounds(&self, pos: (i32, i32)) -> bool {
        pos.0 >= 0 && pos.0 < self.width && pos.1 >= 0 && pos.1 < self.length
    }

    fn is_valid_position(&self, pos: (i32, i32)) -> bool {
        if self.is_in_bounds(pos) {
            !self.obstacles.contains(&pos)
        } else {
            true
        }
    }

    pub fn step(&mut self) -> bool {
        let (dx, dy) = self.guard_direction.get_vector();
        let next = (self.guard_pos.0 + dx, self.guard_pos.1 + dy);
        if self.is_valid_position(next) {
            self.guard_history.push(self.guard_pos);
            self.direction_history.push(self.guard_direction);
            self.guard_pos = next;
            self.is_in_bounds(self.guard_pos)
        } else {
            self.guard_direction = self.guard_direction.turn_right();
            true
        }
    }

    pub fn unique_positions_visited(&self) -> usize {
        self.guard_history.iter().collect::<HashSet<_>>().len()
    }
}

fn main() -> Result<(), Error> {
    let file = File::open("input.txt")?;
    let mut lab_map = LabMap::new(file)?;
    while lab_map.step() {
        lab_map.display(20);
        std::thread::sleep(std::time::Duration::from_millis(1));
        print!("\x1B[2J\x1B[1;1H");
    }
    lab_map.display(20);
    println!(
        "\nSimulation ended - Total steps: {}, unique postiiions visited: {}",
        lab_map.guard_history.len(),
        lab_map.unique_positions_visited()
    );

    Ok(())
}
