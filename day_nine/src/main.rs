use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
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

fn compact_blocks(original_nodes: HashMap<usize, FileNode>) -> Vec<BlockPosition> {
    let mut layout = Vec::new();
    let mut positions = Vec::new();

    let max_id = *original_nodes.keys().max().unwrap_or(&0);

    for id in 0..=max_id {
        if let Some(node) = original_nodes.get(&id) {
            let start = layout.len();
            for _ in 0..node.size {
                layout.push(Some(id));
            }
            positions.push(BlockPosition {
                id,
                start,
                length: node.size as usize,
            });
            for _ in 0..node.free_size {
                layout.push(None);
            }
        }
    }

    let mut result = Vec::new();

    for id in (0..=max_id).rev() {
        if let Some(node) = original_nodes.get(&id) {
            let mut orig_start = None;
            for i in 0..layout.len() {
                if layout[i] == Some(id) {
                    if orig_start.is_none() {
                        orig_start = Some(i);
                    }
                    layout[i] = None;
                }
            }

            let mut blocks_to_place = node.size as usize;

            let mut pos = 0;
            while blocks_to_place > 0 {
                while pos < layout.len() && layout[pos].is_some() {
                    pos += 1;
                }

                if pos >= layout.len() {
                    break;
                }
                let mut free_count = 0;
                let start_pos = pos;
                while pos < layout.len() && layout[pos].is_none() {
                    free_count += 1;
                    pos += 1;
                }
                let blocks_to_place_here = blocks_to_place.min(free_count);
                if blocks_to_place_here > 0 {
                    for i in start_pos..start_pos + blocks_to_place_here {
                        layout[i] = Some(id);
                    }
                    result.push(BlockPosition {
                        id,
                        start: start_pos,
                        length: blocks_to_place_here,
                    });

                    blocks_to_place -= blocks_to_place_here;
                }
            }
        }
    }

    result.sort_by_key(|b| b.start);
    result
}

fn compact_blocks_no_split(original_nodes: HashMap<usize, FileNode>) -> Vec<BlockPosition> {
    let mut layout = Vec::new();
    let mut positions = Vec::new();
    let max_id = *original_nodes.keys().max().unwrap_or(&0);

    for id in 0..=max_id {
        if let Some(node) = original_nodes.get(&id) {
            let start = layout.len();

            for _ in 0..node.size {
                layout.push(Some(id));
            }

            positions.push(BlockPosition {
                id,
                start,
                length: node.size as usize,
            });

            for _ in 0..node.free_size {
                layout.push(None);
            }
        }
    }

    let mut result = Vec::new();

    for id in (0..=max_id).rev() {
        if let Some(node) = original_nodes.get(&id) {
            let mut current_pos = 0;
            while current_pos < layout.len() && layout[current_pos] != Some(id) {
                current_pos += 1;
            }

            let mut best_pos = current_pos;

            'outer: for try_pos in 0..current_pos {
                if try_pos + node.size as usize > current_pos {
                    break;
                }

                for i in 0..node.size as usize {
                    if layout[try_pos + i].is_some() {
                        continue 'outer;
                    }
                }

                best_pos = try_pos;
                break;
            }
            if best_pos != current_pos {
                for i in 0..node.size as usize {
                    layout[current_pos + i] = None;
                }

                for i in 0..node.size as usize {
                    layout[best_pos + i] = Some(id);
                }
            }

            result.push(BlockPosition {
                id,
                start: best_pos,
                length: node.size as usize,
            });
        }
    }

    result.sort_by_key(|b| b.start);
    result
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

fn line_to_pairs(line: String) -> Vec<FileNode> {
    line.chars()
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
        .collect()
}

fn main() {
    let file = File::open("input.txt").unwrap();
    let reader = BufReader::new(file);
    let line = reader.lines().next().unwrap().unwrap();
    let pairs = line_to_pairs(line);

    let fs: HashMap<usize, FileNode> = pairs.into_iter().map(|node| (node.id, node)).collect();
    let compacted = compact_blocks(fs.clone());
    let checksum_split = checksum(&compacted);
    println!("Checksum: {}", checksum_split);
    let compacted_no_split = compact_blocks_no_split(fs.clone());
    let checksum_no_split = checksum(&compacted_no_split);
    println!("Checksum without splitting: {}", checksum_no_split);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_compaction_and_checksum() {
        let input = String::from("2333133121414131402");

        let pairs = line_to_pairs(input);

        let original_nodes: HashMap<usize, FileNode> =
            pairs.into_iter().map(|node| (node.id, node)).collect();

        let compacted = compact_blocks(original_nodes);

        let checksum = checksum(&compacted);

        assert_eq!(checksum, 1928);
    }

    #[test]
    fn test_full_compaction_no_split_and_checksum() {
        let input = String::from("2333133121414131402");

        let pairs = line_to_pairs(input);

        let original_nodes: HashMap<usize, FileNode> =
            pairs.into_iter().map(|node| (node.id, node)).collect();

        let compacted = compact_blocks_no_split(original_nodes);

        let checksum = checksum(&compacted);
        assert_eq!(checksum, 2858);
    }
}
