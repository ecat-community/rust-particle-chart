use super::config::SerialConfig;
use crate::protocol::parser::ProtocolParser;
use crate::protocol::rawd::RawdData;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum SerialStatus {
    Connected,
    Reconnecting,
    Disconnected,
}

pub struct SerialManager {
    config: SerialConfig,
    status_sender: watch::Sender<SerialStatus>,
    cancel_sender: Option<watch::Sender<()>>,
    _cancel_receiver: Option<watch::Receiver<()>>,
}

impl SerialManager {
    pub fn new(config: SerialConfig) -> Self {
        let (status_sender, _) = watch::channel(SerialStatus::Disconnected);
        SerialManager {
            config,
            status_sender,
            cancel_sender: None,
            _cancel_receiver: None,
        }
    }

    #[allow(dead_code)]
    pub fn status_receiver(&self) -> watch::Receiver<SerialStatus> {
        self.status_sender.subscribe()
    }

    pub fn start<F>(&mut self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(RawdData) + Send + 'static,
    {
        self.config.validate()?;

        let (cancel_sender, cancel_receiver) = watch::channel(());
        self.cancel_sender = Some(cancel_sender);
        self._cancel_receiver = Some(cancel_receiver.clone());

        let config = self.config.clone();
        let status_sender = self.status_sender.clone();

        tokio::spawn(async move {
            let mut reconnect_delay = Duration::from_secs(1);
            let mut parser = ProtocolParser::new();
            let mut cancel_receiver = cancel_receiver;

            loop {
                if cancel_receiver.has_changed().unwrap_or(false) {
                    break;
                }

                match Self::open_and_read(
                    &config,
                    &mut parser,
                    &mut callback,
                    &mut cancel_receiver,
                    &status_sender,
                )
                .await
                {
                    Ok(_) => {
                        reconnect_delay = Duration::from_secs(1);
                    }
                    Err(e) => {
                        if e == "Cancelled" {
                            break;
                        }
                        let _ = status_sender.send(SerialStatus::Reconnecting);

                        if cancel_receiver.has_changed().unwrap_or(false) {
                            break;
                        }

                        tokio::time::sleep(reconnect_delay).await;

                        reconnect_delay =
                            std::cmp::min(reconnect_delay * 2, Duration::from_secs(30));
                    }
                }
            }

            let _ = status_sender.send(SerialStatus::Disconnected);
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(cancel_sender) = self.cancel_sender.take() {
            let _ = cancel_sender.send(());
        }
        let _ = self.status_sender.send(SerialStatus::Disconnected);
        Ok(())
    }

    async fn open_and_read<F>(
        config: &SerialConfig,
        parser: &mut ProtocolParser,
        callback: &mut F,
        cancel_receiver: &mut watch::Receiver<()>,
        status_sender: &watch::Sender<SerialStatus>,
    ) -> Result<(), String>
    where
        F: FnMut(RawdData),
    {
        // Open serial port via tokio_serial — no file IO fallback
        let builder = tokio_serial::new(&config.port_name, config.baud_rate)
            .data_bits(config.to_tokio_data_bits())
            .stop_bits(config.to_tokio_stop_bits())
            .parity(config.to_tokio_parity());

        let port = tokio_serial::SerialStream::open(&builder)
            .map_err(|e| format!("Failed to open {}: {}", config.port_name, e))?;
        eprintln!("[SERIAL] Opened via tokio_serial: {}", config.port_name);

        let (read_half, mut write_half) = tokio::io::split(port);
        let reader = BufReader::new(read_half);

        // Send start command
        write_half
            .write_all(b"27\r\n")
            .await
            .map_err(|e| format!("Failed to send start command: {}", e))?;
        eprintln!("[SERIAL] Sent start command '27'");

        let _ = status_sender.send(SerialStatus::Connected);

        let mut lines = reader.lines();
        let mut line_count = 0u32;
        let mut rawd_count = 0u32;

        loop {
            tokio::select! {
                _ = cancel_receiver.changed() => {
                    eprintln!("[SERIAL] Cancel received. lines={}, rawds={}", line_count, rawd_count);
                    return Err("Cancelled".to_string());
                }

                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            line_count += 1;
                            if line_count <= 10 || line.contains("#RAWD") {
                                eprintln!("[SERIAL] Line {}: {:.80}", line_count, line);
                            }
                            if let Some(rawd_data) = parser.feed_line(&line) {
                                rawd_count += 1;
                                eprintln!("[SERIAL] Parsed RAWD #{}: max={}", rawd_count, rawd_data.max_value());
                                callback(rawd_data);
                            }
                        }
                        Ok(None) => {
                            eprintln!("[SERIAL] EOF after {} lines, {} rawds", line_count, rawd_count);
                            break;
                        }
                        Err(e) => {
                            eprintln!("[SERIAL] Read error after {} lines: {}", line_count, e);
                            return Err("Connection lost".to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn list_ports() -> Result<Vec<String>, String> {
    tokio_serial::available_ports()
        .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
        .map_err(|e| format!("Failed to list ports: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        let result = list_ports();
        assert!(result.is_ok(), "list_ports should return Ok");
    }

    #[test]
    fn test_start_invalid_config() {
        let mut manager = SerialManager::new(SerialConfig {
            port_name: "".to_string(),
            ..Default::default()
        });

        let result = manager.start(|_| {});
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port_name cannot be empty"));
    }

    #[test]
    fn test_start_nonexistent_port() {
        let mut manager = SerialManager::new(SerialConfig {
            port_name: "/dev/ttyNONEXISTENT123".to_string(),
            ..Default::default()
        });

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { manager.start(|_| {}) });

        assert!(result.is_ok());

        std::thread::sleep(std::time::Duration::from_millis(100));

        let status_rx = manager.status_receiver();
        let current_status = *status_rx.borrow();
        assert!(
            current_status == SerialStatus::Reconnecting
                || current_status == SerialStatus::Disconnected
        );
    }

    #[test]
    fn test_stop_without_start() {
        let mut manager = SerialManager::new(SerialConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        });

        let result = manager.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_receiver() {
        let manager = SerialManager::new(SerialConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        });

        let status_rx = manager.status_receiver();
        assert_eq!(*status_rx.borrow(), SerialStatus::Disconnected);
    }

    #[test]
    fn test_stop_cancels_running_task() {
        let mut manager = SerialManager::new(SerialConfig {
            port_name: "/dev/ttyNONEXISTENT123".to_string(),
            ..Default::default()
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            manager.start(|_| {}).unwrap();

            tokio::time::sleep(Duration::from_millis(50)).await;

            manager.stop().unwrap();

            tokio::time::sleep(Duration::from_millis(50)).await;

            let status_rx = manager.status_receiver();
            assert_eq!(*status_rx.borrow(), SerialStatus::Disconnected);
        });
    }
}
