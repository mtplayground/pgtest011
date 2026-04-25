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
        <>
            <section class="todoapp">
                <header class="header">
                    <h1>"todos"</h1>
                    <input
                        class="new-todo"
                        placeholder="What needs to be done?"
                        readonly
                        value=""
                    />
                </header>

                <section class="main">
                    <input id="toggle-all" class="toggle-all" type="checkbox" disabled />
                    <label for="toggle-all">"Mark all as complete"</label>

                    <ul class="todo-list">
                        <li>
                            <div class="view">
                                <input class="toggle" type="checkbox" disabled />
                                <label>"TodoMVC shell wired"</label>
                                <button class="destroy" disabled></button>
                            </div>
                        </li>
                    </ul>
                </section>

                <footer class="footer">
                    <span class="todo-count">
                        <strong>"1"</strong>
                        " item left"
                    </span>

                    <ul class="filters">
                        <li><a class="selected" href="/">"All"</a></li>
                        <li><a href="/active">"Active"</a></li>
                        <li><a href="/completed">"Completed"</a></li>
                    </ul>

                    <button class="clear-completed" disabled>
                        "Clear completed"
                    </button>
                </footer>
            </section>

            <footer class="info">
                <p>"Double-click to edit a todo"</p>
                <p>"Created for the TodoMVC reference layout"</p>
            </footer>
        </>
    }
}
