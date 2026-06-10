use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::SerialStream;

const DATA: &[u8] = include_bytes!("../../tests/data-example.txt");

/// Simulated particle counter device.
///
/// Protocol:
///   - Host sends "27\r\n" → device sends datasets continuously
///   - Host sends ESC "\x1B\r\n" → device stops sending, waits for next "27"
///
/// Uses tokio_serial for all I/O (patched serialport for PTY support).
/// Simple design: write with timeout, only check commands during pauses.
#[tokio::main]
async fn main() {
    let port_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ttyV1".to_string());

    eprintln!("[DEV] Simulated device starting on {}...", port_path);

    let port = open_serial(&port_path);
    eprintln!("[DEV] Port opened. Waiting for commands...");

    let (read_half, mut write_half) = tokio::io::split(port);
    let reader = BufReader::new(read_half);
    let mut lines = reader.lines();
    let mut total_datasets = 0u32;

    loop {
        // Phase 1: Wait for start command "27"
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let trimmed = line.trim();
                    eprintln!("[DEV] Cmd: {:?}", trimmed);
                    if trimmed == "27" {
                        eprintln!("[DEV] Start!");
                        break;
                    }
                }
                Ok(None) => {
                    eprintln!("[DEV] EOF, reopen...");
                    let port = open_serial(&port_path);
                    let (r, w) = tokio::io::split(port);
                    write_half = w;
                    lines = BufReader::new(r).lines();
                }
                Err(e) => {
                    eprintln!("[DEV] Read err: {}", e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let port = open_serial(&port_path);
                    let (r, w) = tokio::io::split(port);
                    write_half = w;
                    lines = BufReader::new(r).lines();
                }
            }
        }

        // Phase 2: Send datasets until ESC or error
        let mut sending = true;
        while sending {
            // Write one dataset with timeout
            match tokio::time::timeout(Duration::from_secs(5), write_half.write_all(DATA)).await {
                Ok(Ok(())) => {
                    let _ = write_half.flush().await;
                    total_datasets += 1;
                    eprintln!("[DEV] Sent #{}", total_datasets);
                }
                Ok(Err(e)) => {
                    eprintln!("[DEV] Write err: {} — pausing.", e);
                    break;
                }
                Err(_) => {
                    eprintln!("[DEV] Write timeout — pausing.");
                    break;
                }
            }

            // Pause 2s, check for ESC/27
            tokio::select! {
                line = lines.next_line() => {
                    match line {
                        Ok(Some(l)) => {
                            let t = l.trim();
                            eprintln!("[DEV] Pause cmd: {:?}", t);
                            if t.starts_with('\x1B') {
                                eprintln!("[DEV] ESC — pause.");
                                sending = false;
                            } else if t == "27" {
                                eprintln!("[DEV] Restart — resend.");
                            }
                        }
                        Ok(None) => {
                            eprintln!("[DEV] EOF in pause.");
                            sending = false;
                        }
                        Err(_) => {
                            eprintln!("[DEV] Err in pause.");
                            sending = false;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            }
        }
    }
}

fn open_serial(path: &str) -> SerialStream {
    tokio_serial::SerialStream::open(&tokio_serial::new(path, 115200))
        .unwrap_or_else(|e| panic!("Failed to open {}: {}", path, e))
}
