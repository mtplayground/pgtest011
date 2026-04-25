use leptos::prelude::*;

use crate::{
    models::todo::Todo,
    server_fns::todo_fns::{delete_todo, toggle_todo},
};

#[component]
pub fn TodoItem<F>(todo: Todo, on_change: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let id = todo.id;
    let (title, _) = signal(todo.title);
    let (completed, set_completed) = signal(todo.completed);
    let (deleted, set_deleted) = signal(false);

    let toggle_action = Action::new(move |next_completed: &bool| {
        let next_completed = *next_completed;
        async move { toggle_todo(id, next_completed).await }
    });
    let delete_action = Action::new(move |_: &()| async move { delete_todo(id).await });

    Effect::new(move |_| {
        if let Some(result) = toggle_action.value().get() {
            if let Ok(updated_todo) = result {
                set_completed.set(updated_todo.completed);
                on_change();
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = delete_action.value().get() {
            if result.is_ok() {
                set_deleted.set(true);
                on_change();
            }
        }
    });

    view! {
        <Show when=move || !deleted.get() fallback=|| ()>
            <li class:completed=move || completed.get()>
                <div class="view">
                    <input
                        class="toggle"
                        type="checkbox"
                        prop:checked=move || completed.get()
                        disabled=move || toggle_action.pending().get() || delete_action.pending().get()
                        on:change=move |_| {
                            toggle_action.dispatch(!completed.get_untracked());
                        }
                    />
                    <label>{move || title.get()}</label>
                    <button
                        class="destroy"
                        disabled=move || delete_action.pending().get() || toggle_action.pending().get()
                        on:click=move |_| {
                            delete_action.dispatch(());
                        }
                    ></button>
                </div>
            </li>
        </Show>
    }
}
