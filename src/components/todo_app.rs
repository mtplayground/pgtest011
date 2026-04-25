#[path = "new_todo.rs"]
mod new_todo;
#[path = "todo_footer.rs"]
mod todo_footer;
#[path = "todo_item.rs"]
mod todo_item;

use leptos::prelude::*;

use crate::{models::todo::Todo, server_fns::todo_fns::list_todos};

use self::new_todo::NewTodo;
use self::todo_footer::{TodoFilter, TodoFooter};
use self::todo_item::TodoItem;

#[component]
pub fn TodoApp() -> impl IntoView {
    let (active_filter, set_active_filter) = signal(TodoFilter::All);

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
    let active_count = Signal::derive(move || match todos.get() {
        Some(Ok(items)) => items.iter().filter(|todo| !todo.completed).count(),
        _ => 0,
    });
    let completed_count = Signal::derive(move || match todos.get() {
        Some(Ok(items)) => items.iter().filter(|todo| todo.completed).count(),
        _ => 0,
    });
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
                    <TodoFooter
                        active_count
                        completed_count
                        current_filter=active_filter
                        set_filter=set_active_filter
                        on_cleared=refetch_todos
                    />
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
