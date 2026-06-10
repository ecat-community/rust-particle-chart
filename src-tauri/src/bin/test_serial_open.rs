use serialport::SerialPort;

fn main() {
    let path = std::env::args().nth(1).unwrap_or("/tmp/ttyV0".to_string());
    println!("Testing serialport::new({}).open()...", path);
    match serialport::new(&path, 115200)
        .timeout(std::time::Duration::from_secs(1))
        .open()
    {
        Ok(port) => {
            println!("✅ Opened OK!");
            println!("   name: {:?}", port.name());
        }
        Err(e) => println!("❌ Failed: {:?}", e),
    }
}
