#[path = "new_todo.rs"]
mod new_todo;
#[path = "todo_item.rs"]
mod todo_item;

use leptos::{ev, prelude::*};

use crate::{
    models::todo::{Todo, TodoStatus},
    server_fns::todo_fns::list_todos,
};

use self::new_todo::NewTodo;
use self::todo_item::TodoItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveFilter {
    All,
    Active,
    Completed,
}

impl ActiveFilter {
    fn status(self) -> Option<TodoStatus> {
        match self {
            Self::All => None,
            Self::Active => Some(TodoStatus::Active),
            Self::Completed => Some(TodoStatus::Completed),
        }
    }

    fn href(self) -> &'static str {
        match self {
            Self::All => "/",
            Self::Active => "/active",
            Self::Completed => "/completed",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }
}

#[component]
pub fn TodoApp() -> impl IntoView {
    let (active_filter, set_active_filter) = signal(ActiveFilter::All);

    let todos = Resource::new(
        move || active_filter.get(),
        |filter| async move { list_todos(filter.status()).await },
    );
    let refetch_todos = {
        let todos = todos.clone();
        move || todos.refetch()
    };

    let item_count = move || match todos.get() {
        Some(Ok(items)) => items.len(),
        _ => 0,
    };
    let active_count = move || match todos.get() {
        Some(Ok(items)) => items.iter().filter(|todo| !todo.completed).count(),
        _ => 0,
    };
    let completed_count = move || match todos.get() {
        Some(Ok(items)) => items.iter().filter(|todo| todo.completed).count(),
        _ => 0,
    };
    let has_items = move || item_count() > 0;
    let show_main = move || match todos.get() {
        None => true,
        Some(Err(_)) => true,
        Some(Ok(items)) => !items.is_empty(),
    };

    view! {
        <>
            <section class="todoapp">
                <header class="header">
                    <h1>"todos"</h1>
                    <NewTodo on_created=refetch_todos />
                </header>

                <Show when=show_main fallback=|| ()>
                    <section class="main">
                        <input id="toggle-all" class="toggle-all" type="checkbox" disabled />
                        <label for="toggle-all">"Mark all as complete"</label>

                        <ul class="todo-list">
                            {move || match todos.get() {
                                None => view! {
                                    <li class="todo-list-status">
                                        <div class="view">
                                            <label>"Loading todos..."</label>
                                        </div>
                                    </li>
                                }
                                    .into_any(),
                                Some(Err(error)) => view! {
                                    <li class="todo-list-status">
                                        <div class="view">
                                            <label>{format!("Unable to load todos: {error}")}</label>
                                        </div>
                                    </li>
                                }
                                    .into_any(),
                                Some(Ok(items)) => render_todo_items(items, refetch_todos).into_any(),
                            }}
                        </ul>
                    </section>
                </Show>

                <Show when=has_items fallback=|| ()>
                    <footer class="footer">
                        <span class="todo-count">
                            <strong>{active_count}</strong>
                            {move || {
                                if active_count() == 1 {
                                    " item left"
                                } else {
                                    " items left"
                                }
                            }}
                        </span>

                        <ul class="filters">
                            {[
                                ActiveFilter::All,
                                ActiveFilter::Active,
                                ActiveFilter::Completed,
                            ]
                                .into_iter()
                                .map(|filter| {
                                    let set_active_filter = set_active_filter;
                                    view! {
                                        <li>
                                            <a
                                                href=filter.href()
                                                class:selected=move || active_filter.get() == filter
                                                on:click=move |event: ev::MouseEvent| {
                                                    event.prevent_default();
                                                    set_active_filter.set(filter);
                                                }
                                            >
                                                {filter.label()}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>

                        <button class="clear-completed" disabled=move || completed_count() == 0>
                            "Clear completed"
                        </button>
                    </footer>
                </Show>
            </section>

            <footer class="info">
                <p>"Double-click to edit a todo"</p>
                <p>"Created for the TodoMVC reference layout"</p>
            </footer>
        </>
    }
}

fn render_todo_items<F>(items: Vec<Todo>, on_change: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    items
        .into_iter()
        .map(|todo| {
            view! {
                <TodoItem todo on_change />
            }
        })
        .collect_view()
}
