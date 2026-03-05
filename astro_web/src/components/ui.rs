use leptos::prelude::*;

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
        <div class="flex justify-between items-baseline gap-4 py-2.5 px-3
                    border-b border-divider/30 last:border-0
                    rounded hover:bg-edge/20">
            <span class="text-label text-sm flex items-center gap-1">
                {label.run()}
                {hint.map(|h| view! { <InfoHint text=h /> })}
            </span>
            <span class="text-heading text-sm font-mono tabular-nums text-right">
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
        <div class="flex justify-between items-baseline gap-4 py-2.5 px-3
                    border-b border-divider/30 last:border-0
                    rounded hover:bg-edge/20">
            <span class="text-label text-sm flex items-center gap-1">
                {label.run()}
                {hint.map(|h| view! { <InfoHint text=h /> })}
            </span>
            <span class=move || {
                if value.get() {
                    "text-xs font-semibold px-2.5 py-0.5 rounded-full \
                     bg-ok/15 text-ok ring-1 ring-ok/25"
                } else {
                    "text-xs font-semibold px-2.5 py-0.5 rounded-full \
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

/// A small ⓘ button that toggles a popover with hint text.
#[component]
pub fn InfoHint(#[prop(into)] text: ViewFn) -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <span class="relative inline-flex">
            <button
                type="button"
                class="text-hint hover:text-accent cursor-pointer
                       w-4 h-4 flex items-center justify-center
                       rounded-full text-[11px] leading-none
                       hover:bg-accent/10 transition-colors"
                on:click=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    open.update(|v| *v = !*v);
                }
            >
                "ⓘ"
            </button>
            <Show when=move || open.get()>
                // Invisible overlay to close on outside click
                <div
                    class="fixed inset-0 z-40"
                    on:click=move |_| open.set(false)
                />
                // Popover bubble
                <div class="absolute left-1/2 -translate-x-1/2 top-full mt-2 z-50
                            w-56 p-3 rounded-xl
                            bg-card border border-edge shadow-xl shadow-black/40
                            text-[12px] leading-relaxed text-label">
                    // Arrow
                    <div class="absolute left-1/2 -translate-x-1/2 -top-1.5
                                w-3 h-3 rotate-45
                                bg-card border-l border-t border-edge" />
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
                type="number"
                step=step
                prop:value=move || value.get().to_string()
                class="bg-inset border border-edge rounded-lg
                       px-3 py-2 text-heading text-sm font-mono
                       outline-none
                       focus:border-accent focus:ring-1 focus:ring-accent/40
                       hover:border-divider
                       placeholder:text-hint w-full"
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                        value.set(v);
                    }
                }
            />
        </div>
    }
}
