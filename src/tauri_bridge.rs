use js_sys::{JsString, Object, Reflect};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[derive(Debug, Clone, Deserialize)]
pub struct RawdPayload {
    pub array1: Vec<u16>,
    pub array2: Vec<u16>,
    pub max_value: u16,
}

// Tauri invoke bindings
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    fn listen(eventName: &str, handler: &js_sys::Function) -> JsValue;
}

pub async fn list_serial_ports() -> Result<Vec<String>, String> {
    let result = invoke("list_serial_ports", JsValue::NULL).await;

    if result.is_undefined() {
        return Err("Failed to list serial ports".to_string());
    }

    // Try parsing as Vec<String> first
    match serde_wasm_bindgen::from_value::<Vec<String>>(result.clone()) {
        Ok(parsed) => Ok(parsed),
        Err(_) => {
            // Try parsing as object with error
            if let Ok(obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result) {
                if let Some(err) = obj.get("error").and_then(|e| e.as_str()) {
                    return Err(err.to_string());
                }
            }
            Err("Failed to parse serial ports response".to_string())
        }
    }
}

pub async fn start_capture(
    port_name: String,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
) -> Result<(), String> {
    // Create arguments object
    let args = Object::new().unchecked_into();

    Reflect::set(&args, &"port_name".into(), &JsString::from(port_name)).map_err(|e| format!("Failed to set port_name: {:?}", e))?;
    Reflect::set(&args, &"baud_rate".into(), &(baud_rate as f64).into()).map_err(|e| format!("Failed to set baud_rate: {:?}", e))?;
    Reflect::set(&args, &"data_bits".into(), &(data_bits as f64).into()).map_err(|e| format!("Failed to set data_bits: {:?}", e))?;
    Reflect::set(&args, &"stop_bits".into(), &(stop_bits as f64).into()).map_err(|e| format!("Failed to set stop_bits: {:?}", e))?;
    Reflect::set(&args, &"parity".into(), &JsString::from(parity)).map_err(|e| format!("Failed to set parity: {:?}", e))?;

    let result = invoke("start_capture", args.into()).await;

    // Check for error response
    if let Ok(obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result.clone()) {
        if let Some(err) = obj.get("error").and_then(|e| e.as_str()) {
            return Err(err.to_string());
        }
    }

    Ok(())
}

pub async fn stop_capture() -> Result<(), String> {
    let result = invoke("stop_capture", JsValue::NULL).await;

    // Check for error response
    if let Ok(obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result.clone()) {
        if let Some(err) = obj.get("error").and_then(|e| e.as_str()) {
            return Err(err.to_string());
        }
    }

    Ok(())
}

pub fn setup_rawd_data_listener(callback: impl Fn(RawdPayload) + 'static) {
    web_sys::console::log_1(&"[FRONTEND] Setting up rawd-data listener".into());

    let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        // Tauri 2 listen() callback receives { event, id, payload }
        // We need to extract .payload from the event object
        web_sys::console::log_1(&"[FRONTEND] rawd-data event fired".into());

        let payload_value = Reflect::get(&event, &JsString::from("payload"))
            .unwrap_or(JsValue::UNDEFINED);

        if payload_value.is_undefined() {
            web_sys::console::error_1(&"[FRONTEND] rawd-data event has no payload field".into());
            return;
        }

        if let Ok(payload) = serde_wasm_bindgen::from_value::<RawdPayload>(payload_value) {
            web_sys::console::log_1(&format!("[FRONTEND] Parsed payload: array1[0..5]={:?}, max={}", &payload.array1[0..5], payload.max_value).into());
            callback(payload);
        } else {
            web_sys::console::error_1(&"[FRONTEND] Failed to parse rawd-data payload".into());
        }
    });

    let handler = closure.as_ref().unchecked_ref::<js_sys::Function>();

    let _ = listen("rawd-data", handler);

    // Leak the closure to keep it alive
    closure.forget();
}

pub fn is_tauri_available() -> bool {
    if let Some(win) = window() {
        if let Ok(tauri_obj) = Reflect::get(&win, &JsString::from("__TAURI__")) {
            return !tauri_obj.is_undefined();
        }
    }
    false
}
