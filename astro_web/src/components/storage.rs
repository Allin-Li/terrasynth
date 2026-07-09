use leptos::prelude::*;

fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Creates an `RwSignal<f64>` backed by `localStorage[key]`.
/// Reads the stored value on first use; writes back on every change.
pub fn ls_f64(key: &'static str, default: f64) -> RwSignal<f64> {
    let initial = get_storage()
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(key, &sig.get().to_string());
        }
    });
    sig
}

/// Like `ls_f64` but with a dynamic (owned) key string.
pub fn ls_f64_dyn(key: String, default: f64) -> RwSignal<f64> {
    let initial = get_storage()
        .and_then(|s| s.get_item(&key).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    let key_clone = key.clone();
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(&key_clone, &sig.get().to_string());
        }
    });
    sig
}

/// Like `ls_bool` but with a dynamic (owned) key string.
pub fn ls_bool_dyn(key: String, default: bool) -> RwSignal<bool> {
    let initial = get_storage()
        .and_then(|s| s.get_item(&key).ok().flatten())
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    let key_clone = key.clone();
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(&key_clone, &sig.get().to_string());
        }
    });
    sig
}

/// Creates an `RwSignal<String>` backed by `localStorage[key]`.
pub fn ls_string_dyn(key: String, default: String) -> RwSignal<String> {
    let initial = get_storage()
        .and_then(|s| s.get_item(&key).ok().flatten())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    let key_clone = key.clone();
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(&key_clone, &sig.get());
        }
    });
    sig
}

/// Creates an `RwSignal<u32>` backed by `localStorage[key]`.
pub fn ls_u32_dyn(key: String, default: u32) -> RwSignal<u32> {
    let initial = get_storage()
        .and_then(|s| s.get_item(&key).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    let key_clone = key.clone();
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(&key_clone, &sig.get().to_string());
        }
    });
    sig
}

/// Raw (non-reactive) localStorage read.
pub fn raw_get(key: &str) -> Option<String> {
    get_storage().and_then(|s| s.get_item(key).ok().flatten())
}

/// Loads the persisted moon id list (`localStorage["moon_ids"]`,
/// comma-separated). A missing key means the list was never saved — fall back
/// to the single default moon; an empty string means "all moons deleted".
pub fn load_moon_ids() -> Vec<u32> {
    match get_storage().and_then(|s| s.get_item("moon_ids").ok().flatten()) {
        Some(v) => v.split(',').filter_map(|p| p.parse().ok()).collect(),
        None => vec![0],
    }
}

/// Persists the moon id list as `localStorage["moon_ids"]`.
pub fn save_moon_ids(ids: &[u32]) {
    if let Some(s) = get_storage() {
        let joined = ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        let _ = s.set_item("moon_ids", &joined);
    }
}

/// Loads the persisted planet id list. Unlike moons, a system always has at
/// least one planet, so an empty or missing list falls back to `[0]`.
pub fn load_planet_ids() -> Vec<u32> {
    let ids: Vec<u32> = get_storage()
        .and_then(|s| s.get_item("planet_ids").ok().flatten())
        .map(|v| v.split(',').filter_map(|p| p.parse().ok()).collect())
        .unwrap_or_default();
    if ids.is_empty() { vec![0] } else { ids }
}

/// Persists the planet id list as `localStorage["planet_ids"]`.
pub fn save_planet_ids(ids: &[u32]) {
    if let Some(s) = get_storage() {
        let joined = ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        let _ = s.set_item("planet_ids", &joined);
    }
}

/// Points moons orbiting planet `from` at planet `to` (used when a planet is
/// deleted). Writes storage directly: the Moon tab is unmounted at that point
/// and will pick the values up on next mount.
pub fn reassign_moon_parents(from: u32, to: u32) {
    let Some(s) = get_storage() else { return };
    for m in load_moon_ids() {
        let key = format!("moon_{m}_parent");
        let parent = s.get_item(&key).ok().flatten().and_then(|v| v.parse::<u32>().ok());
        if parent == Some(from) {
            let _ = s.set_item(&key, &to.to_string());
        }
    }
}

/// Creates an `RwSignal<bool>` backed by `localStorage[key]`.
pub fn ls_bool(key: &'static str, default: bool) -> RwSignal<bool> {
    let initial = get_storage()
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(default);

    let sig = RwSignal::new(initial);
    Effect::new(move |_| {
        if let Some(s) = get_storage() {
            let _ = s.set_item(key, &sig.get().to_string());
        }
    });
    sig
}
