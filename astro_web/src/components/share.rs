use crate::i18n::*;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

/// Persisted `f64` input keys that define a world and travel in a share link.
/// Per-planet and per-moon keys are dynamic — see the suffix lists below.
const F64_KEYS: &[&str] = &[
    // star tab
    "star_mass",
    "star_b_mass",
    "binary_separation",
    "binary_eccentricity",
    // globals shared by every planet
    "system_age_gyr",
    "planet_custom_star_mass",
];

/// Persisted `bool` toggle keys included in a share link.
const BOOL_KEYS: &[&str] = &["star_binary_mode", "planet_custom_star"];

/// Per-planet keys look like `planet_{id}_{suffix}`; the id list itself
/// travels as `planet_ids` (comma-separated, never empty).
const PLANET_F64_SUFFIXES: &[&str] = &[
    "mass", "manual_radius", "semi_major", "ecc", "tilt", "peri_long", "albedo", "co2",
    "atmo_mass",
];
const PLANET_BOOL_SUFFIXES: &[&str] = &["use_manual_r"];

/// Per-moon `f64` keys look like `moon_{id}_{suffix}`; the id list itself
/// travels as `moon_ids` (comma-separated, may be empty).
const MOON_F64_SUFFIXES: &[&str] = &["radius", "density", "dist"];

/// Longest accepted id list in a share link — guards against hash-crafted
/// localStorage flooding.
const MAX_SHARED_BODIES: usize = 32;

/// Longest accepted decoded planet/moon name.
const MAX_NAME_LEN: usize = 60;

/// Single-planet keys from links minted before multi-planet support, mapped
/// onto their planet-0 equivalents.
const LEGACY_PLANET_KEYS: &[(&str, &str)] = &[
    ("planet_mass", "planet_0_mass"),
    ("planet_use_manual_r", "planet_0_use_manual_r"),
    ("planet_manual_radius", "planet_0_manual_radius"),
    ("planet_semi_major", "planet_0_semi_major"),
    ("planet_eccentricity", "planet_0_ecc"),
    ("planet_axial_tilt", "planet_0_tilt"),
    ("planet_peri_long", "planet_0_peri_long"),
    ("planet_albedo", "planet_0_albedo"),
    ("planet_co2_fraction", "planet_0_co2"),
    ("planet_atmo_mass", "planet_0_atmo_mass"),
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
        let k = translate_legacy(k);
        let Some(value) = validated_value(k, v) else { continue };
        if storage.set_item(k, &value).is_ok() {
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

/// Maps pre-multi-planet keys to their planet-0 equivalents.
fn translate_legacy(k: &str) -> &str {
    LEGACY_PLANET_KEYS
        .iter()
        .find(|(old, _)| *old == k)
        .map_or(k, |(_, new)| *new)
}

fn is_finite_f64(v: &str) -> bool {
    v.parse::<f64>().map(f64::is_finite).unwrap_or(false)
}

/// A bounded comma-separated list of u32 ids.
fn is_valid_id_list(v: &str, allow_empty: bool) -> bool {
    if v.is_empty() {
        return allow_empty;
    }
    v.split(',').count() <= MAX_SHARED_BODIES && v.split(',').all(|p| p.parse::<u32>().is_ok())
}

/// Percent-decodes a shared name and bounds its length.
fn decode_name(v: &str) -> Option<String> {
    let s: String = js_sys::decode_uri_component(v).ok()?.into();
    (s.chars().count() <= MAX_NAME_LEN).then_some(s)
}

/// Returns the value to store for hash pair `k=v` if the pair is a valid
/// world key, or `None` to skip it. Names come back percent-decoded.
fn validated_value(k: &str, v: &str) -> Option<String> {
    if F64_KEYS.contains(&k) {
        return is_finite_f64(v).then(|| v.to_owned());
    }
    if BOOL_KEYS.contains(&k) {
        return v.parse::<bool>().is_ok().then(|| v.to_owned());
    }
    if k == "planet_ids" {
        return is_valid_id_list(v, false).then(|| v.to_owned());
    }
    if k == "moon_ids" {
        return is_valid_id_list(v, true).then(|| v.to_owned());
    }
    if let Some(rest) = k.strip_prefix("planet_") {
        let (id, suffix) = rest.split_once('_')?;
        if id.parse::<u32>().is_err() {
            return None;
        }
        if PLANET_F64_SUFFIXES.contains(&suffix) {
            return is_finite_f64(v).then(|| v.to_owned());
        }
        if PLANET_BOOL_SUFFIXES.contains(&suffix) {
            return v.parse::<bool>().is_ok().then(|| v.to_owned());
        }
        if suffix == "host" {
            return matches!(v.parse::<u32>(), Ok(h) if h <= 2).then(|| v.to_owned());
        }
        if suffix == "name" {
            return decode_name(v);
        }
        return None;
    }
    if let Some(rest) = k.strip_prefix("moon_") {
        let (id, suffix) = rest.split_once('_')?;
        if id.parse::<u32>().is_err() {
            return None;
        }
        if MOON_F64_SUFFIXES.contains(&suffix) {
            return is_finite_f64(v).then(|| v.to_owned());
        }
        if suffix == "parent" {
            return v.parse::<u32>().is_ok().then(|| v.to_owned());
        }
        if suffix == "name" {
            return decode_name(v);
        }
        return None;
    }
    None
}

/// Build a shareable URL encoding all currently stored world inputs.
fn build_share_url() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let path = location.pathname().ok()?;
    let search = location.search().ok()?;
    let storage = get_storage()?;

    let mut params: Vec<String> = F64_KEYS
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

    // A stored name is user text: percent-encode it for the hash.
    fn push_name(params: &mut Vec<String>, storage: &web_sys::Storage, key: String) {
        if let Some(name) = storage.get_item(&key).ok().flatten() {
            if !name.is_empty() {
                let encoded: String = js_sys::encode_uri_component(&name).into();
                params.push(format!("{key}={encoded}"));
            }
        }
    }

    // Per-planet inputs for every persisted planet id.
    let planet_ids = super::storage::load_planet_ids();
    for id in &planet_ids {
        for suffix in PLANET_F64_SUFFIXES
            .iter()
            .chain(PLANET_BOOL_SUFFIXES)
            .chain(&["host"])
        {
            let key = format!("planet_{id}_{suffix}");
            if let Some(v) = storage.get_item(&key).ok().flatten() {
                params.push(format!("{key}={v}"));
            }
        }
        push_name(&mut params, &storage, format!("planet_{id}_name"));
    }
    params.push(format!(
        "planet_ids={}",
        planet_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    ));

    // Per-moon inputs for every persisted moon id.
    let moon_ids = super::storage::load_moon_ids();
    for id in &moon_ids {
        for suffix in MOON_F64_SUFFIXES {
            let key = format!("moon_{id}_{suffix}");
            if let Some(v) = storage.get_item(&key).ok().flatten() {
                params.push(format!("{key}={v}"));
            }
        }
        let parent_key = format!("moon_{id}_parent");
        if let Some(v) = storage.get_item(&parent_key).ok().flatten() {
            params.push(format!("{parent_key}={v}"));
        }
        push_name(&mut params, &storage, format!("moon_{id}_name"));
    }
    params.push(format!(
        "moon_ids={}",
        moon_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    ));

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
