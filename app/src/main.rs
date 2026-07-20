mod model;
mod storage;
mod views;

use leptos::*;
use model::{in_tauri, load_bank, win_cmd, Chapter};
use views::{ChapterView, ExamView, HomeView, QuizConfig, QuizView, Sidebar, WrongView};

/// Which screen is showing in the main area.
#[derive(Clone)]
pub enum Route {
    Home,
    Chapter(u32),
    Exam,
    Wrong,
    Quiz(QuizConfig),
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let bank: RwSignal<Vec<Chapter>> = create_rw_signal(Vec::new());
    let route = create_rw_signal(Route::Home);
    // Bumped whenever localStorage changes so the sidebar stats refresh.
    let storage_version = create_rw_signal(0u32);
    let light = create_rw_signal(storage::theme_light());
    // Mobile drawer state for the sidebar (ignored by desktop CSS).
    let side_open = create_rw_signal(false);
    provide_context(bank);
    provide_context(route);
    provide_context(storage_version);
    provide_context(side_open);

    // Apply the theme class on <body> and persist the preference.
    create_effect(move |_| {
        let l = light.get();
        if let Some(body) = document().body() {
            body.set_class_name(if l { "light" } else { "" });
        }
        storage::save_theme(l);
    });

    // Bank loads asynchronously (from bank/*.json next to the exe on desktop).
    spawn_local(async move {
        bank.set(load_bank().await);
    });

    view! {
        <div class="titlebar" data-tauri-drag-region="">
            <img class="tb-crab" src="ferris.png" alt="" data-tauri-drag-region=""/>
            <span class="tb-title" data-tauri-drag-region="">"Rust Book Quiz"</span>
            <div class="tb-controls">
                <button class="tb-btn" title="Toggle theme" on:click=move |_| light.update(|l| *l = !*l)>
                    {move || if light.get() { "☀" } else { "☾" }}
                </button>
                // Window controls only exist inside the Tauri shell; in a
                // plain browser the tab/window chrome handles these.
                {in_tauri().then(|| view! {
                    <button class="tb-btn" title="Minimize" on:click=|_| win_cmd("win_minimize")>"–"</button>
                    <button class="tb-btn" title="Maximize" on:click=|_| win_cmd("win_toggle_maximize")>"▢"</button>
                    <button class="tb-btn close" title="Close" on:click=|_| win_cmd("win_close")>"✕"</button>
                })}
            </div>
        </div>
        {move || (!matches!(route.get(), Route::Quiz(_))).then(|| view! {
            <button class="menu-fab" on:click=move |_| side_open.update(|v| *v = !*v)>"☰"</button>
        })}
        <div class="shell">
            // Zen mode: the sidebar hides while a quiz is running.
            {move || (!matches!(route.get(), Route::Quiz(_))).then(|| view! { <Sidebar/> })}
            <main class="main">
                {move || {
                    // Re-render the current view when the bank arrives.
                    bank.track();
                    match route.get() {
                        Route::Home => view! { <HomeView/> }.into_view(),
                        Route::Chapter(num) => view! { <ChapterView num/> }.into_view(),
                        Route::Exam => view! { <ExamView/> }.into_view(),
                        Route::Wrong => view! { <WrongView/> }.into_view(),
                        Route::Quiz(cfg) => view! { <QuizView cfg/> }.into_view(),
                    }
                }}
            </main>
        </div>
    }
}
