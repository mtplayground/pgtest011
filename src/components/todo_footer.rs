use leptos::{ev, prelude::*};

use crate::{models::todo::TodoStatus, server_fns::todo_fns::clear_completed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

impl TodoFilter {
    pub fn status(self) -> Option<TodoStatus> {
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
pub fn TodoFooter<F>(
    active_count: Signal<usize>,
    completed_count: Signal<usize>,
    current_filter: ReadSignal<TodoFilter>,
    set_filter: WriteSignal<TodoFilter>,
    on_cleared: F,
) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let clear_completed_action = Action::new(|_: &()| async move { clear_completed().await });

    Effect::new(move |_| {
        if let Some(result) = clear_completed_action.value().get() {
            if result.is_ok() {
                on_cleared();
            }
        }
    });

    view! {
        <footer class="footer">
            <span class="todo-count">
                <strong>{move || active_count.get()}</strong>
                {move || {
                    if active_count.get() == 1 {
                        " item left"
                    } else {
                        " items left"
                    }
                }}
            </span>

            <ul class="filters">
                {[TodoFilter::All, TodoFilter::Active, TodoFilter::Completed]
                    .into_iter()
                    .map(|filter| {
                        view! {
                            <li>
                                <a
                                    href=filter.href()
                                    class:selected=move || current_filter.get() == filter
                                    on:click=move |event: ev::MouseEvent| {
                                        event.prevent_default();
                                        set_filter.set(filter);
                                    }
                                >
                                    {filter.label()}
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>

            <button
                class="clear-completed"
                disabled=move || completed_count.get() == 0 || clear_completed_action.pending().get()
                on:click=move |_| {
                    clear_completed_action.dispatch(());
                }
            >
                "Clear completed"
            </button>
        </footer>
    }
}
