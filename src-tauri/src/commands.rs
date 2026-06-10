use crate::protocol::rawd::RawdData;
use crate::serial::{config::SerialConfig, manager::SerialManager};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub struct AppState {
    pub manager: Mutex<Option<SerialManager>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RawdPayload {
    pub array1: Vec<u16>,
    pub array2: Vec<u16>,
    pub max_value: u16,
}

impl From<RawdData> for RawdPayload {
    fn from(data: RawdData) -> Self {
        let max_value = data.max_value();
        Self {
            array1: data.array1.to_vec(),
            array2: data.array2.to_vec(),
            max_value,
        }
    }
}

#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<String>, String> {
    crate::serial::manager::list_ports()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn start_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    port_name: String,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
) -> Result<(), String> {
    eprintln!(
        "[CMD] start_capture called: port={}, baud={}",
        port_name, baud_rate
    );

    // Stop any existing manager
    {
        let mut guard = state.manager.lock().unwrap();
        if let Some(old_mgr) = guard.as_mut() {
            eprintln!("[CMD] Stopping existing manager");
            let _ = old_mgr.stop();
        }
        *guard = None;
    }

    // Give old task time to exit and release file descriptors.
    // On PTY the device keeps sending into the void until the new
    // reader opens, at which point the pending write_all unblocks
    // and fresh data starts flowing.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let config = SerialConfig {
        port_name,
        baud_rate,
        data_bits,
        stop_bits,
        parity,
    };

    let mut mgr = SerialManager::new(config);
    let app_handle = app.clone();

    mgr.start(move |data: RawdData| {
        let payload = RawdPayload::from(data);
        eprintln!("[EMIT] rawd-data: max={}", payload.max_value);
        app_handle.emit("rawd-data", &payload).ok();
    })?;

    *state.manager.lock().unwrap() = Some(mgr);
    eprintln!("[CMD] start_capture: SerialManager started OK");
    Ok(())
}

#[tauri::command]
pub fn stop_capture(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.manager.lock().unwrap();
    if let Some(mgr) = guard.as_mut() {
        mgr.stop()?;
    }
    *guard = None;
    Ok(())
}
