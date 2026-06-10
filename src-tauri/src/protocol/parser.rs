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
                    self.state = ParserState::InRawdHeader;
                }
                None
            }
            ParserState::InRawdHeader => {
                match parse_hex_line(line) {
                    Ok(array1) => {
                        self.state = ParserState::InRawdArray1(array1);
                    }
                    Err(_) => {
                        // Metadata line (e.g. " 10 3684"), wait for actual hex data
                    }
                }
                None
            }
            ParserState::InRawdArray1(array1) => match parse_hex_line(line) {
                Ok(array2) => {
                    let result = RawdData {
                        array1: *array1,
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

        assert!(parser.feed_line("'#RAWD").is_none());
        assert!(parser.feed_line(" 10 3684").is_none());

        let binding1 = "0001 ".repeat(256);
        let array1_line = binding1.trim();
        assert!(parser.feed_line(array1_line).is_none());

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

        assert!(parser.feed_line("'#RAWD").is_none());
        assert!(parser.feed_line(" 10 3684").is_none());
        assert!(parser.feed_line("INVALID DATA HERE").is_none());
        assert!(parser.feed_line("'#RAWD").is_none());
    }

    const SAMPLE_PROTOCOL_DATA: &str = include_str!("../../tests/data-example.txt");

    #[test]
    fn test_parse_real_data_example() {
        let mut parser = ProtocolParser::new();
        let mut rawd_count = 0;

        for line in SAMPLE_PROTOCOL_DATA.lines() {
            if parser.feed_line(line).is_some() {
                rawd_count += 1;
            }
        }

        assert_eq!(rawd_count, 4, "Expected 4 RAWD blocks, got {}", rawd_count);
    }
}
