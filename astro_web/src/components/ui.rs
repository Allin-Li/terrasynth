use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Keep only digits, dots, commas, and leading minus — strip everything else.
pub fn filter_numeric(s: &str) -> String {
    s.chars()
        .enumerate()
        .filter(|(i, c)| c.is_ascii_digit() || *c == '.' || *c == ',' || (*c == '-' && *i == 0))
        .map(|(_, c)| c)
        .collect()
}

/// Format a `Result<f64, E>` as a string with `precision` decimal places,
/// or the error message if `Err`.
pub fn fmt_result<E: std::fmt::Display>(r: Result<f64, E>, precision: usize) -> String {
    match r {
        Ok(v) => format!("{v:.precision$}"),
        Err(e) => e.to_string(),
    }
}

/// A single label/value row used throughout all result panels.
#[component]
pub fn ResultRow(
    #[prop(into)] label: ViewFn,
    #[prop(optional, into)] hint: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex justify-between items-start gap-4 py-2.5 px-3
                    border-b border-divider/30 last:border-0
                    rounded hover:bg-edge/20">
            <span class="text-label text-sm flex items-center gap-1 flex-1 min-w-0 flex-wrap">
                {label.run()}
                {hint.map(|h| view! { <InfoHint text=h /> })}
            </span>
            <span class="text-heading text-sm font-mono tabular-nums text-right shrink-0">
                {children()}
            </span>
        </div>
    }
}

/// A label/value row where the value is a boolean rendered as pill badge.
#[component]
pub fn BoolRow(
    #[prop(into)] label: ViewFn,
    #[prop(into)] value: Signal<bool>,
    #[prop(optional, into)] hint: Option<ViewFn>,
) -> impl IntoView {
    let i18n = crate::i18n::use_i18n();
    view! {
        <div class="flex justify-between items-start gap-4 py-2.5 px-3
                    border-b border-divider/30 last:border-0
                    rounded hover:bg-edge/20">
            <span class="text-label text-sm flex items-center gap-1 flex-1 min-w-0 flex-wrap">
                {label.run()}
                {hint.map(|h| view! { <InfoHint text=h /> })}
            </span>
            <span class=move || {
                if value.get() {
                    "text-xs font-semibold px-2.5 py-0.5 rounded-full shrink-0 \
                     bg-ok/15 text-ok ring-1 ring-ok/25"
                } else {
                    "text-xs font-semibold px-2.5 py-0.5 rounded-full shrink-0 \
                     bg-err/15 text-err ring-1 ring-err/25"
                }
            }>
                {move || if value.get() {
                    crate::i18n::t_string!(i18n, yes_label)
                } else {
                    crate::i18n::t_string!(i18n, no_label)
                }}
            </span>
        </div>
    }
}

/// A small section divider used inside result panels.
#[component]
pub fn SectionHeader(#[prop(into)] label: ViewFn) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 pt-5 pb-2">
            <div class="h-px flex-1 bg-gradient-to-r from-divider/60 to-transparent" />
            <span class="text-[11px] font-semibold text-hint uppercase tracking-widest">
                {label.run()}
            </span>
            <div class="h-px flex-1 bg-gradient-to-l from-divider/60 to-transparent" />
        </div>
    }
}

/// Popover position computed from the button's bounding rect.
#[derive(Clone, Copy, Default)]
struct PopoverPos {
    /// Fixed `left` for the popover box (px)
    left: f64,
    /// Fixed `top` for the popover box (px)
    top: f64,
    /// `left` for the arrow inside the popover box (px, from popover left edge)
    arrow_left: f64,
}

/// A small ⓘ button that toggles a popover with hint text.
/// The popover is rendered with `position: fixed` and its horizontal position
/// is clamped to the viewport so it never clips on mobile.
#[component]
pub fn InfoHint(#[prop(into)] text: ViewFn) -> impl IntoView {
    let open = RwSignal::new(false);
    let pos = RwSignal::new(PopoverPos::default());
    let button_ref = NodeRef::<leptos::html::Button>::new();

    // Close the popover on scroll so it doesn't float away from its button.
    Effect::new(move |_| {
        if !open.get() { return; }
        let close = Closure::<dyn Fn()>::new(move || open.set(false));
        if let Some(win) = web_sys::window() {
            win.add_event_listener_with_callback("scroll", close.as_ref().unchecked_ref()).ok();
            let f = close.as_ref().unchecked_ref::<js_sys::Function>().clone();
            close.forget();
            on_cleanup(move || {
                if let Some(w) = web_sys::window() {
                    w.remove_event_listener_with_callback("scroll", &f).ok();
                }
            });
        } else {
            close.forget();
        }
    });

    let toggle = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        if !open.get() {
            if let Some(btn) = button_ref.get() {
                let rect = btn.get_bounding_client_rect();
                let btn_center_x = rect.left() + rect.width() / 2.0;
                let btn_bottom = rect.bottom();

                const POPOVER_W: f64 = 224.0; // w-56 = 14rem = 224px
                const MARGIN: f64 = 8.0;

                let vw = web_sys::window()
                    .and_then(|w| w.inner_width().ok())
                    .and_then(|v| v.as_f64())
                    .unwrap_or(375.0);

                let natural_left = btn_center_x - POPOVER_W / 2.0;
                let clamped_left = natural_left
                    .max(MARGIN)
                    .min(vw - POPOVER_W - MARGIN);

                pos.set(PopoverPos {
                    left: clamped_left,
                    top: btn_bottom + 8.0,
                    // how far the button centre is from the popover's left edge
                    arrow_left: btn_center_x - clamped_left,
                });
            }
        }

        open.update(|v| *v = !*v);
    };

    view! {
        <span class="relative inline-flex">
            <button
                node_ref=button_ref
                type="button"
                class="text-hint hover:text-accent cursor-pointer
                       w-4 h-4 flex items-center justify-center
                       rounded-full text-[11px] leading-none
                       hover:bg-accent/10 transition-colors"
                on:click=toggle
            >
                "ⓘ"
            </button>
            <Show when=move || open.get()>
                // Invisible overlay to close on outside click
                <div
                    class="fixed inset-0 z-40"
                    on:click=move |_| open.set(false)
                />
                // Popover bubble — fixed position, clamped to viewport
                <div
                    class="fixed z-50 w-56 p-3 rounded-xl
                            bg-card border border-edge shadow-xl shadow-black/40
                            text-[12px] leading-relaxed text-label"
                    style=move || {
                        let p = pos.get();
                        format!("left:{}px;top:{}px", p.left, p.top)
                    }
                >
                    // Arrow — centered on the button, not on the popover
                    <div
                        class="absolute w-3 h-3 bg-card border-l border-t border-edge"
                        style=move || {
                            let p = pos.get();
                            // arrow_left is the button centre from popover left edge;
                            // subtract half the arrow width (6px) to centre it
                            format!(
                                "left:{}px;top:-6px;transform:rotate(45deg)",
                                p.arrow_left - 6.0
                            )
                        }
                    />
                    <div class="relative z-10">{text.run()}</div>
                </div>
            </Show>
        </span>
    }
}

/// A labelled numeric input bound to an `RwSignal<f64>`.
#[component]
pub fn NumberInput(
    #[prop(into)] label: ViewFn,
    value: RwSignal<f64>,
    #[prop(optional)] unit: Option<&'static str>,
    #[prop(optional, into)] hint: Option<ViewFn>,
    #[prop(default = "any")] step: &'static str,
) -> impl IntoView {
    let _ = step; // kept for API compat, not used with type="text"

    // Local string signal so intermediate input like "1." or "1," isn't clobbered.
    let text = RwSignal::new(value.get().to_string());

    // Sync external value changes → text (but don't overwrite when text already
    // represents the same number, e.g. "1." and "1," both parse to 1.0).
    Effect::new(move |_| {
        let v = value.get();
        let current = text.get_untracked();
        let current_parsed = current.replace(',', ".").parse::<f64>().ok();
        if current_parsed != Some(v) {
            text.set(v.to_string());
        }
    });

    view! {
        <div class="flex flex-col gap-1.5">
            <div class="flex items-baseline justify-between">
                <span class="text-xs font-medium text-label flex items-center gap-1">
                    {label.run()}
                    {hint.map(|h| view! { <InfoHint text=h /> })}
                </span>
                {unit.map(|u| view! {
                    <span class="text-[10px] font-mono text-hint
                                 bg-edge/40 px-1.5 py-0.5 rounded">{u}</span>
                })}
            </div>
            <input
                type="text"
                inputmode="decimal"
                prop:value=move || text.get()
                class="bg-inset border border-edge rounded-lg
                       px-3 py-2 text-heading text-sm font-mono
                       outline-none
                       focus:border-accent focus:ring-1 focus:ring-accent/40
                       hover:border-divider
                       placeholder:text-hint w-full"
                on:input=move |ev| {
                    let filtered = filter_numeric(&event_target_value(&ev));
                    text.set(filtered.clone());
                    let sanitized = filtered.replace(',', ".");
                    if let Ok(v) = sanitized.parse::<f64>() {
                        value.set(v);
                    }
                }
            />
        </div>
    }
}
