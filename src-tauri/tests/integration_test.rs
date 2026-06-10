//! Integration test for serial communication using socat virtual serial port pairs.
//!
//! This test simulates a particle counter device and verifies the full data flow:
//! 1. Send start command "27" to the device
//! 2. Device responds with data from data-example.txt
//! 3. Client parses the response and verifies #RAWD blocks are received
//!
//! # Running this test
//!
//! This test requires socat to be running externally with virtual PTY pairs:
//!
//! ```bash
//! # Terminal 1: Start socat
//! socat -d -d pty,raw,echo=0,link=/tmp/ttyV0 pty,raw,echo=0,link=/tmp/ttyV1
//!
//! # Terminal 2: Run the test
//! cd src-tauri && cargo test --test integration_test -- --ignored
//! ```
//!
//! # Platform limitations
//!
//! This test works on Linux with socat-created PTY pairs.
//! On macOS, PTYs don't support termios operations required by tokio-serial,
//! so this test will fail with "Not a typewriter" error.
//! The test is marked #[ignore] by default and should only be run on Linux systems
//! or in CI environments that support proper PTY emulation.

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::SerialPortBuilderExt;

async fn run_simulated_device(port_path: &str) -> tokio_serial::Result<()> {
    let port = tokio_serial::new(port_path, 115200).open_native_async()?;
    let (read_half, mut write_half) = tokio::io::split(port);
    let reader = BufReader::new(read_half);
    let mut lines = reader.lines();

    // Wait for start command "27"
    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) if line.trim() == "27" => break,
                    Ok(None) => return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Disconnected").into()),
                    Err(e) => return Err(e.into()),
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout").into());
            }
        }
    }

    // Send data from data-example.txt
    let data = include_str!("../../data-example.txt");
    for _ in 0..3 {
        write_half.write_all(data.as_bytes()).await?;
        write_half.flush().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[tokio::test]
#[ignore] // Requires socat running externally (Linux only - macOS PTYs don't support termios)
async fn test_serial_data_flow() {
    let client_port = "/tmp/ttyV0";
    let device_port = "/tmp/ttyV1";

    // Spawn simulated device
    let device_handle = tokio::spawn(async move {
        let _ = run_simulated_device(device_port).await;
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Client side
    let port = tokio_serial::new(client_port, 115200)
        .open_native_async()
        .expect("Failed to open client port");

    let (read_half, mut write_half) = tokio::io::split(port);
    let mut reader = BufReader::new(read_half);

    // Send start command
    write_half
        .write_all(b"27\r\n")
        .await
        .expect("Failed to send start");
    write_half.flush().await.expect("Failed to flush");

    // Read and parse looking for #RAWD
    let mut rawd_count = 0;
    let mut line = String::new();
    let timeout = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if line.contains("#RAWD") {
                        rawd_count += 1;
                    }
                }
                Err(_) => break,
            }
            if rawd_count >= 3 {
                break;
            }
        }
    })
    .await;

    assert!(timeout.is_ok(), "Test timed out");
    assert!(
        rawd_count >= 3,
        "Expected at least 3 RAWD blocks, got {}",
        rawd_count
    );

    device_handle.abort();
}
