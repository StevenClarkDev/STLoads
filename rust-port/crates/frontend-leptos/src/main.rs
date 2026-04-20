#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(frontend_leptos::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("frontend-leptos is a browser app and should be built for wasm32-unknown-unknown.");
}
