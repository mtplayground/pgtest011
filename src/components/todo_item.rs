use leptos::{ev, html, prelude::*};

use crate::{
    models::todo::Todo,
    server_fns::todo_fns::{delete_todo, toggle_todo, update_todo},
};

#[component]
pub fn TodoItem<F>(todo: Todo, on_change: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let id = todo.id;
    let (title, set_title) = signal(todo.title);
    let (completed, set_completed) = signal(todo.completed);
    let (deleted, set_deleted) = signal(false);
    let (editing, set_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(String::new());
    let (skip_blur_submit, set_skip_blur_submit) = signal(false);
    let edit_input_ref = NodeRef::<html::Input>::new();

    let toggle_action = Action::new(move |next_completed: &bool| {
        let next_completed = *next_completed;
        async move { toggle_todo(id, next_completed).await }
    });
    let delete_action = Action::new(move |_: &()| async move { delete_todo(id).await });
    let update_action = Action::new(move |next_title: &String| {
        let next_title = next_title.clone();
        async move { update_todo(id, Some(next_title), None).await }
    });

    Effect::new(move |_| {
        if let Some(result) = toggle_action.value().get() {
            if let Ok(updated_todo) = result {
                set_completed.set(updated_todo.completed);
                on_change();
            }
        }
    });

    Effect::new(move |_| {
        if editing.get() {
            if let Some(input) = edit_input_ref.get() {
                let _ = input.focus();
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

    Effect::new(move |_| {
        if let Some(result) = update_action.value().get() {
            if let Ok(updated_todo) = result {
                let next_title = updated_todo.title;
                set_title.set(next_title.clone());
                set_completed.set(updated_todo.completed);
                set_edit_value.set(next_title);
                set_editing.set(false);
                on_change();
            }
        }
    });

    let submit_edit = move || {
        let current_title = title.get_untracked();
        let trimmed = edit_value.get_untracked().trim().to_string();

        if trimmed.is_empty() {
            delete_action.dispatch(());
            return;
        }

        if trimmed == current_title {
            set_edit_value.set(trimmed);
            set_editing.set(false);
            return;
        }

        update_action.dispatch(trimmed);
    };

    view! {
        <Show when=move || !deleted.get() fallback=|| ()>
            <li class:completed=move || completed.get() class:editing=move || editing.get()>
                <div class="view">
                    <input
                        class="toggle"
                        type="checkbox"
                        prop:checked=move || completed.get()
                        disabled=move || {
                            editing.get() || toggle_action.pending().get() || delete_action.pending().get()
                                || update_action.pending().get()
                        }
                        on:change=move |_| {
                            toggle_action.dispatch(!completed.get_untracked());
                        }
                    />
                    <label
                        on:dblclick=move |_: ev::MouseEvent| {
                            set_edit_value.set(title.get_untracked());
                            set_skip_blur_submit.set(false);
                            set_editing.set(true);
                        }
                    >
                        {move || title.get()}
                    </label>
                    <button
                        class="destroy"
                        disabled=move || {
                            delete_action.pending().get() || toggle_action.pending().get()
                                || update_action.pending().get()
                        }
                        on:click=move |_| {
                            delete_action.dispatch(());
                        }
                    ></button>
                </div>
                <input
                    node_ref=edit_input_ref
                    class="edit"
                    prop:value=move || edit_value.get()
                    disabled=move || update_action.pending().get() || delete_action.pending().get()
                    on:input=move |event| set_edit_value.set(event_target_value(&event))
                    on:blur=move |_| {
                        if skip_blur_submit.get_untracked() {
                            set_skip_blur_submit.set(false);
                            return;
                        }

                        submit_edit();
                    }
                    on:keydown=move |event: ev::KeyboardEvent| {
                        if event.key() == "Enter" {
                            submit_edit();
                        } else if event.key() == "Escape" {
                            set_skip_blur_submit.set(true);
                            set_edit_value.set(title.get_untracked());
                            set_editing.set(false);
                        }
                    }
                />
            </li>
        </Show>
    }
}
