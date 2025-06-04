#[cfg(target_arch = "wasm32")]
pub(crate) fn is_webgpu_or_native() -> bool {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        navigator.gpu().is_object()
    } else {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn is_webgpu_or_native() -> bool {
    true
}
