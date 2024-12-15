use colored::Colorize;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
enum Move {
    North,
    South,
    East,
    West,
}

impl FromStr for Move {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "^" => Ok(Move::North),
            "v" => Ok(Move::South),
            ">" => Ok(Move::East),
            "<" => Ok(Move::West),
            _ => Err(format!("Invalid move: {}", s)),
        }
    }
}

struct Warehouse {
    width: usize,
    height: usize,
    boxes: Vec<(usize, usize)>,
    walls: Vec<(usize, usize)>,
    robot_pos: (usize, usize),
}

impl Warehouse {
    fn from_file(filename: &str) -> io::Result<Warehouse> {
        let path = Path::new(filename);
        let file = File::open(&path)?;
        let reader = io::BufReader::new(file);
        let mut boxes = Vec::new();
        let mut walls = Vec::new();
        let mut robot_pos = (0, 0);
        let mut width = 0;
        let mut height = 0;

        for (y, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            width = line.len();
            height = y + 1;
            for (x, ch) in line.chars().enumerate() {
                match ch {
                    '#' => walls.push((x, y)),
                    'O' => boxes.push((x, y)),
                    '@' => robot_pos = (x, y),
                    _ => (),
                }
            }
        }
        Ok(Warehouse {
            width,
            height,
            boxes,
            walls,
            robot_pos,
        })
    }

    fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x, y);
                if self.walls.contains(&pos) {
                    print!("{}", "#".bright_black());
                } else if pos == self.robot_pos {
                    print!("{}", "@".bright_magenta());
                } else if self.boxes.contains(&pos) {
                    print!("{}", "0".bright_cyan());
                } else {
                    print!("{}", ".".white().dimmed());
                }
            }
            println!();
        }
    }

    fn parse_moves(filename: &str) -> io::Result<Vec<Move>> {
        let path = Path::new(filename);
        let content = std::fs::read_to_string(&path)?;
        let moves: Result<Vec<Move>, String> = content
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| Move::from_str(&c.to_string()))
            .collect();
        moves.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn try_move(&self, direction: &Move) -> Result<Vec<((usize, usize), (usize, usize))>, String> {
        let (x, y) = self.robot_pos;
        let new_pos = match direction {
            Move::North => (x, y.checked_sub(1).ok_or("Out of bounds north")?),
            Move::South => {
                let new_y = y.checked_add(1).ok_or("Out of bounds south")?;
                (x, new_y)
            }
            Move::East => {
                let new_x = x.checked_add(1).ok_or("Out of bounds east")?;
                (new_x, y)
            }
            Move::West => (x.checked_sub(1).ok_or("Out of bounds west")?, y),
        };

        if new_pos.0 >= self.width || new_pos.1 >= self.height {
            return Err("Position outside warehouse bounds".to_string());
        }

        if self.walls.contains(&new_pos) {
            return Err("Cannot move into wall".to_string());
        }

        if self.boxes.contains(&new_pos) {
            let mut box_moves = Vec::new();
            let mut current_pos = new_pos;

            loop {
                let next_pos = match direction {
                    Move::North => (
                        current_pos.0,
                        current_pos
                            .1
                            .checked_sub(1)
                            .ok_or("Box out of bounds north")?,
                    ),
                    Move::South => {
                        let new_y = current_pos
                            .1
                            .checked_add(1)
                            .ok_or("Box out of bounds south")?;
                        (current_pos.0, new_y)
                    }
                    Move::East => {
                        let new_x = current_pos
                            .0
                            .checked_add(1)
                            .ok_or("Box out of bounds east")?;
                        (new_x, current_pos.1)
                    }
                    Move::West => (
                        current_pos
                            .0
                            .checked_sub(1)
                            .ok_or("Box out of bounds west")?,
                        current_pos.1,
                    ),
                };

                if next_pos.0 >= self.width || next_pos.1 >= self.height {
                    return Err("Box position outside warehouse bounds".to_string());
                }
                if self.walls.contains(&next_pos) {
                    return Err("Cannot push boxes into wall".to_string());
                }

                box_moves.push((current_pos, next_pos));

                match direction {
                    Move::North | Move::South => {
                        if !self.boxes.contains(&next_pos) || next_pos.0 != current_pos.0 {
                            break;
                        }
                    }
                    Move::East | Move::West => {
                        if !self.boxes.contains(&next_pos) || next_pos.1 != current_pos.1 {
                            break;
                        }
                    }
                }

                current_pos = next_pos;
            }

            Ok(box_moves)
        } else {
            Ok(Vec::new())
        }
    }

    fn step(&mut self, direction: &Move) -> Result<(), String> {
        let box_moves = self.try_move(direction)?;

        for (old_pos, new_pos) in box_moves.iter().rev() {
            self.boxes.retain(|&pos| pos != *old_pos);
            self.boxes.push(*new_pos);
        }

        self.robot_pos = match direction {
            Move::North => (self.robot_pos.0, self.robot_pos.1 - 1),
            Move::South => (self.robot_pos.0, self.robot_pos.1 + 1),
            Move::East => (self.robot_pos.0 + 1, self.robot_pos.1),
            Move::West => (self.robot_pos.0 - 1, self.robot_pos.1),
        };

        Ok(())
    }

    fn calculate_score(&self) -> usize {
        self.boxes.iter().map(|(x, y)| 100 * y + x).sum()
    }
}

fn main() {
    let mut warehouse = Warehouse::from_file("map.txt").unwrap();
    let moves = Warehouse::parse_moves("input.txt").unwrap();
    for (i, movement) in moves.iter().enumerate() {
        match warehouse.step(movement) {
            Ok(_) => {
                println!("Move {}", i + 1);
            }
            Err(e) => {
                println!("Move {} failed: {}", i + 1, e);
            }
        }
    }
    let score = warehouse.calculate_score();
    println!("Final score: {}", score);
}
