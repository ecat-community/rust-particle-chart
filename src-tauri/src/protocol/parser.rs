use crate::protocol::rawd::{parse_hex_line, RawdData};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
enum ParserState {
    Idle,
    InRawdHeader,
    InRawdArray1([u16; 256]),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolParser {
    state: ParserState,
}

impl ProtocolParser {
    pub fn new() -> Self {
        ProtocolParser {
            state: ParserState::Idle,
        }
    }

    pub fn feed_line(&mut self, line: &str) -> Option<RawdData> {
        let line = line.trim();

        match &self.state {
            ParserState::Idle => {
                if line.contains("#RAWD") {
                    eprintln!("[PARSER] Found #RAWD header");
                    self.state = ParserState::InRawdHeader;
                }
                None
            }
            ParserState::InRawdHeader => {
                // Try to parse this line as array1
                match parse_hex_line(line) {
                    Ok(array1) => {
                        eprintln!("[PARSER] Parsed array1 (first 5: {:?})", &array1[0..5]);
                        self.state = ParserState::InRawdArray1(array1);
                    }
                    Err(e) => {
                        eprintln!("[PARSER] Header line not hex data ({}), waiting for array1", e);
                        // This isn't hex data, so it must be the metadata line (e.g., " 10 3684")
                        // Stay in InRawdHeader state - the NEXT line should be array1
                    }
                }
                None
            }
            ParserState::InRawdArray1(array1) => {
                // We have array1, now parse array2 and emit
                match parse_hex_line(line) {
                    Ok(array2) => {
                        eprintln!("[PARSER] Parsed array2 (first 5: {:?})", &array2[0..5]);
                        let result = RawdData {
                            array1: *array1,
                            array2,
                        };
                        self.state = ParserState::Idle;
                        Some(result)
                    }
                    Err(e) => {
                        eprintln!("[PARSER] Expected array2 but got error: {}. Resetting.", e);
                        self.state = ParserState::Idle;
                        None
                    }
                }
            }
        }
    }
}

impl Default for ProtocolParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_rawd_block() {
        let mut parser = ProtocolParser::new();

        // Feed header
        assert!(parser.feed_line("'#RAWD").is_none());

        // Feed metadata line
        assert!(parser.feed_line(" 10 3684").is_none());

        // Feed first array
        let binding1 = "0001 ".repeat(256);
        let array1_line = binding1.trim();
        assert!(parser.feed_line(array1_line).is_none());

        // Feed second array - should get result
        let binding2 = "0002 ".repeat(256);
        let array2_line = binding2.trim();
        let result = parser.feed_line(array2_line);

        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.array1[0], 1);
        assert_eq!(data.array2[0], 2);
        assert_eq!(data.array1[255], 1);
        assert_eq!(data.array2[255], 2);
    }

    #[test]
    fn test_parser_resets_after_partial_block() {
        let mut parser = ProtocolParser::new();

        // Feed header
        assert!(parser.feed_line("'#RAWD").is_none());

        // Feed metadata line
        assert!(parser.feed_line(" 10 3684").is_none());

        // Feed invalid first array
        assert!(parser.feed_line("INVALID DATA HERE").is_none());

        // Parser should now be reset to Idle
        assert!(parser.feed_line("'#RAWD").is_none());
    }

    #[test]
    fn test_parse_real_data_example() {
        let data = include_str!("../../../data-example.txt");
        let mut parser = ProtocolParser::new();
        let mut rawd_count = 0;

        for line in data.lines() {
            if let Some(_) = parser.feed_line(line) {
                rawd_count += 1;
            }
        }

        // The data-example.txt contains 4 RAWD blocks
        assert_eq!(rawd_count, 4, "Expected 4 RAWD blocks, got {}", rawd_count);
    }
}
