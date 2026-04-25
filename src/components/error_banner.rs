use leptos::prelude::*;

#[component]
pub fn ErrorBanner<F>(message: Signal<Option<String>>, on_clear: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <Show when=move || message.get().is_some() fallback=|| ()>
            <div class="error-banner" role="alert" aria-live="polite">
                <span>{move || message.get().unwrap_or_default()}</span>
                <button
                    type="button"
                    class="error-banner-dismiss"
                    aria-label="Dismiss error"
                    on:click=move |_| on_clear()
                >
                    "Dismiss"
                </button>
            </div>
        </Show>
    }
}
