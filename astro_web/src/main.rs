mod components;

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

use components::{MoonTab, PlanetTab, StarTab, Tab, TabBar};
use i18n::*;
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();
    let active_tab = RwSignal::new(Tab::Star);

    view! {
        <I18nContextProvider>
            <AppInner active_tab=active_tab />
        </I18nContextProvider>
    }
}

#[component]
fn AppInner(active_tab: RwSignal<Tab>) -> impl IntoView {
    let i18n = use_i18n();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            Locale::en => Locale::ru,
            Locale::ru => Locale::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <div class="min-h-dvh bg-base text-heading overflow-x-auto">
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-8 flex flex-col gap-8 min-w-[320px]">

                <header class="flex items-start justify-between gap-4">
                    <div class="flex flex-col gap-1">
                        <h1 class="text-3xl sm:text-4xl font-bold tracking-tight
                                   bg-gradient-to-r from-accent to-accent-alt
                                   bg-clip-text text-transparent">
                            {t!(i18n, app_title)}
                        </h1>
                        <p class="text-sm text-hint">
                            {t!(i18n, app_subtitle)}
                        </p>
                    </div>
                    <button
                        class="text-[11px] font-semibold px-3 py-1.5 rounded-lg cursor-pointer
                               bg-edge/40 text-hint ring-1 ring-edge
                               hover:text-label hover:bg-edge/60 mt-2"
                        on:click=on_switch
                    >
                        {t!(i18n, switch_lang)}
                    </button>
                </header>

                <TabBar active_tab=active_tab />

                <main>
                    {move || match active_tab.get() {
                        Tab::Star   => view! { <StarTab /> }.into_any(),
                        Tab::Planet => view! { <PlanetTab /> }.into_any(),
                        Tab::Moon   => view! { <MoonTab /> }.into_any(),
                    }}
                </main>
            </div>
        </div>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
