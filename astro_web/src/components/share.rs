use crate::i18n::*;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

/// Persisted `f64` input keys that define a world and travel in a share link.
const F64_KEYS: &[&str] = &[
    // star tab
    "star_mass",
    "star_b_mass",
    "binary_separation",
    "binary_eccentricity",
    // planet tab
    "planet_mass",
    "planet_manual_radius",
    "planet_semi_major",
    "planet_eccentricity",
    "planet_axial_tilt",
    "system_age_gyr",
    "planet_custom_star_mass",
    "planet_albedo",
    "planet_co2_fraction",
    "planet_atmo_mass",
    // moon tab (custom overrides + the always-present first moon)
    "moon_planet_mass",
    "moon_planet_radius",
    "moon_planet_density",
    "moon_planet_orb_a",
    "moon_star_mass",
    "moon_0_radius",
    "moon_0_density",
    "moon_0_dist",
];

/// Persisted `bool` toggle keys included in a share link.
const BOOL_KEYS: &[&str] = &[
    "star_binary_mode",
    "binary_p_type",
    "planet_use_manual_r",
    "planet_custom_star",
    "moon_custom_planet",
];

fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Import world state from the URL hash (`#key=value&…`) into localStorage.
///
/// Must run before any `ls_f64`/`ls_bool` signal is created so the signals
/// pick up the imported values. Unknown keys and malformed values are
/// ignored; on success the hash is removed from the address bar.
pub fn import_from_hash() {
    let Some(window) = web_sys::window() else { return };
    let location = window.location();
    let Ok(hash) = location.hash() else { return };
    let Some(pairs) = hash.strip_prefix('#') else { return };
    if pairs.is_empty() {
        return;
    }
    let Some(storage) = get_storage() else { return };

    let mut imported = false;
    for pair in pairs.split('&') {
        let Some((k, v)) = pair.split_once('=') else { continue };
        let valid = (F64_KEYS.contains(&k)
            && v.parse::<f64>().map(f64::is_finite).unwrap_or(false))
            || (BOOL_KEYS.contains(&k) && v.parse::<bool>().is_ok());
        if valid && storage.set_item(k, v).is_ok() {
            imported = true;
        }
    }

    // Drop the hash so further local edits aren't mistaken for the shared state.
    if imported {
        if let (Ok(history), Ok(path), Ok(search)) =
            (window.history(), location.pathname(), location.search())
        {
            let _ = history.replace_state_with_url(
                &JsValue::NULL,
                "",
                Some(&format!("{path}{search}")),
            );
        }
    }
}

/// Build a shareable URL encoding all currently stored world inputs.
fn build_share_url() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let path = location.pathname().ok()?;
    let search = location.search().ok()?;
    let storage = get_storage()?;

    let params: Vec<String> = F64_KEYS
        .iter()
        .chain(BOOL_KEYS)
        .filter_map(|k| {
            storage
                .get_item(k)
                .ok()
                .flatten()
                .map(|v| format!("{k}={v}"))
        })
        .collect();

    Some(format!("{origin}{path}{search}#{}", params.join("&")))
}

/// Header button that opens a modal with a copyable share link.
#[component]
pub fn ShareButton() -> impl IntoView {
    let i18n = use_i18n();
    let share_url: RwSignal<Option<String>> = RwSignal::new(None);

    view! {
        <button
            class="text-[11px] font-semibold px-3 py-1.5 rounded-lg cursor-pointer
                   bg-accent/15 text-accent ring-1 ring-accent/20
                   hover:bg-accent/25 mt-2"
            on:click=move |_| share_url.set(build_share_url())
        >
            {t!(i18n, share_link)}
        </button>

        <Show when=move || share_url.get().is_some()>
            <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50
                        flex items-center justify-center"
                 on:click=move |_| share_url.set(None)>
                <div class="bg-card border border-edge rounded-2xl p-6
                            w-full max-w-2xl mx-4 shadow-2xl shadow-black/50"
                     on:click=move |ev| ev.stop_propagation()>
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-sm font-medium text-label">
                            {t!(i18n, share_modal_title)}
                        </h3>
                        <button
                            class="text-hint hover:text-heading cursor-pointer
                                   hover:bg-edge/40 rounded-lg
                                   w-7 h-7 flex items-center justify-center"
                            on:click=move |_| share_url.set(None)
                        >
                            "✕"
                        </button>
                    </div>
                    <textarea
                        class="w-full h-28 bg-inset border border-edge
                               rounded-lg p-4 text-label text-xs font-mono
                               outline-none resize-none break-all
                               focus:ring-1 focus:ring-accent/40 focus:border-accent"
                        readonly
                        prop:value=move || share_url.get().unwrap_or_default()
                        on:click=move |ev| {
                            if let Some(t) = ev
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                            {
                                t.select();
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}
