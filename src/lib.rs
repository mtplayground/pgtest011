pub mod app;
pub mod models {
    pub mod todo;
}
pub mod server_fns {
    pub mod todo_fns;
}

#[cfg(feature = "ssr")]
pub mod repo {
    pub mod todo_repo;
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
