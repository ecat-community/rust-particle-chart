use leptos::*;

#[component]
pub fn SettingsView() -> impl IntoView {
    view! {
        <div class="settings-container">
            <h2>"Settings"</h2>
            <p class="settings-placeholder">"Application settings will be available here."</p>
        </div>
    }
}
