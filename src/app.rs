use crate::components::{
    channel_chart::ChannelChart,
    serial_config::SerialConfig,
    settings_view::SettingsView,
    top_nav::{Tab, TopNav},
};
use crate::tauri_bridge;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let active_tab = create_rw_signal(Tab::Dashboard);
    let status = create_rw_signal("Ready".to_string());
    let array1_data = create_rw_signal::<Vec<u16>>(vec![0; 256]);
    let array2_data = create_rw_signal::<Vec<u16>>(vec![0; 256]);
    let max_value = create_rw_signal::<u16>(0);
    let is_connected = create_rw_signal::<bool>(false);
    let sample_count = create_rw_signal::<u32>(0);
    let is_running = create_rw_signal::<bool>(false);
    let selected_port = create_rw_signal::<String>(String::new());
    let sidebar_collapsed = create_rw_signal::<bool>(false);

    let toggle_sidebar = move || {
        sidebar_collapsed.update(|c| *c = !*c);
    };

    create_effect(move |_| {
        tauri_bridge::setup_rawd_data_listener(move |payload: tauri_bridge::RawdPayload| {
            array1_data.set(payload.array1.clone());
            array2_data.set(payload.array2.clone());
            max_value.set(payload.max_value);
            status.set("Receiving data".to_string());
            is_connected.set(true);
            sample_count.update(|n| *n += 1);
        });
    });

    let handle_start = move |port_name: String, baud_rate: u32, data_bits: u8, stop_bits: u8, parity: String| {
        status.set(format!("Connecting {} @ {}...", port_name, baud_rate));
        is_running.set(true);

        spawn_local(async move {
            match tauri_bridge::start_capture(port_name, baud_rate, data_bits, stop_bits, parity).await {
                Ok(_) => {
                    status.set("Capture started".to_string());
                }
                Err(e) => {
                    status.set(format!("Error: {}", e));
                    is_connected.set(false);
                    is_running.set(false);
                }
            }
        });
    };

    let handle_stop = move || {
        status.set("Stopping...".to_string());
        is_running.set(false);

        spawn_local(async move {
            match tauri_bridge::stop_capture().await {
                Ok(_) => {
                    status.set("Stopped".to_string());
                    is_connected.set(false);
                    array1_data.set(vec![0; 256]);
                    array2_data.set(vec![0; 256]);
                    max_value.set(0);
                }
                Err(e) => {
                    status.set(format!("Error: {}", e));
                }
            }
        });
    };

    view! {
        <div class="app-layout">
            <TopNav
                active_tab=active_tab
                status=Signal::derive(move || status.get())
                is_connected=Signal::derive(move || is_connected.get())
            />
            <div class="app-body">
                <Show
                    when=move || active_tab.get() == Tab::Dashboard
                    fallback=move || view! {
                        <div class="sidebar">
                            <div class="sidebar-section">
                                <p class="section-title">"Navigation"</p>
                            </div>
                        </div>
                        <SettingsView />
                    }
                >
                    <div class="sidebar-wrapper"
                        class:collapsed=move || sidebar_collapsed.get()
                    >
                        <button
                            class="sidebar-toggle"
                            on:click=move |_| toggle_sidebar()
                            title=move || if sidebar_collapsed.get() { "Expand sidebar" } else { "Collapse sidebar" }
                        >
                            {move || if sidebar_collapsed.get() { "▶" } else { "◀" }}
                        </button>
                        <div class="sidebar">
                            <SerialConfig
                                on_start=handle_start
                                on_stop=handle_stop
                                status=Signal::derive(move || status.get())
                                is_running=is_running
                                selected_port=selected_port
                            />
                            <div class="sidebar-section max-display">
                                <span class="max-label">"Max Value"</span>
                                <span class="max-number">{move || max_value.get()}</span>
                            </div>
                            <div class="stats-row">
                                <span>"Samples"</span>
                                <span class="stats-value">{move || sample_count.get()}</span>
                            </div>
                        </div>
                    </div>
                    <div class="main-content">
                        <div class="charts-grid">
                            <ChannelChart
                                title="Channel 1".to_string()
                                chart_id="chart1".to_string()
                                data=Signal::derive(move || array1_data.get())
                                max_value=Signal::derive(move || max_value.get())
                            />
                            <ChannelChart
                                title="Channel 2".to_string()
                                chart_id="chart2".to_string()
                                data=Signal::derive(move || array2_data.get())
                                max_value=Signal::derive(move || max_value.get())
                            />
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
