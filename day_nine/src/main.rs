use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct FileNode {
    id: usize,
    size: i32,
    free_size: i32,
}

#[derive(Debug, Clone)]
struct BlockPosition {
    id: usize,
    start: usize,
    length: usize,
}

impl fmt::Display for FileNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bars = "|".repeat(self.size as usize);
        let dots = ".".repeat(self.free_size as usize);
        write!(f, "{}{}", bars, dots)
    }
}

fn find_first_free_space(positions: &[BlockPosition], total_length: usize) -> usize {
    if positions.is_empty() {
        return 0;
    }

    let mut occupied_spaces: Vec<bool> = vec![false; total_length];
    for pos in positions {
        for i in pos.start..(pos.start + pos.length) {
            occupied_spaces[i] = true;
        }
    }

    for i in 0..total_length {
        if !occupied_spaces[i] {
            return i;
        }
    }
    total_length
}

fn compact_blocks(original_nodes: HashMap<usize, FileNode>) -> Vec<BlockPosition> {
    let mut positions = Vec::new();
    let mut current_pos = 0;

    // Process blocks in order (0 to 9)
    for id in 0..=9 {
        if let Some(node) = original_nodes.get(&id) {
            positions.push(BlockPosition {
                id,
                start: current_pos,
                length: node.size as usize,
            });
            current_pos += node.size as usize;
        }
    }

    // Now we have the initial layout, let's compact from right to left
    let mut compacted: Vec<BlockPosition> = Vec::new();
    for i in (0..positions.len()).rev() {
        let block = &positions[i];

        // Find leftmost available space
        let mut start_pos = 0;
        'outer: loop {
            // Check if this position is available
            for existing in &compacted {
                if (start_pos + block.length > existing.start
                    && start_pos < existing.start + existing.length)
                    || (existing.start > start_pos && existing.start < start_pos + block.length)
                {
                    start_pos = existing.start + existing.length;
                    continue 'outer;
                }
            }
            break;
        }

        compacted.push(BlockPosition {
            id: block.id,
            start: start_pos,
            length: block.length,
        });
    }

    compacted.sort_by_key(|pos| pos.start);
    compacted
}

fn checksum(compacted: &[BlockPosition]) -> i64 {
    let mut sorted = compacted.to_vec();
    sorted.sort_by_key(|block| block.start);

    sorted
        .iter()
        .flat_map(|block| (block.start..block.start + block.length).map(move |pos| pos * block.id))
        .map(|x| x as i64)
        .sum()
}

fn main() {
    let file = File::open("input.txt").unwrap();
    let reader = BufReader::new(file);
    let line = reader.lines().next().unwrap().unwrap();

    let pairs: Vec<FileNode> = line
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .enumerate()
        .map(|(id, chunk)| {
            let size = chunk[0].to_digit(10).unwrap() as i32;
            let free_size = if chunk.len() > 1 {
                chunk[1].to_digit(10).unwrap() as i32
            } else {
                0
            };
            FileNode {
                id,
                size,
                free_size,
            }
        })
        .collect();

    let fs: HashMap<usize, FileNode> = pairs.into_iter().map(|node| (node.id, node)).collect();
    let compacted = compact_blocks(fs);
    let checksum = checksum(&compacted);
    println!("Checksum: {}", checksum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_compaction_and_checksum() {
        let input = "2333133121414131402";

        let pairs: Vec<FileNode> = input
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .enumerate()
            .map(|(id, chunk)| {
                let size = chunk[0].to_digit(10).unwrap() as i32;
                let free_size = if chunk.len() > 1 {
                    chunk[1].to_digit(10).unwrap() as i32
                } else {
                    0
                };
                println!(
                    "Created FileNode: id={}, size={}, free_size={}",
                    id, size, free_size
                );
                FileNode {
                    id,
                    size,
                    free_size,
                }
            })
            .collect();

        let original_nodes: HashMap<usize, FileNode> =
            pairs.into_iter().map(|node| (node.id, node)).collect();

        let compacted = compact_blocks(original_nodes);

        println!("\nCompacted blocks:");
        for block in &compacted {
            println!(
                "Block: id={}, start={}, length={}",
                block.id, block.start, block.length
            );
        }

        let checksum = checksum(&compacted);
        println!("\nChecksum calculation:");
        for block in compacted.iter() {
            for pos in block.start..block.start + block.length {
                println!("Position {} * ID {} = {}", pos, block.id, pos * block.id);
            }
        }

        assert_eq!(checksum, 1928);
    }
}
