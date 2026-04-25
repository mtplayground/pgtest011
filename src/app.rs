use leptos::hydration::{AutoReload, HydrationScripts};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/pgtest011.css" />
        <Title text="pgtest011" />

        <Router>
            <main class="app-shell">
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section class="hero">
            <p class="eyebrow">"Leptos + Axum"</p>
            <h1>"Hello, world!"</h1>
            <p class="copy">
                "The pgtest011 scaffold is running with server-side rendering and hydration enabled."
            </p>
        </section>
    }
}
