//! Tiny key/value persistence.
//!
//! The game ships as WebAssembly on GitHub Pages, so "persistent" means the
//! browser's `localStorage`. Native builds keep values in memory only for the
//! session — a desktop save file would be dead weight for a target nobody
//! distributes.
//!
//! A best score that survives a reload is what turns a session into a series:
//! without a stored number there is nothing to beat, and the next run has no
//! stake.

#[cfg(target_arch = "wasm32")]
pub fn load(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn save(key: &str, value: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    // Storage can be unavailable (private mode, quota). Losing a best score is
    // not worth interrupting play over.
    let _ = storage.set_item(key, value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load(_key: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(_key: &str, _value: &str) {}
