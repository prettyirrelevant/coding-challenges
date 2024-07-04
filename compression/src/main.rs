use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <compress|decompress> <filename>", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "compress" => {
            let output_filename = generate_output_filename(filename, "encode")?;
            encode_file(filename, &output_filename)?
        }
        // "decompress" => {
        //     let output_filename = generate_output_filename(filename, "decode")?;
        //     decode_file(filename, &output_filename)?
        // }
        _ => eprintln!("Invalid command. Use 'compress' or 'decompress'."),
    }

    Ok(())
}

// Encode the input file using Huffman coding
fn encode_file(input_filename: &str, output_filename: &str) -> Result<(), std::io::Error> {
    let freq_map = count_frequencies(input_filename)?;
    let tree = build_huffman_tree(&freq_map);

    let mut codes = HashMap::new();
    generate_codes(&tree, String::new(), &mut codes);

    let input_file = File::open(input_filename)?;
    let mut reader = BufReader::new(input_file);
    let output_file = File::create(output_filename)?;
    let mut writer = BufWriter::new(output_file);

    // Write file extension to header
    let extension = Path::new(input_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    write!(writer, "{}|", extension)?;

    // Write frequency table to header
    for (c, freq) in &freq_map {
        write!(writer, "{}:{},", c, freq)?;
    }
    writeln!(writer)?;

    // Encode file contents
    let mut buffer = [0; 8192];
    let mut byte = 0u8;
    let mut bit_count = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        for &b in &buffer[..bytes_read] {
            if let Some(code) = codes.get(&(b as char)) {
                for bit in code.chars() {
                    byte <<= 1;
                    if bit == '1' {
                        byte |= 1;
                    }
                    bit_count += 1;
                    if bit_count == 8 {
                        writer.write_all(&[byte])?;
                        byte = 0;
                        bit_count = 0;
                    }
                }
            }
        }
    }

    // Write any remaining bits
    if bit_count > 0 {
        byte <<= 8 - bit_count;
        writer.write_all(&[byte])?;
    }

    println!("Encoded file saved as: {}", output_filename);
    Ok(())
}

// Decode the input file using Huffman coding
// fn decode_file(input_filename: &str, output_filename: &str) -> Result<(), std::io::Error> {
//     let input_file = File::open(input_filename)?;
//     let mut reader = BufReader::new(&input_file);

//     // Read and parse header
//     let mut header = String::new();
//     reader.read_line(&mut header)?;

//     let header_parts: Vec<&str> = header.trim().split('|').collect();
//     let freq_str = header_parts[1];

//     let mut freq_map = HashMap::new();
//     for part in freq_str.split(',') {
//         if let Some((c, freq)) = part.split_once(':') {
//             if let (Some(c), Ok(freq)) = (c.chars().next(), freq.parse::<usize>()) {
//                 freq_map.insert(c, freq);
//             }
//         }
//     }

//     let tree = build_huffman_tree(&freq_map);

//     let output_file = File::create(&output_filename)?;
//     let mut writer = BufWriter::new(output_file);

//     // Decode file contents
//     let mut current = &tree;
//     let mut buffer = [0; 8192];
//     let mut bits_to_read = 8; // Number of bits left to read in the current byte

//     loop {
//         let bytes_read = reader.read(&mut buffer)?;

//         if bytes_read == 0 {
//             // Handle any remaining bits in the buffer
//             for i in (8 - bits_to_read)..8 {
//                 let bit = (buffer[bytes_read - 1] >> i) & 1;
//                 current = if bit == 0 {
//                     current.left.as_ref().unwrap()
//                 } else {
//                     current.right.as_ref().unwrap()
//                 };

//                 if let Some(c) = current.char {
//                     write!(writer, "{}", c)?;
//                     current = &tree;
//                 }
//             }
//             break;
//         }

//         for &byte in &buffer[..bytes_read] {
//             for i in (8 - bits_to_read)..8 {
//                 let bit = (byte >> i) & 1;
//                 current = if bit == 0 {
//                     current.left.as_ref().unwrap()
//                 } else {
//                     current.right.as_ref().unwrap()
//                 };

//                 if let Some(c) = current.char {
//                     write!(writer, "{}", c)?;
//                     current = &tree;
//                 }
//             }
//             bits_to_read = 0; // Reset bits to read for the next byte
//         }
//     }

//     println!("Decoded file saved as: {}", output_filename);
//     Ok(())
// }

// Node structure for Huffman tree
#[derive(Debug, Eq)]
struct Node {
    char: Option<char>,
    freq: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

// Implement ordering for Node to use in BinaryHeap
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

// Build Huffman tree from character frequencies
fn build_huffman_tree(freq_map: &HashMap<char, usize>) -> Node {
    let mut heap = BinaryHeap::new();

    // Create leaf nodes for each character
    for (&c, &freq) in freq_map {
        heap.push(Node {
            char: Some(c),
            freq,
            left: None,
            right: None,
        });
    }

    // Build the tree by combining nodes
    while heap.len() > 1 {
        let left = Box::new(heap.pop().unwrap());
        let right = Box::new(heap.pop().unwrap());
        let combined_freq = left.freq + right.freq;
        heap.push(Node {
            char: None,
            freq: combined_freq,
            left: Some(left),
            right: Some(right),
        });
    }

    heap.pop().unwrap()
}

// Generate Huffman codes for each character
fn generate_codes(root: &Node, prefix: String, codes: &mut HashMap<char, String>) {
    if let Some(c) = root.char {
        codes.insert(c, prefix);
    } else {
        if let Some(ref left) = root.left {
            generate_codes(left, prefix.clone() + "0", codes);
        }
        if let Some(ref right) = root.right {
            generate_codes(right, prefix + "1", codes);
        }
    }
}

// Count character frequencies in the input file
fn count_frequencies(filename: &str) -> Result<HashMap<char, usize>, std::io::Error> {
    let file = File::open(filename)?;
    let mut buffer = [0; 8192];
    let mut reader = BufReader::new(file);
    let mut char_freq = HashMap::new();

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        for &byte in &buffer[..bytes_read] {
            *char_freq.entry(byte as char).or_insert(0) += 1;
        }
    }

    Ok(char_freq)
}

// Generate output filename based on input filename and operation
fn generate_output_filename(input: &str, operation: &str) -> Result<String, std::io::Error> {
    let path = Path::new(input);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let extension = match operation {
        "compress" => "huff".to_string(),
        "decompress" => {
            let input_file = File::open(input)?;
            let mut reader = BufReader::new(input_file);

            let mut buffer = Vec::new();
            reader.read_until(b'|', &mut buffer)?;

            let header = String::from_utf8_lossy(&buffer);
            header.replace('|', "").trim().to_string()
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid operation",
            ));
        }
    };

    if extension.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid header format",
        ));
    }

    Ok(format!("{}.{}", stem, extension))
}
