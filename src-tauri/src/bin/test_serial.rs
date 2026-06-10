//! Standalone test: verifies serial data flow from simulated device through parser.
//!
//! Usage: cargo run --bin test-serial /tmp/ttyV0
//! Prerequisites: socat running, simulated-device running on /tmp/ttyV1

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// Inline the parser logic to avoid pulling in the full crate
fn parse_hex_line(line: &str) -> Result<[u16; 256], String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 256 {
        return Err(format!("Expected 256 hex values, got {}", parts.len()));
    }
    let mut array = [0u16; 256];
    for (i, hex) in parts.iter().enumerate() {
        array[i] = u16::from_str_radix(hex, 16)
            .map_err(|e| format!("Invalid hex '{}' at index {}: {}", hex, i, e))?;
    }
    Ok(array)
}

#[derive(Debug)]
struct RawdData {
    array1: [u16; 256],
    array2: [u16; 256],
}

enum ParserState {
    Idle,
    InRawdHeader,
    InRawdArray1(Box<[u16; 256]>),
}

struct Parser {
    state: ParserState,
}

impl Parser {
    fn new() -> Self {
        Parser {
            state: ParserState::Idle,
        }
    }

    fn feed_line(&mut self, line: &str) -> Option<RawdData> {
        let line = line.trim();
        match &self.state {
            ParserState::Idle => {
                if line.contains("#RAWD") {
                    eprintln!("  [PARSER] Found #RAWD header");
                    self.state = ParserState::InRawdHeader;
                }
                None
            }
            ParserState::InRawdHeader => {
                match parse_hex_line(line) {
                    Ok(array1) => {
                        eprintln!("  [PARSER] Got array1, first 5: {:?}", &array1[0..5]);
                        self.state = ParserState::InRawdArray1(Box::new(array1));
                    }
                    Err(_) => {
                        eprintln!("  [PARSER] Metadata line: '{}'", line);
                    }
                }
                None
            }
            ParserState::InRawdArray1(array1) => match parse_hex_line(line) {
                Ok(array2) => {
                    let result = RawdData {
                        array1: **array1,
                        array2,
                    };
                    self.state = ParserState::Idle;
                    Some(result)
                }
                Err(_) => {
                    self.state = ParserState::Idle;
                    None
                }
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let port_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ttyV0".to_string());
    eprintln!("=== Serial Backend Test ===");
    eprintln!("Opening {} ...", port_path);

    // Try tokio-serial first, fall back to plain file
    let file = tokio::fs::File::options()
        .read(true)
        .write(true)
        .open(&port_path)
        .await
        .expect("Failed to open port");

    eprintln!("Port opened.");
    let (read_half, mut write_half) = tokio::io::split(file);
    let reader = BufReader::new(read_half);
    let mut lines = reader.lines();

    // Send start command
    eprintln!("Sending start command '27'...");
    write_half
        .write_all(b"27\r\n")
        .await
        .expect("Failed to write");
    write_half.flush().await.expect("Failed to flush");

    eprintln!("Waiting for data...");

    let mut parser = Parser::new();
    let mut line_count = 0u32;
    let mut rawd_count = 0u32;
    let timeout = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    line_count += 1;
                    if line_count <= 5 || line.contains("#RAWD") {
                        eprintln!("  Line {}: {:.80}", line_count, line);
                    }
                    if let Some(data) = parser.feed_line(&line) {
                        rawd_count += 1;
                        let max1 = data.array1.iter().max().unwrap_or(&0);
                        let max2 = data.array2.iter().max().unwrap_or(&0);
                        eprintln!("  >>> RAWD #{}: max1={}, max2={}", rawd_count, max1, max2);
                        if rawd_count >= 1 {
                            break; // Got at least one, success
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("  EOF at line {}", line_count);
                    break;
                }
                Err(e) => {
                    eprintln!("  Read error at line {}: {}", line_count, e);
                    break;
                }
            }
        }
    })
    .await;

    match timeout {
        Ok(_) => {
            eprintln!("=== Test Result ===");
            eprintln!("Lines read: {}", line_count);
            eprintln!("RAWD blocks parsed: {}", rawd_count);
            if rawd_count > 0 {
                eprintln!("PASS: Backend serial communication works!");
            } else {
                eprintln!("FAIL: No RAWD data parsed");
            }
        }
        Err(_) => {
            eprintln!(
                "TIMEOUT after 10s. Lines: {}, RAWDs: {}",
                line_count, rawd_count
            );
        }
    }
}
