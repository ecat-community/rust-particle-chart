use crate::tauri_bridge;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SerialConfig(
    on_start: impl Fn(String, u32, u8, u8, String) + 'static,
    on_stop: impl Fn() + 'static,
    #[allow(unused_variables)]
    status: Signal<String>,
    is_running: RwSignal<bool>,
    selected_port: RwSignal<String>,
) -> impl IntoView {
    let ports = create_rw_signal::<Vec<String>>(vec![]);
    let baud_rate = create_rw_signal::<u32>(115200);
    let data_bits = create_rw_signal::<u8>(8);
    let stop_bits = create_rw_signal::<u8>(1);
    let parity = create_rw_signal::<String>("none".to_string());
    let is_refreshing = create_rw_signal::<bool>(false);

    create_effect(move |_| {
        spawn_local(async move {
            refresh_ports(ports, is_refreshing).await;
        });
    });

    let handle_refresh = move |_| {
        spawn_local(async move {
            refresh_ports(ports, is_refreshing).await;
        });
    };

    let handle_start = move |_| {
        let port = selected_port.get();
        if port.is_empty() {
            return;
        }
        on_start(port, baud_rate.get(), data_bits.get(), stop_bits.get(), parity.get());
    };

    let handle_stop = move |_| {
        on_stop();
    };

    let handle_port_select = move |ev| {
        let val = event_target_value(&ev);
        if !val.is_empty() {
            selected_port.set(val);
        }
    };

    view! {
        <div class="sidebar-section">
            <p class="section-title">"Connection"</p>

            <div class="config-field">
                <label>"Port"</label>
                <div class="port-row">
                    <input
                        type="text"
                        placeholder="/dev/ttyUSB0"
                        prop:value=move || selected_port.get()
                        prop:inputmode="url"
                        prop:enterkeyhint="done"
                        prop:autocomplete="off"
                        prop:autocorrect="off"
                        on:input=move |ev| {
                            selected_port.set(event_target_value(&ev));
                        }
                        disabled=move || is_running.get()
                    />
                    <select
                        on:change=handle_port_select
                        disabled=move || is_running.get()
                    >
                        <option value="">"--"</option>
                        {move || {
                            ports.get()
                                .into_iter()
                                .map(|port| {
                                    view! {
                                        <option value=port.clone()>{port}</option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </div>
            </div>

            <div class="config-field">
                <label>"Baud Rate"</label>
                <select
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        baud_rate.set(val.parse().unwrap_or(115200));
                    }
                    disabled=move || is_running.get()
                >
                    <option value="9600" prop:selected=move || baud_rate.get() == 9600>"9600"</option>
                    <option value="19200" prop:selected=move || baud_rate.get() == 19200>"19200"</option>
                    <option value="38400" prop:selected=move || baud_rate.get() == 38400>"38400"</option>
                    <option value="57600" prop:selected=move || baud_rate.get() == 57600>"57600"</option>
                    <option value="115200" prop:selected=move || baud_rate.get() == 115200>"115200"</option>
                    <option value="230400" prop:selected=move || baud_rate.get() == 230400>"230400"</option>
                    <option value="460800" prop:selected=move || baud_rate.get() == 460800>"460800"</option>
                    <option value="921600" prop:selected=move || baud_rate.get() == 921600>"921600"</option>
                </select>
            </div>

            <div class="config-field">
                <label>"Data Bits"</label>
                <select
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        data_bits.set(val.parse().unwrap_or(8));
                    }
                    disabled=move || is_running.get()
                >
                    <option value="5" prop:selected=move || data_bits.get() == 5>"5"</option>
                    <option value="6" prop:selected=move || data_bits.get() == 6>"6"</option>
                    <option value="7" prop:selected=move || data_bits.get() == 7>"7"</option>
                    <option value="8" prop:selected=move || data_bits.get() == 8>"8"</option>
                </select>
            </div>

            <div class="config-field">
                <label>"Stop Bits"</label>
                <select
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        stop_bits.set(val.parse().unwrap_or(1));
                    }
                    disabled=move || is_running.get()
                >
                    <option value="1" prop:selected=move || stop_bits.get() == 1>"1"</option>
                    <option value="2" prop:selected=move || stop_bits.get() == 2>"2"</option>
                </select>
            </div>

            <div class="config-field">
                <label>"Parity"</label>
                <select
                    on:change=move |ev| {
                        parity.set(event_target_value(&ev));
                    }
                    disabled=move || is_running.get()
                >
                    <option value="none" prop:selected=move || parity.get() == "none">"None"</option>
                    <option value="odd" prop:selected=move || parity.get() == "odd">"Odd"</option>
                    <option value="even" prop:selected=move || parity.get() == "even">"Even"</option>
                </select>
            </div>

            <div class="btn-row">
                <button
                    class="btn-start"
                    on:click=handle_start
                    disabled=move || selected_port.get().is_empty() || is_running.get()
                >
                    "Start"
                </button>
                <button
                    class="btn-stop"
                    on:click=handle_stop
                    disabled=move || !is_running.get()
                >
                    "Stop"
                </button>
            </div>

            <div style="margin-top: 8px; text-align: center;">
                <button
                    class="btn-refresh"
                    on:click=handle_refresh
                    disabled=move || is_running.get() || is_refreshing.get()
                >
                    {move || if is_refreshing.get() { "Refreshing..." } else { "Refresh Ports" }}
                </button>
            </div>
        </div>
    }
}

async fn refresh_ports(ports_signal: RwSignal<Vec<String>>, refreshing: RwSignal<bool>) {
    refreshing.set(true);
    match tauri_bridge::list_serial_ports().await {
        Ok(port_list) => {
            ports_signal.set(port_list);
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to list ports: {}", e).into());
        }
    }
    refreshing.set(false);
}
