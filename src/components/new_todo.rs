use leptos::{ev, prelude::*};

use crate::server_fns::todo_fns::add_todo;

#[component]
pub fn NewTodo<F>(on_created: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let (title, set_title) = signal(String::new());
    let add_todo = Action::new(|title: &String| {
        let title = title.clone();
        async move { add_todo(title).await }
    });

    Effect::new(move |_| {
        if let Some(result) = add_todo.value().get() {
            match result {
                Ok(_) => {
                    set_title.set(String::new());
                    on_created();
                }
                Err(_) => {}
            }
        }
    });

    let submit = move || {
        let trimmed = title.get_untracked().trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        add_todo.dispatch(trimmed);
    };

    view! {
        <input
            class="new-todo"
            placeholder="What needs to be done?"
            prop:value=move || title.get()
            on:input=move |event| set_title.set(event_target_value(&event))
            on:keydown=move |event: ev::KeyboardEvent| {
                if event.key() == "Enter" {
                    submit();
                }
            }
        />
    }
}
