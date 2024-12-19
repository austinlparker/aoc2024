use rayon::prelude::*;
use tracing::{debug, info, instrument};
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
enum OpCode {
    ADV = 0, // divide register A by combo value of operand ^2, truncate to int, write to A
    BXL = 1, // bitwise XOR of B and operand, store B
    BST = 2, // modulo 8 of operand, store in B
    JNZ = 3, // if A is 0, do nothing. else, jump to operand. do not increase PC.
    BXC = 4, // bitwise XOR of operand B and C, store B (ignore operand)
    OUT = 5, // modulo 8 of operand and write (separate multiple values by commas)
    BDV = 6, // like adv but store in B
    CDV = 7, // like adv but store in C
}

#[derive(Debug)]
struct Computer {
    pc: u8,
    register_a: u32,
    register_b: u32,
    register_c: u32,
    memory: Vec<u8>,
    output: Vec<u8>,
}

impl Computer {
    fn new() -> Self {
        Computer {
            pc: 0,
            register_a: 0,
            register_b: 0,
            register_c: 0,
            memory: Vec::new(),
            output: Vec::new(),
        }
    }

    fn load_program(&mut self, program: Vec<u8>) {
        self.memory = program;
    }

    fn set_register_a(&mut self, value: u32) {
        self.register_a = value;
    }

    #[instrument]
    fn execute(&mut self) {
        info!("Beginning execution");
        debug!(
            "Initial state: PC={}, A={}, B={}, C={}",
            self.pc, self.register_a, self.register_b, self.register_c
        );
        loop {
            if self.pc as usize >= self.memory.len() {
                info!("Program terminated: reached end of memory");
                break;
            }
            let instruction = self.memory[self.pc as usize];
            let operand = self.memory[(self.pc + 1) as usize];
            debug!(
                "Executing: PC={}, instruction={}, operand={}",
                self.pc, instruction, operand
            );

            let get_combo_value = |op: u8, _computer: &Computer| -> u32 {
                let value = match op {
                    0..=3 => op as u32,
                    4 => self.register_a,
                    5 => self.register_b,
                    6 => self.register_c,
                    _ => 0,
                };
                debug!("Combo value for operand {}: {}", op, value);
                value
            };

            match instruction {
                x if x == OpCode::ADV as u8 => {
                    let power = get_combo_value(operand, self);
                    let divisor = 1u32 << power;
                    debug!(
                        "ADV: A={} / 2^{} = {}",
                        self.register_a,
                        power,
                        self.register_a / divisor
                    );
                    self.register_a /= divisor;
                }
                x if x == OpCode::BXL as u8 => {
                    debug!(
                        "BXL: B={} XOR {} = {}",
                        self.register_b,
                        operand,
                        self.register_b ^ (operand as u32)
                    );
                    self.register_b ^= operand as u32;
                }
                x if x == OpCode::BST as u8 => {
                    let value = get_combo_value(operand, self);
                    let result = value % 8;
                    debug!("BST: {} % 8 = {}", value, result);
                    self.register_b = result;
                }
                x if x == OpCode::JNZ as u8 => {
                    debug!(
                        "JNZ: A={}, jumping to {} if non-zero",
                        self.register_a, operand
                    );
                    if self.register_a != 0 {
                        self.pc = operand;
                        continue;
                    }
                }
                x if x == OpCode::BXC as u8 => {
                    debug!(
                        "BXC: B={} XOR C={} = {}",
                        self.register_b,
                        self.register_c,
                        self.register_b ^ self.register_c
                    );
                    self.register_b ^= self.register_c;
                }
                x if x == OpCode::OUT as u8 => {
                    let value = get_combo_value(operand, self);
                    let output = (value % 8) as u8;
                    debug!("OUT: {} % 8 = {}", value, output);
                    self.output.push(output);
                }
                x if x == OpCode::BDV as u8 => {
                    let power = get_combo_value(operand, self);
                    let divisor = 1u32 << power;
                    debug!(
                        "BDV: A={} / 2^{} = {}",
                        self.register_a,
                        power,
                        self.register_a / divisor
                    );
                    self.register_b = self.register_a / divisor;
                }
                x if x == OpCode::CDV as u8 => {
                    let power = get_combo_value(operand, self);
                    let divisor = 1u32 << power;
                    debug!(
                        "CDV: A={} / 2^{} = {}",
                        self.register_a,
                        power,
                        self.register_a / divisor
                    );
                    self.register_c = self.register_a / divisor;
                }
                _ => {
                    info!("Program terminated: invalid instruction {}", instruction);
                    break;
                }
            }
            self.pc += 2;
            debug!(
                "After instruction: PC={}, A={}, B={}, C={}",
                self.pc, self.register_a, self.register_b, self.register_c
            );
        }
        info!("Program completed");
        debug!(
            "Final state: PC={}, A={}, B={}, C={}",
            self.pc, self.register_a, self.register_b, self.register_c
        );
    }
}

fn find_matching_output(program: &[u8]) -> Vec<u32> {
    (0..1_000_000_000)
        .into_par_iter()
        .filter_map(|a| {
            let mut computer = Computer::new();
            computer.load_program(program.to_vec());
            computer.set_register_a(a);
            computer.execute();
            if computer.output == program {
                Some(a)
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let program = vec![2, 4, 1, 5, 7, 5, 1, 6, 4, 1, 5, 5, 0, 3, 3, 0];

    let mut computer = Computer::new();
    computer.load_program(program);
    computer.set_register_a(60589763);
    computer.execute();
    println!(
        "{}",
        computer
            .output
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("Searching for matching outputs...");
    let program_again = vec![2, 4, 1, 5, 7, 5, 1, 6, 4, 1, 5, 5, 0, 3, 3, 0];
    let matches = find_matching_output(&program_again);
    for x in matches {
        println!("{}", x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute() {
        let program = vec![0, 1, 5, 4, 3, 0];
        let mut computer = Computer::new();
        computer.load_program(program);
        computer.set_register_a(729);
        computer.execute();
        assert_eq!(computer.output, vec![4, 6, 3, 5, 6, 3, 5, 2, 1, 0]);
    }
}
