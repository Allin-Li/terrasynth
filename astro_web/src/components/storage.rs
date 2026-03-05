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
