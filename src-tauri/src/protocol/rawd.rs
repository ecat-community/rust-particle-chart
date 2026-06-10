#[derive(Debug, Clone, PartialEq)]
pub struct RawdData {
    pub array1: [u16; 256],
    pub array2: [u16; 256],
}

impl RawdData {
    pub fn max_value(&self) -> u16 {
        let max1 = self.array1.iter().max().copied().unwrap_or(0);
        let max2 = self.array2.iter().max().copied().unwrap_or(0);
        max1.max(max2)
    }
}

pub fn parse_hex_line(line: &str) -> Result<[u16; 256], String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_line_valid() {
        let binding = "0001 0002 0003 0004 ".repeat(64);
        let line = binding.trim();
        let result = parse_hex_line(line).unwrap();
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 2);
        assert_eq!(result[2], 3);
        assert_eq!(result[3], 4);
    }

    #[test]
    fn test_parse_hex_line_wrong_count() {
        let line = "0001 0002 0003";
        let result = parse_hex_line(line);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 256 hex values, got 3"));
    }

    #[test]
    fn test_parse_hex_line_invalid_hex() {
        let binding = "ZZZZ ".repeat(256);
        let line = binding.trim();
        let result = parse_hex_line(line);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex"));
    }

    #[test]
    fn test_max_value() {
        let mut array1 = [0u16; 256];
        let mut array2 = [0u16; 256];
        array1[10] = 100;
        array2[20] = 200;
        array1[30] = 150;

        let data = RawdData { array1, array2 };
        assert_eq!(data.max_value(), 200);
    }

    #[test]
    fn test_parse_real_data_fragment() {
        // Sample from actual data - first few values from line 4 of data-example.txt
        let line = "0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0001 0004 0005 0012 0016 001E 001D 0033 0046 0038 0037 0067 005C 004F 0043";
        let remaining = "0000 ".repeat(256 - 31); // 31 values in line, need 225 more
        let line = format!("{} {}", line.trim(), remaining.trim());

        let result = parse_hex_line(&line).unwrap();
        assert_eq!(result[0], 0);
        assert_eq!(result[16], 1);
        assert_eq!(result[17], 4);
        assert_eq!(result[18], 5);
        assert_eq!(result[19], 18);
        assert_eq!(result[20], 22);
        assert_eq!(result[21], 30);
        assert_eq!(result[22], 29);
        assert_eq!(result[23], 51);
        assert_eq!(result[24], 70);
        assert_eq!(result[25], 56);
        assert_eq!(result[26], 55);
    }
}
