use std::fs::File;
use std::io::{BufRead, BufReader};

const A_TOKENS: usize = 3;
const B_TOKENS: usize = 1;
const BIG_OFFSET: usize = 10000000000000;

struct ClawGame {
    prize_location: (usize, usize),
    a_input: (usize, usize),
    b_input: (usize, usize),
}

#[derive(Debug, PartialEq)]
struct Solution {
    a_presses: usize,
    b_presses: usize,
}

trait GameSource {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_>;
}

impl GameSource for File {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_> {
        let reader = BufReader::new(self);
        Box::new(reader.lines().map(|l| l.unwrap()))
    }
}

impl GameSource for Vec<String> {
    fn lines(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.clone().into_iter())
    }
}

fn solve_games(games: &[ClawGame], offset: i64) -> (i64, i64) {
    // Part A
    let sum_a: i64 = games
        .iter()
        .map(|s| {
            let mut min_cost = i64::MAX;
            for i in 1..=100 {
                for j in 1..=100 {
                    if s.a_input.0 as i64 * i + s.b_input.0 as i64 * j == s.prize_location.0 as i64
                        && s.a_input.1 as i64 * i + s.b_input.1 as i64 * j
                            == s.prize_location.1 as i64
                    {
                        let cost = 3 * i + j;
                        if cost < min_cost {
                            min_cost = cost;
                        }
                        break;
                    }
                }
            }
            if min_cost != i64::MAX {
                min_cost
            } else {
                0
            }
        })
        .sum();

    // Part B
    let sum_b: i64 = games
        .iter()
        .map(|s| {
            let tx = s.prize_location.0 as i64 + offset;
            let ty = s.prize_location.1 as i64 + offset;
            let ax = s.a_input.0 as i64;
            let ay = s.a_input.1 as i64;
            let bx = s.b_input.0 as i64;
            let by = s.b_input.1 as i64;

            if tx % gcd(ax, bx) != 0 || ty % gcd(ay, by) != 0 {
                return 0;
            }

            let denom = by * ax - ay * bx;
            if denom == 0 {
                return 0;
            }

            let a = tx * by - ty * bx;
            if a % denom != 0 {
                return 0;
            }
            let a = a / denom;

            let b = ty * ax - tx * ay;
            if b % denom != 0 {
                return 0;
            }
            let b = b / denom;

            if a < 0 || b < 0 {
                return 0;
            }

            3 * a + b
        })
        .sum();

    (sum_a, sum_b)
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while a != b {
        if a > b {
            a -= b;
        } else {
            b -= a;
        }
    }
    a
}

fn load_games(source: &impl GameSource) -> Vec<ClawGame> {
    let mut games = Vec::new();
    let mut current_game = None;
    let mut line_count = 0;

    for line in source.lines() {
        match line_count % 4 {
            0 => {
                // Button A line
                let coords = parse_button_coords(&line, "Button A: ");
                current_game = Some(ClawGame {
                    prize_location: (0, 0),
                    a_input: coords,
                    b_input: (0, 0),
                });
            }
            1 => {
                // Button B line
                if let Some(game) = &mut current_game {
                    game.b_input = parse_button_coords(&line, "Button B: ");
                }
            }
            2 => {
                // Prize line
                if let Some(mut game) = current_game.take() {
                    game.prize_location = parse_prize_coords(&line);
                    games.push(game);
                }
            }
            3 => (), // Empty line
            _ => unreachable!(),
        }
        line_count += 1;
    }

    games
}

fn parse_button_coords(line: &str, prefix: &str) -> (usize, usize) {
    let coords = line.strip_prefix(prefix).unwrap();
    let (x, y) = coords.split_once(", ").unwrap();
    let x = x.strip_prefix("X+").unwrap().parse().unwrap();
    let y = y.strip_prefix("Y+").unwrap().parse().unwrap();
    (x, y)
}

fn parse_prize_coords(line: &str) -> (usize, usize) {
    let coords = line.strip_prefix("Prize: ").unwrap();
    let (x, y) = coords.split_once(", ").unwrap();
    let x = x.strip_prefix("X=").unwrap().parse().unwrap();
    let y = y.strip_prefix("Y=").unwrap().parse().unwrap();
    (x, y)
}

fn main() {
    let input = File::open("input.txt").unwrap();
    let games = load_games(&input);
    println!("Loaded {} games.", games.len());

    let (part_a, part_b) = solve_games(&games, BIG_OFFSET as i64);
    println!("Part A: {}", part_a);
    println!("Part B: {}", part_b);
}
