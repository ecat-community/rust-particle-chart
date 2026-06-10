use leptos::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Dashboard,
    Settings,
}

#[component]
pub fn TopNav(
    active_tab: RwSignal<Tab>,
    status: Signal<String>,
    is_connected: Signal<bool>,
) -> impl IntoView {
    view! {
        <nav class="top-nav">
            <div class="nav-brand">
                <span class="brand-dot"></span>
                <span class="brand-title">"Particle Chart"</span>
            </div>
            <div class="nav-tabs">
                <button
                    class="nav-tab"
                    class:active=move || active_tab.get() == Tab::Dashboard
                    on:click=move |_| active_tab.set(Tab::Dashboard)
                >
                    "Dashboard"
                </button>
                <button
                    class="nav-tab"
                    class:active=move || active_tab.get() == Tab::Settings
                    on:click=move |_| active_tab.set(Tab::Settings)
                >
                    "Settings"
                </button>
            </div>
            <div class="nav-status">
                <span class="status-indicator" class:connected=move || is_connected.get()></span>
                <span>{move || status.get()}</span>
            </div>
        </nav>
    }
}
