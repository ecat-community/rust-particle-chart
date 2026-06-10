use std::fmt;

#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig {
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".to_string(),
        }
    }
}

impl SerialConfig {
    const VALID_BAUD_RATES: [u32; 9] = [
        4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];

    pub fn validate(&self) -> Result<(), String> {
        if self.port_name.trim().is_empty() {
            return Err("port_name cannot be empty".to_string());
        }

        if !Self::VALID_BAUD_RATES.contains(&self.baud_rate) {
            return Err(format!(
                "invalid baud_rate: {}. Valid values: {:?}",
                self.baud_rate, Self::VALID_BAUD_RATES
            ));
        }

        if !(5..=8).contains(&self.data_bits) {
            return Err(format!(
                "invalid data_bits: {}. Must be between 5 and 8",
                self.data_bits
            ));
        }

        if self.stop_bits != 1 && self.stop_bits != 2 {
            return Err(format!(
                "invalid stop_bits: {}. Must be 1 or 2",
                self.stop_bits
            ));
        }

        let parity_lower = self.parity.to_lowercase();
        if !matches!(parity_lower.as_str(), "none" | "odd" | "even") {
            return Err(format!(
                "invalid parity: {}. Must be 'none', 'odd', or 'even'",
                self.parity
            ));
        }

        Ok(())
    }

    pub fn to_tokio_data_bits(&self) -> tokio_serial::DataBits {
        match self.data_bits {
            5 => tokio_serial::DataBits::Five,
            6 => tokio_serial::DataBits::Six,
            7 => tokio_serial::DataBits::Seven,
            8 => tokio_serial::DataBits::Eight,
            _ => tokio_serial::DataBits::Eight, // Default fallback, should not happen if validated
        }
    }

    pub fn to_tokio_stop_bits(&self) -> tokio_serial::StopBits {
        match self.stop_bits {
            1 => tokio_serial::StopBits::One,
            2 => tokio_serial::StopBits::Two,
            _ => tokio_serial::StopBits::One, // Default fallback, should not happen if validated
        }
    }

    pub fn to_tokio_parity(&self) -> tokio_serial::Parity {
        match self.parity.to_lowercase().as_str() {
            "none" => tokio_serial::Parity::None,
            "odd" => tokio_serial::Parity::Odd,
            "even" => tokio_serial::Parity::Even,
            _ => tokio_serial::Parity::None, // Default fallback, should not happen if validated
        }
    }
}

impl fmt::Display for SerialConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SerialConfig {{ port_name: {}, baud_rate: {}, data_bits: {}, stop_bits: {}, parity: {} }}",
            self.port_name, self.baud_rate, self.data_bits, self.stop_bits, self.parity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SerialConfig {
        SerialConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".to_string(),
        }
    }

    #[test]
    fn test_valid_default_config() {
        let config = SerialConfig::default();
        let mut test_config = config;
        test_config.port_name = "/dev/ttyUSB0".to_string();
        assert!(test_config.validate().is_ok());
    }

    #[test]
    fn test_empty_port_name_fails() {
        let mut config = create_test_config();
        config.port_name = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port_name cannot be empty"));
    }

    #[test]
    fn test_whitespace_only_port_name_fails() {
        let mut config = create_test_config();
        config.port_name = "   ".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port_name cannot be empty"));
    }

    #[test]
    fn test_invalid_baud_rate_fails() {
        let mut config = create_test_config();
        config.baud_rate = 12345;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid baud_rate"));
    }

    #[test]
    fn test_invalid_data_bits_fails() {
        let mut config = create_test_config();
        config.data_bits = 9;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid data_bits"));
    }

    #[test]
    fn test_invalid_data_bits_too_low_fails() {
        let mut config = create_test_config();
        config.data_bits = 4;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid data_bits"));
    }

    #[test]
    fn test_invalid_stop_bits_fails() {
        let mut config = create_test_config();
        config.stop_bits = 3;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid stop_bits"));
    }

    #[test]
    fn test_invalid_parity_fails() {
        let mut config = create_test_config();
        config.parity = "invalid".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid parity"));
    }

    #[test]
    fn test_all_valid_baud_rates() {
        let valid_rates = SerialConfig::VALID_BAUD_RATES;
        for baud_rate in valid_rates {
            let mut config = create_test_config();
            config.baud_rate = baud_rate;
            assert!(config.validate().is_ok(), "baud_rate {} should be valid", baud_rate);
        }
    }

    #[test]
    fn test_valid_data_bits_range() {
        for data_bits in 5..=8 {
            let mut config = create_test_config();
            config.data_bits = data_bits;
            assert!(config.validate().is_ok(), "data_bits {} should be valid", data_bits);
        }
    }

    #[test]
    fn test_parity_conversion() {
        let mut config = create_test_config();

        config.parity = "none".to_string();
        assert_eq!(config.to_tokio_parity(), tokio_serial::Parity::None);
        assert!(config.validate().is_ok());

        config.parity = "odd".to_string();
        assert_eq!(config.to_tokio_parity(), tokio_serial::Parity::Odd);
        assert!(config.validate().is_ok());

        config.parity = "even".to_string();
        assert_eq!(config.to_tokio_parity(), tokio_serial::Parity::Even);
        assert!(config.validate().is_ok());

        config.parity = "NONE".to_string();
        assert_eq!(config.to_tokio_parity(), tokio_serial::Parity::None);
        assert!(config.validate().is_ok());

        config.parity = "Odd".to_string();
        assert_eq!(config.to_tokio_parity(), tokio_serial::Parity::Odd);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_data_bits_conversion() {
        let mut config = create_test_config();

        for data_bits in 5..=8 {
            config.data_bits = data_bits;
            let tokio_bits = config.to_tokio_data_bits();
            match data_bits {
                5 => assert_eq!(tokio_bits, tokio_serial::DataBits::Five),
                6 => assert_eq!(tokio_bits, tokio_serial::DataBits::Six),
                7 => assert_eq!(tokio_bits, tokio_serial::DataBits::Seven),
                8 => assert_eq!(tokio_bits, tokio_serial::DataBits::Eight),
                _ => panic!("Unexpected data_bits value"),
            }
        }
    }

    #[test]
    fn test_stop_bits_conversion() {
        let mut config = create_test_config();

        config.stop_bits = 1;
        assert_eq!(config.to_tokio_stop_bits(), tokio_serial::StopBits::One);

        config.stop_bits = 2;
        assert_eq!(config.to_tokio_stop_bits(), tokio_serial::StopBits::Two);
    }

    #[test]
    fn test_complete_valid_config() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.data_bits, 8);
        assert_eq!(config.stop_bits, 1);
        assert_eq!(config.parity, "none");
    }
}
