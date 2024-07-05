use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

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
            let output_filename = format!("{}.huff", filename);
            encode_file(filename, &output_filename)?;
        }
        "decompress" => {
            let output_filename = format!("{}.decoded", filename);
            decode_file(filename, &output_filename)?;
        }
        _ => eprintln!("Invalid command. Use 'compress' or 'decompress'."),
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Node {
    freq: usize,
    value: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq && self.value == other.value
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.freq
            .cmp(&other.freq)
            .then_with(|| self.value.cmp(&other.value))
    }
}

fn encode_file(input_filename: &str, output_filename: &str) -> Result<(), std::io::Error> {
    let mut file = File::open(input_filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let freq_map = count_frequencies(&buffer);
    let tree = build_huffman_tree(&freq_map);
    let codes = generate_codes(&tree);

    let output_file = File::create(output_filename)?;
    let mut writer = BufWriter::new(output_file);

    for (byte, freq) in &freq_map {
        write!(writer, "{}:{},", byte, freq)?;
    }
    writeln!(writer)?;

    let mut bit_writer = BitWriter::new(&mut writer);
    for &byte in &buffer {
        if let Some(code) = codes.get(&byte) {
            for &bit in code {
                bit_writer.write_bit(bit)?;
            }
        }
    }
    bit_writer.flush()?;

    Ok(())
}

fn decode_file(input_filename: &str, output_filename: &str) -> Result<(), std::io::Error> {
    let mut reader = BufReader::new(File::open(input_filename)?);
    let mut header = String::new();
    reader.read_line(&mut header)?;

    let mut freq_map = HashMap::new();
    for part in header.trim().split(',') {
        if let Some((byte_str, freq_str)) = part.split_once(':') {
            if let (Ok(byte), Ok(freq)) = (byte_str.parse::<u8>(), freq_str.parse::<usize>()) {
                freq_map.insert(byte, freq);
            }
        }
    }

    let tree = build_huffman_tree(&freq_map);
    let mut writer = BufWriter::new(File::create(output_filename)?);
    let mut current = &tree;
    let mut bit_reader = BitReader::new(reader);

    loop {
        match bit_reader.read_bit()? {
            Some(false) => {
                if let Some(ref left) = current.left {
                    current = left;
                }
            }
            Some(true) => {
                if let Some(ref right) = current.right {
                    current = right;
                }
            }
            None => break,
        }

        if let Some(byte) = current.value {
            writer.write_all(&[byte])?;
            current = &tree;
        }
    }

    Ok(())
}

fn build_huffman_tree(freq_map: &HashMap<u8, usize>) -> Node {
    let mut nodes: Vec<Node> = freq_map
        .iter()
        .map(|(&byte, &freq)| Node {
            freq,
            value: Some(byte),
            left: None,
            right: None,
        })
        .collect();

    while nodes.len() > 1 {
        nodes.sort_by(|a, b| b.cmp(a)); // Sort in descending order
        let right = nodes.pop().unwrap();
        let left = nodes.pop().unwrap();
        let parent = Node {
            freq: left.freq + right.freq,
            value: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        nodes.push(parent);
    }

    nodes.pop().unwrap()
}

fn count_frequencies(buffer: &[u8]) -> HashMap<u8, usize> {
    let mut freq_map = HashMap::new();
    for &byte in buffer {
        *freq_map.entry(byte).or_insert(0) += 1;
    }
    freq_map
}

fn generate_codes(root: &Node) -> HashMap<u8, Vec<bool>> {
    let mut codes = HashMap::new();
    let mut stack = vec![(root, Vec::new())];

    while let Some((node, code)) = stack.pop() {
        if let Some(byte) = node.value {
            codes.insert(byte, code);
        } else {
            if let Some(ref left) = node.left {
                let mut left_code = code.clone();
                left_code.push(false);
                stack.push((left, left_code));
            }
            if let Some(ref right) = node.right {
                let mut right_code = code.clone();
                right_code.push(true);
                stack.push((right, right_code));
            }
        }
    }

    return codes;
}

// BitWriter struct for writing individual bits
struct BitWriter<W: Write> {
    writer: W,
    current_byte: u8,
    bit_count: u8,
}

impl<W: Write> BitWriter<W> {
    fn new(writer: W) -> Self {
        BitWriter {
            writer,
            current_byte: 0,
            bit_count: 0,
        }
    }

    fn write_bit(&mut self, bit: bool) -> Result<(), std::io::Error> {
        self.current_byte <<= 1;
        if bit {
            self.current_byte |= 1;
        }
        self.bit_count += 1;

        if self.bit_count == 8 {
            self.writer.write_all(&[self.current_byte])?;
            self.current_byte = 0;
            self.bit_count = 0;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        if self.bit_count > 0 {
            self.current_byte <<= 8 - self.bit_count;
            self.writer.write_all(&[self.current_byte])?;
        }
        self.writer.flush()
    }
}

// BitReader struct for reading individual bits
struct BitReader<R: Read> {
    reader: R,
    current_byte: u8,
    bit_count: u8,
}

impl<R: Read> BitReader<R> {
    fn new(reader: R) -> Self {
        BitReader {
            reader,
            current_byte: 0,
            bit_count: 0,
        }
    }

    fn read_bit(&mut self) -> Result<Option<bool>, std::io::Error> {
        if self.bit_count == 0 {
            let mut buffer = [0u8; 1];
            match self.reader.read_exact(&mut buffer) {
                Ok(_) => {
                    self.current_byte = buffer[0];
                    self.bit_count = 8;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        self.bit_count -= 1;
        let bit = (self.current_byte & (1 << self.bit_count)) != 0;
        Ok(Some(bit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufRead, Read, Write};
    use std::path::PathBuf;

    fn create_temp_file(filename: &str, content: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(filename);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        return path;
    }

    #[test]
    fn test_count_frequencies() {
        let freq_map = count_frequencies(b"abracadabra");
        assert_eq!(freq_map.get(&b'a'), Some(&5));
        assert_eq!(freq_map.get(&b'b'), Some(&2));
        assert_eq!(freq_map.get(&b'r'), Some(&2));
        assert_eq!(freq_map.get(&b'c'), Some(&1));
        assert_eq!(freq_map.get(&b'd'), Some(&1));
    }

    #[test]
    fn test_build_huffman_tree() {
        let mut freq_map = HashMap::new();
        freq_map.insert(b'a', 3);
        freq_map.insert(b'b', 2);
        freq_map.insert(b'c', 1);

        let tree = build_huffman_tree(&freq_map);

        // Check the root
        assert_eq!(tree.freq, 6);
        assert_eq!(tree.value, None);

        // Check left child (should be 'a')
        let left = tree.left.as_ref().unwrap();
        assert_eq!(left.freq, 3);
        assert_eq!(left.value, Some(b'a'));

        // Check right child (should be a subtree)
        let right = tree.right.as_ref().unwrap();
        assert_eq!(right.freq, 3);
        assert_eq!(right.value, None);

        // Check right subtree
        let right_left = right.left.as_ref().unwrap();
        let right_right = right.right.as_ref().unwrap();

        assert_eq!(right_left.freq, 2);
        assert_eq!(right_right.freq, 1);
        assert_eq!(right_left.value, Some(b'b'));
        assert_eq!(right_right.value, Some(b'c'));
    }

    #[test]
    fn test_generate_codes() {
        let mut freq_map = HashMap::new();
        freq_map.insert(b'a', 3);
        freq_map.insert(b'b', 2);
        freq_map.insert(b'c', 1);

        let tree = build_huffman_tree(&freq_map);
        let codes = generate_codes(&tree);

        assert_eq!(codes.len(), 3);
        assert!(codes.contains_key(&b'a'));
        assert!(codes.contains_key(&b'b'));
        assert!(codes.contains_key(&b'c'));

        // Check that no code is a prefix of another
        for (&b1, code1) in &codes {
            for (&b2, code2) in &codes {
                if b1 != b2 {
                    assert!(!is_prefix(code1, code2));
                    assert!(!is_prefix(code2, code1));
                }
            }
        }

        // Check that the more frequent character has a shorter or equal-length code
        assert!(codes[&b'a'].len() <= codes[&b'b'].len());
        assert!(codes[&b'b'].len() <= codes[&b'c'].len());
    }

    fn is_prefix(a: &[bool], b: &[bool]) -> bool {
        a.len() <= b.len() && a.iter().zip(b).all(|(x, y)| x == y)
    }

    #[test]
    fn test_encode_decode_file() {
        let content = b"hello huffman coding";
        let input_file_path = create_temp_file("test_encode_file.txt", content);
        let encoded_file_path = std::env::temp_dir().join("test_encode_file.huff");
        let decoded_file_path = std::env::temp_dir().join("decoded_file.txt");

        encode_file(
            input_file_path.to_str().unwrap(),
            encoded_file_path.to_str().unwrap(),
        )
        .unwrap();

        decode_file(
            encoded_file_path.to_str().unwrap(),
            decoded_file_path.to_str().unwrap(),
        )
        .unwrap();

        let mut decoded_content = Vec::new();
        File::open(&decoded_file_path)
            .unwrap()
            .read_to_end(&mut decoded_content)
            .unwrap();

        assert_eq!(decoded_content, content);
    }

    #[test]
    fn test_encoded_file_header() {
        let input_file_path = create_temp_file("test_header.txt", b"hello huffman");
        let encoded_file_path = std::env::temp_dir().join("test_header.huff");

        encode_file(
            input_file_path.to_str().unwrap(),
            encoded_file_path.to_str().unwrap(),
        )
        .unwrap();

        let mut encoded_file = File::open(&encoded_file_path).unwrap();
        let mut header = String::new();
        BufReader::new(&mut encoded_file)
            .read_line(&mut header)
            .unwrap();

        assert!(header.contains("104:2")); // 'h'
        assert!(header.contains("101:1")); // 'e'
        assert!(header.contains("108:2")); // 'l'
        assert!(header.contains("111:1")); // 'o'
        assert!(header.contains("32:1")); // ' '
        assert!(header.contains("117:1")); // 'u'
        assert!(header.contains("102:2")); // 'f'
        assert!(header.contains("109:1")); // 'm'
        assert!(header.contains("97:1")); // 'a'
        assert!(header.contains("110:1")); // 'n'
    }
}
