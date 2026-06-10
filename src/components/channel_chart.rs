use charming::element::{
    AxisLabel, AxisLine, AxisPointer, AxisType, LineStyle, LineStyleType, NameLocation,
    SplitLine, Tooltip, Trigger,
};
use charming::series::Bar;
use charming::{Chart, WasmRenderer};
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ChannelChart(
    title: String,
    chart_id: String,
    data: Signal<Vec<u16>>,
    max_value: Signal<u16>,
) -> impl IntoView {
    let title_clone = title.clone();
    let chart_id_clone = chart_id.clone();

    create_effect(move |_| {
        let data_values = data.get();
        let max_val = max_value.get();

        if data_values.is_empty() {
            return;
        }

        let chart = build_chart(&title_clone, &data_values, max_val);
        render_chart(&chart_id_clone, &chart);
        setup_resize_observer(&chart_id_clone);
    });

    view! {
        <div class="channel-chart">
            <div id=chart_id class="chart-container"></div>
        </div>
    }
}

fn build_chart(title: &str, data_values: &[u16], max_val: u16) -> Chart {
    let categories: Vec<String> = (0..256).map(|i| i.to_string()).collect();
    let values: Vec<f64> = data_values.iter().map(|v| *v as f64).collect();

    let is_ch1 = title.contains("1");
    let bar_color = if is_ch1 { "#42a5f5" } else { "#ef5350" };

    Chart::new()
        .title(
            charming::component::Title::new()
                .text(&format!("{} — Particle Size Distribution", title))
                .text_style(
                    charming::element::TextStyle::new()
                        .color("#8888aa")
                        .font_size(12)
                        .font_weight("normal"),
                )
                .left("4%")
                .top("2%"),
        )
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Axis)
                .axis_pointer(AxisPointer::new()),
        )
        .x_axis(
            charming::component::Axis::new()
                .type_(AxisType::Category)
                .data(categories)
                .name("Channel Number")
                .name_location(NameLocation::Middle)
                .name_gap(30)
                .name_text_style(
                    charming::element::TextStyle::new().color("#8888aa").font_size(11),
                )
                .axis_label(AxisLabel::new().color("#666688").font_size(10).interval(31.0))
                .axis_line(AxisLine::new())
                .split_line(
                    SplitLine::new().line_style(
                        LineStyle::new().color("#2a2a4a").type_(LineStyleType::Dashed),
                    ),
                ),
        )
        .y_axis(
            charming::component::Axis::new()
                .type_(AxisType::Value)
                .name("Counts")
                .name_text_style(
                    charming::element::TextStyle::new().color("#8888aa").font_size(11),
                )
                .max(if max_val > 0 {
                    max_val as f64
                } else {
                    100.0
                })
                .axis_label(AxisLabel::new().color("#666688").font_size(10))
                .axis_line(AxisLine::new())
                .split_line(
                    SplitLine::new().line_style(
                        LineStyle::new().color("#2a2a4a").type_(LineStyleType::Dashed),
                    ),
                ),
        )
        .series(
            Bar::new()
                .data(values)
                .item_style(charming::element::ItemStyle::new().color(bar_color))
                .bar_width("1.5"),
        )
        .grid(
            charming::component::Grid::new()
                .left("6%")
                .right("3%")
                .top("14%")
                .bottom("12%")
                .contain_label(false),
        )
        .background_color("#1e1e38")
}

fn render_chart(chart_id: &str, chart: &Chart) {
    let container = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(chart_id));

    let (width, height) = match container {
        Some(ref el) => {
            let html_el: &web_sys::HtmlElement = el.unchecked_ref();
            let w = html_el.offset_width() as u32;
            let h = html_el.offset_height() as u32;
            if w > 0 && h > 0 { (w, h) } else { (800, 300) }
        }
        None => (800, 300),
    };

    let renderer = WasmRenderer::new(width, height);
    if let Err(e) = renderer.render(chart_id, chart) {
        web_sys::console::error_1(&format!("Failed to render chart {}: {:?}", chart_id, e).into());
    }
}

/// Resize chart using echarts resize() with explicit dimensions + debounce.
fn resize_chart(chart_id: &str) {
    let js = format!(
        r#"(() => {{
            const key = '__chart_resize_timer_' + '{cid}';
            if (window[key]) clearTimeout(window[key]);
            window[key] = setTimeout(() => {{
                const el = document.getElementById('{cid}');
                if (!el) return;
                const inst = echarts.getInstanceByDom(el);
                if (inst) inst.resize({{ width: el.offsetWidth, height: el.offsetHeight }});
            }}, 150);
        }})()"#,
        cid = chart_id
    );
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Function::new_no_args(&js).call0(&window);
    }
}

fn setup_resize_observer(chart_id: &str) {
    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };
    let container = match document.get_element_by_id(chart_id) {
        Some(el) => el,
        None => return,
    };

    let chart_id_cb = chart_id.to_string();

    let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |_entries: js_sys::Array| {
        resize_chart(&chart_id_cb);
    });

    let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref())
        .expect("Failed to create ResizeObserver");

    observer.observe(&container);

    // Store to prevent GC
    let window = web_sys::window().unwrap();
    let key = format!("__resize_observer_{}", chart_id);
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("observer"), &observer).ok();
    js_sys::Reflect::set(&obj, &JsValue::from_str("closure"), &callback.into_js_value()).ok();
    js_sys::Reflect::set(&window, &JsValue::from_str(&key), &obj).ok();
}
