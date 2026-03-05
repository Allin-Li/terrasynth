use crate::i18n::*;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Star,
    Planet,
    Moon,
}

#[component]
pub fn TabBar(active_tab: RwSignal<Tab>) -> impl IntoView {
    let i18n = use_i18n();

    let btn = move |tab: Tab, icon: &'static str| {
        view! {
            <button
                class=move || {
                    let base = "flex items-center gap-2 px-4 py-2 text-sm font-medium \
                                rounded-lg cursor-pointer";
                    if active_tab.get() == tab {
                        format!("{base} bg-accent/20 text-accent ring-1 ring-accent/30")
                    } else {
                        format!("{base} text-hint hover:text-label hover:bg-card")
                    }
                }
                on:click=move |_| active_tab.set(tab)
            >
                <span class="text-base">{icon}</span>
                {match tab {
                    Tab::Star   => view! { {t!(i18n, tab_star)} }.into_any(),
                    Tab::Planet => view! { {t!(i18n, tab_planet)} }.into_any(),
                    Tab::Moon   => view! { {t!(i18n, tab_moon)} }.into_any(),
                }}
            </button>
        }
    };

    view! {
        <nav class="flex gap-1 p-1 bg-card/60 rounded-xl w-fit border border-edge/50">
            {btn(Tab::Star,   "★")}
            {btn(Tab::Planet, "◉")}
            {btn(Tab::Moon,   "☽")}
        </nav>
    }
}
