use std::collections::{BTreeMap, BTreeSet, HashMap};

use leptos::*;
use rand::seq::SliceRandom;
use wasm_bindgen::JsCast;

use crate::model::{difficulty_label, highlight_rust, mastery_message, open_url, rich_text, Chapter, Question};
use crate::storage;
use crate::Route;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Chapter,
    Exam,
    Wrong,
}

/// A question plus the chapter it came from (needed in mixed-mode quizzes).
#[derive(Clone, PartialEq)]
pub struct QuestionCtx {
    pub q: Question,
    pub chapter: u32,
}

#[derive(Clone)]
pub struct QuizConfig {
    pub mode: Mode,
    pub title: String,
    pub questions: Vec<QuestionCtx>,
    pub chapter_for_score: Option<u32>,
}

/* ================= Shared helpers ================= */

fn wrong_entries(bank: RwSignal<Vec<Chapter>>) -> Vec<QuestionCtx> {
    bank.with_untracked(|b| {
        storage::wrong_book()
            .keys()
            .filter_map(|id| {
                b.iter().find_map(|c| {
                    c.questions
                        .iter()
                        .find(|q| &q.id == id)
                        .map(|q| QuestionCtx { q: q.clone(), chapter: c.chapter })
                })
            })
            .collect()
    })
}

fn start_chapter(bank: RwSignal<Vec<Chapter>>, route: RwSignal<Route>, num: u32) {
    bank.with_untracked(|b| {
        if let Some(ch) = b.iter().find(|c| c.chapter == num) {
            route.set(Route::Quiz(QuizConfig {
                mode: Mode::Chapter,
                title: format!("Ch {} · {}", num, ch.title),
                questions: ch
                    .questions
                    .iter()
                    .cloned()
                    .map(|q| QuestionCtx { q, chapter: num })
                    .collect(),
                chapter_for_score: Some(num),
            }));
        }
    });
}

/// Practice a single subsection. No chapter score is recorded (partial run).
fn start_section(bank: RwSignal<Vec<Chapter>>, route: RwSignal<Route>, num: u32, section: String) {
    bank.with_untracked(|b| {
        if let Some(ch) = b.iter().find(|c| c.chapter == num) {
            let questions: Vec<QuestionCtx> = ch
                .questions
                .iter()
                .filter(|q| q.section == section)
                .cloned()
                .map(|q| QuestionCtx { q, chapter: num })
                .collect();
            if questions.is_empty() {
                return;
            }
            route.set(Route::Quiz(QuizConfig {
                mode: Mode::Chapter,
                title: format!("Ch {num} · § {section}"),
                questions,
                chapter_for_score: None,
            }));
        }
    });
}

fn start_wrong(bank: RwSignal<Vec<Chapter>>, route: RwSignal<Route>) {
    let mut questions = wrong_entries(bank);
    if questions.is_empty() {
        return;
    }
    questions.shuffle(&mut rand::thread_rng());
    route.set(Route::Quiz(QuizConfig {
        mode: Mode::Wrong,
        title: "Wrong Answer Practice".into(),
        questions,
        chapter_for_score: None,
    }));
}

fn toggle_select(
    answers: RwSignal<HashMap<String, BTreeSet<String>>>,
    qid: &str,
    multi: bool,
    letter: &str,
) {
    let qid = qid.to_string();
    let l = letter.to_string();
    answers.update(|a| {
        let set = a.entry(qid.clone()).or_default();
        if multi {
            if !set.remove(&l) {
                set.insert(l);
            }
        } else {
            set.clear();
            set.insert(l);
        }
        let empty = set.is_empty();
        if empty {
            a.remove(&qid);
        }
    });
}

/* ================= Sidebar ================= */

#[component]
pub fn Sidebar() -> impl IntoView {
    let bank = use_context::<RwSignal<Vec<Chapter>>>().expect("bank context");
    let route = use_context::<RwSignal<Route>>().expect("route context");
    let ver = use_context::<RwSignal<u32>>().expect("storage version context");
    let side_open = use_context::<RwSignal<bool>>().expect("drawer context");

    view! {
        <aside
            class="side"
            class:open=move || side_open.get()
            on:click=move |_| side_open.set(false)
        >
            {move || {
                ver.track();
                let cur = route.get();
                let scores = storage::scores();
                let wrong_n = wrong_entries(bank).len();
                let chapters = bank.get();
                let total_q: usize = chapters.iter().map(|c| c.questions.len()).sum();

                let home_act = matches!(cur, Route::Home);
                let exam_act = matches!(cur, Route::Exam);
                let wrong_act = matches!(cur, Route::Wrong);

                view! {
                    <div class="logo"><b>"RUST"</b>" QUIZ"</div>
                    <div class="nav" class:act=home_act on:click=move |_| route.set(Route::Home)>
                        <span class="nav-ico">"◈"</span>"Dashboard"
                    </div>
                    <div class="nav" class:act=exam_act on:click=move |_| route.set(Route::Exam)>
                        <span class="nav-ico">"▤"</span>"Random Exam"
                    </div>
                    <div class="nav" class:act=wrong_act on:click=move |_| route.set(Route::Wrong)>
                        <span class="nav-ico">"↻"</span>"Wrong Book"
                        {(wrong_n > 0).then(|| view! { <span class="nav-badge">{wrong_n}</span> })}
                    </div>
                    <div class="side-sec">"CHAPTERS"</div>
                    <div class="side-chapters">
                        {chapters.into_iter().map(|ch| {
                            let num = ch.chapter;
                            let best = scores.get(&num).map(|e| e.best);
                            let active = matches!(&cur, Route::Chapter(n) if *n == num)
                                || matches!(&cur, Route::Quiz(c) if c.chapter_for_score == Some(num));
                            view! {
                                <div class="chrow" class:act=active on:click=move |_| route.set(Route::Chapter(num))>
                                    <span class="ch-name">{format!("Ch{} {}", num, ch.title)}</span>
                                    <span class="ch-bar"><i style=format!("width:{}%", best.unwrap_or(0))></i></span>
                                    <span class="ch-pct">{best.map(|b| format!("{b}%")).unwrap_or_else(|| "—".into())}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    <div class="side-stats">
                        <span>"Questions"<b>{total_q}</b></span>
                        <span>"Wrong book"<b>{wrong_n}</b></span>
                    </div>
                }
            }}
        </aside>
    }
}

/* ================= Dashboard ================= */

#[component]
pub fn HomeView() -> impl IntoView {
    let bank = use_context::<RwSignal<Vec<Chapter>>>().expect("bank context");
    let route = use_context::<RwSignal<Route>>().expect("route context");

    let scores = storage::scores();
    let wrong_n = wrong_entries(bank).len();
    let chapters = bank.get_untracked();
    let total_q: usize = chapters.iter().map(|c| c.questions.len()).sum();
    let n_ch = chapters.len();

    view! {
        <div class="page">
            <div class="page-head">
                <div>
                    <h1>"Dashboard"</h1>
                    <p class="page-sub">"The Rust Programming Language — knowledge checks"</p>
                </div>
                <button class="btn ghost" on:click=|_| open_url("https://doc.rust-lang.org/book/")>"Open the book ↗"</button>
            </div>

            <div class="stat-cards">
                <div class="stat-card"><b>{n_ch}</b><span>"Chapters"</span></div>
                <div class="stat-card"><b>{total_q}</b><span>"Questions"</span></div>
                <div class="stat-card"><b>{wrong_n}</b><span>"In wrong book"</span></div>
            </div>

            <h2 class="sec-title">"Chapters"</h2>
            <div class="grid">
                {chapters.into_iter().map(|ch| {
                    let num = ch.chapter;
                    let link = ch.link.clone();
                    let best = scores.get(&num).map(|e| e.best);
                    view! {
                        <div class="gcard" on:click=move |_| route.set(Route::Chapter(num))>
                            <div class="gc-num">{format!("CH {num:02}")}</div>
                            <div class="gc-title">{ch.title.clone()}</div>
                            <div class="gc-meta">
                                <span class="pill">{format!("{} Q", ch.questions.len())}</span>
                                {match best {
                                    Some(b) => view! { <span class="pill ok">{format!("best {b}%")}</span> }.into_view(),
                                    None => view! { <span class="pill dim">"new"</span> }.into_view(),
                                }}
                                <span class="gc-link" on:click=move |e| {
                                    e.stop_propagation();
                                    open_url(&link);
                                }>"read ↗"</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/* ================= Chapter detail (sections) ================= */

#[component]
pub fn ChapterView(num: u32) -> impl IntoView {
    let bank = use_context::<RwSignal<Vec<Chapter>>>().expect("bank context");
    let route = use_context::<RwSignal<Route>>().expect("route context");

    let Some(ch) = bank.with_untracked(|b| b.iter().find(|c| c.chapter == num).cloned()) else {
        return view! { <div class="page"><p class="muted">"Chapter not found."</p></div> }.into_view();
    };

    let best = storage::scores().get(&num).map(|e| e.best);
    let total = ch.questions.len();
    let link = ch.link.clone();

    // Sections in order of first appearance, with question counts.
    let mut sections: Vec<(String, usize)> = Vec::new();
    for q in &ch.questions {
        match sections.iter_mut().find(|(s, _)| s == &q.section) {
            Some((_, n)) => *n += 1,
            None => sections.push((q.section.clone(), 1)),
        }
    }

    view! {
        <div class="page narrow">
            <div class="page-head">
                <div>
                    <h1>{format!("Ch {} · {}", num, ch.title)}</h1>
                    <p class="page-sub">
                        {format!("{total} questions · {} section(s)", sections.len())}
                        {best.map(|b| format!(" · best {b}%")).unwrap_or_default()}
                    </p>
                </div>
                <button class="btn ghost" on:click=move |_| open_url(&link)>"Read chapter ↗"</button>
            </div>

            <div class="panel">
                <button class="btn primary big" on:click=move |_| start_chapter(bank, route, num)>
                    {format!("Practice whole chapter · {total} Q →")}
                </button>
                <div class="field-label" style="margin-top:6px;">"Or practice one section (not scored)"</div>
                <div class="sec-list">
                    {sections.into_iter().map(|(section, n)| {
                        let sec2 = section.clone();
                        view! {
                            <div class="sec-row" on:click=move |_| start_section(bank, route, num, sec2.clone())>
                                <span class="sec-name">{format!("§ {section}")}</span>
                                <span class="pill">{format!("{n} Q")}</span>
                                <span class="sec-go">"→"</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
    .into_view()
}

/* ================= Random exam builder ================= */

#[component]
pub fn ExamView() -> impl IntoView {
    let bank = use_context::<RwSignal<Vec<Chapter>>>().expect("bank context");
    let route = use_context::<RwSignal<Route>>().expect("route context");

    let selected: RwSignal<BTreeSet<u32>> =
        create_rw_signal(bank.with_untracked(|b| b.iter().map(|c| c.chapter).collect()));
    let count_input = create_rw_signal(String::from("25"));
    let diff_filter = create_rw_signal(0u8);

    let start = move |_| {
        let picked = selected.get_untracked();
        let want: usize = count_input.get_untracked().parse().unwrap_or(25).max(1);
        let diff = diff_filter.get_untracked();
        let mut pool: Vec<QuestionCtx> = bank.with_untracked(|b| {
            b.iter()
                .filter(|c| picked.contains(&c.chapter))
                .flat_map(|c| {
                    c.questions
                        .iter()
                        .cloned()
                        .map(|q| QuestionCtx { q, chapter: c.chapter })
                })
                .filter(|qc| diff == 0 || qc.q.difficulty == diff)
                .collect()
        });
        if pool.is_empty() {
            return;
        }
        pool.shuffle(&mut rand::thread_rng());
        pool.truncate(want);
        let n = pool.len();
        route.set(Route::Quiz(QuizConfig {
            mode: Mode::Exam,
            title: format!("Random Exam · {n} questions"),
            questions: pool,
            chapter_for_score: None,
        }));
    };

    let chips = bank
        .with_untracked(|b| b.iter().map(|c| c.chapter).collect::<Vec<_>>())
        .into_iter()
        .map(|num| {
            view! {
                <span
                    class="chip"
                    class:on=move || selected.with(|s| s.contains(&num))
                    on:click=move |_| selected.update(|s| {
                        if !s.remove(&num) {
                            s.insert(num);
                        }
                    })
                >
                    {format!("Ch {num}")}
                </span>
            }
        })
        .collect_view();

    let bump = move |delta: i64| {
        let cur: i64 = count_input.get_untracked().parse().unwrap_or(25);
        count_input.set((cur + delta).max(1).to_string());
    };

    view! {
        <div class="page narrow">
            <div class="page-head">
                <div>
                    <h1>"Random Exam"</h1>
                    <p class="page-sub">"Answer the whole paper, then submit for grading. Questions and order are randomized."</p>
                </div>
            </div>
            <div class="panel">
                <div class="field-label-row">
                    <span class="field-label">"Chapters"</span>
                    <span class="mini-link" on:click=move |_| {
                        selected.set(bank.with_untracked(|b| b.iter().map(|c| c.chapter).collect()));
                    }>"Select all"</span>
                    <span class="mini-link" on:click=move |_| selected.update(|s| s.clear())>"Clear"</span>
                </div>
                <div class="chips">{chips}</div>
                <div class="builder-row">
                    <div class="field">
                        <label>"Questions"</label>
                        <div class="stepper">
                            <button class="step-btn" on:click=move |_| bump(-5)>"−"</button>
                            <input
                                class="step-val"
                                type="text"
                                inputmode="numeric"
                                prop:value=move || count_input.get()
                                on:input=move |ev| count_input.set(event_target_value(&ev))
                            />
                            <button class="step-btn" on:click=move |_| bump(5)>"+"</button>
                        </div>
                    </div>
                    <div class="field">
                        <label>"Difficulty"</label>
                        <select on:change=move |ev| diff_filter.set(event_target_value(&ev).parse().unwrap_or(0))>
                            <option value="0">"All levels"</option>
                            <option value="1">"Easy only"</option>
                            <option value="2">"Medium only"</option>
                            <option value="3">"Hard only"</option>
                        </select>
                    </div>
                    <button class="btn primary big" on:click=start>"Start exam →"</button>
                </div>
            </div>
        </div>
    }
}

/* ================= Wrong book ================= */

#[component]
pub fn WrongView() -> impl IntoView {
    let bank = use_context::<RwSignal<Vec<Chapter>>>().expect("bank context");
    let route = use_context::<RwSignal<Route>>().expect("route context");
    let ver = use_context::<RwSignal<u32>>().expect("storage version context");

    let n = wrong_entries(bank).len();
    let confirm_clear = create_rw_signal(false);

    view! {
        <div class="page narrow">
            <div class="page-head">
                <div>
                    <h1>"Wrong Answer Book"</h1>
                    <p class="page-sub">
                        "Missed questions land here automatically. Answer one correctly and it leaves the book."
                    </p>
                </div>
            </div>
            <div class="panel center-panel">
                <div class="wrong-count">{n}</div>
                <p class="muted">{if n > 0 { "questions waiting for re-drill" } else { "the book is empty — nice work" }}</p>
                <div class="btn-row">
                    <button class="btn primary big" prop:disabled={n == 0} on:click=move |_| start_wrong(bank, route)>
                        "Practice now →"
                    </button>
                    <button class="btn danger" prop:disabled={n == 0} on:click=move |_| confirm_clear.set(true)>
                        "Clear all"
                    </button>
                </div>
            </div>
            {move || confirm_clear.get().then(|| view! {
                <div class="modal-back" on:click=move |_| confirm_clear.set(false)>
                    <div class="modal" on:click=|e| e.stop_propagation()>
                        <h3>"Clear wrong book?"</h3>
                        <p class="muted">"All collected questions will be removed. This cannot be undone."</p>
                        <div class="btn-row center">
                            <button class="btn" on:click=move |_| confirm_clear.set(false)>"Cancel"</button>
                            <button class="btn danger" on:click=move |_| {
                                storage::clear_all_wrong();
                                ver.update(|v| *v += 1);
                                confirm_clear.set(false);
                                route.set(Route::Wrong);
                            }>"Clear all"</button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

/* ================= Quiz (single-question flow) ================= */

#[component]
pub fn QuizView(cfg: QuizConfig) -> impl IntoView {
    let route = use_context::<RwSignal<Route>>().expect("route context");
    let ver = use_context::<RwSignal<u32>>().expect("storage version context");
    let QuizConfig { mode, title, questions, chapter_for_score } = cfg;
    let total = questions.len();
    let questions = store_value(questions);

    let idx = create_rw_signal(0usize);
    let answers: RwSignal<HashMap<String, BTreeSet<String>>> = create_rw_signal(HashMap::new());
    // qid -> answered correctly (present = checked/locked)
    let checked: RwSignal<HashMap<String, bool>> = create_rw_signal(HashMap::new());
    let finished = create_rw_signal(false);
    // Exam mode grades the whole paper on submit; other modes check per question.
    let instant = mode != Mode::Exam;
    let graded = create_rw_signal(false);
    // In-app confirmation modal for submitting with unanswered questions.
    let confirm_submit = create_rw_signal(false);

    let check_current = move || {
        if !instant || finished.get_untracked() {
            return;
        }
        let qc = questions.with_value(|qs| qs[idx.get_untracked()].clone());
        if checked.with_untracked(|c| c.contains_key(&qc.q.id)) {
            return;
        }
        let sel = answers.with_untracked(|a| a.get(&qc.q.id).cloned().unwrap_or_default());
        if sel.is_empty() {
            return;
        }
        let ok = sel.into_iter().collect::<Vec<_>>() == qc.q.answer.letters();
        checked.update(|c| {
            c.insert(qc.q.id.clone(), ok);
        });
        if ok {
            storage::clear_wrong(&qc.q.id);
        } else {
            storage::record_wrong(&qc.q.id);
        }
    };

    let finish = move || {
        if !graded.get_untracked() {
            if !instant {
                // Grade the whole paper at once (exam mode).
                questions.with_value(|qs| {
                    for qc in qs {
                        let sel = answers.with_untracked(|a| a.get(&qc.q.id).cloned().unwrap_or_default());
                        let attempted = !sel.is_empty();
                        let ok = attempted
                            && sel.into_iter().collect::<Vec<_>>() == qc.q.answer.letters();
                        checked.update(|c| {
                            c.insert(qc.q.id.clone(), ok);
                        });
                        if ok {
                            storage::clear_wrong(&qc.q.id);
                        } else if attempted {
                            storage::record_wrong(&qc.q.id);
                        }
                    }
                });
            }
            if let (Mode::Chapter, Some(ch)) = (mode, chapter_for_score) {
                let right = checked.with_untracked(|c| c.values().filter(|v| **v).count());
                let pct = ((right as f64 / total.max(1) as f64) * 100.0).round() as u32;
                storage::save_score(ch, pct);
            }
            graded.set(true);
            ver.update(|v| *v += 1);
        }
        finished.set(true);
    };

    let next = move || {
        let i = idx.get_untracked();
        if i + 1 < total {
            idx.set(i + 1);
            return;
        }
        // Last question: exam mode confirms before submitting with blanks.
        if !instant && !graded.get_untracked() {
            let unanswered = total - answers.with_untracked(|a| a.len());
            if unanswered > 0 {
                confirm_submit.set(true);
                return;
            }
        }
        finish();
    };
    let prev = move || {
        let i = idx.get_untracked();
        if i > 0 {
            idx.set(i - 1);
        }
    };
    let check_or_next = move || {
        if !instant {
            next();
            return;
        }
        let qid = questions.with_value(|qs| qs[idx.get_untracked()].q.id.clone());
        if checked.with_untracked(|c| c.contains_key(&qid)) {
            next();
        } else {
            check_current();
        }
    };

    // Keyboard: 1-5 select, Enter check/next, arrows navigate, Esc exits.
    let key_handle = window_event_listener(ev::keydown, move |e| {
        if confirm_submit.get_untracked() {
            if e.key() == "Escape" {
                confirm_submit.set(false);
            }
            return;
        }
        if e.key() == "Escape" {
            route.set(Route::Home);
            return;
        }
        if finished.get_untracked() {
            return;
        }
        // Ignore keys typed into form controls.
        if let Some(t) = e.target() {
            if let Ok(el) = t.dyn_into::<web_sys::HtmlElement>() {
                let tag = el.tag_name();
                if tag == "INPUT" || tag == "SELECT" || tag == "TEXTAREA" {
                    return;
                }
            }
        }
        match e.key().as_str() {
            "ArrowRight" => next(),
            "ArrowLeft" => prev(),
            "Enter" => {
                e.prevent_default();
                check_or_next();
            }
            k if k.len() == 1 => {
                if let Some(d) = k.chars().next().and_then(|c| c.to_digit(10)) {
                    let qc = questions.with_value(|qs| qs[idx.get_untracked()].clone());
                    if checked.with_untracked(|c| c.contains_key(&qc.q.id)) {
                        return;
                    }
                    let letters: Vec<String> = qc.q.options.keys().cloned().collect();
                    if d >= 1 {
                        if let Some(letter) = letters.get(d as usize - 1) {
                            toggle_select(answers, &qc.q.id, qc.q.answer.is_multi(), letter);
                        }
                    }
                }
            }
            _ => {}
        }
    });
    on_cleanup(move || key_handle.remove());

    // Progress: dots for short quizzes, compact bar for long ones.
    let progress = move || {
        let cur = idx.get();
        if total <= 40 {
            view! {
                <div class="dots">
                    {(0..total).map(|i| {
                        let st = questions.with_value(|qs| checked.with(|c| c.get(&qs[i].q.id).copied()));
                        let ans = questions.with_value(|qs| answers.with(|a| a.contains_key(&qs[i].q.id)));
                        let cls = if i == cur {
                            "dot cur"
                        } else {
                            match st {
                                Some(true) => "dot good",
                                Some(false) => "dot bad",
                                None => {
                                    if ans {
                                        "dot ans"
                                    } else {
                                        "dot"
                                    }
                                }
                            }
                        };
                        view! { <span class=cls on:click=move |_| idx.set(i)></span> }
                    }).collect_view()}
                </div>
            }
            .into_view()
        } else {
            let pct = (cur + 1) * 100 / total.max(1);
            view! {
                <div class="qprog">
                    <span>{format!("Q {} / {}", cur + 1, total)}</span>
                    <span class="track"><i style=format!("width:{pct}%")></i></span>
                </div>
            }
            .into_view()
        }
    };

    // Current question panel — rebuilt whenever idx/answers/checked change.
    let question_panel = move || {
        let i = idx.get();
        let qc = questions.with_value(|qs| qs[i].clone());
        let chapter = qc.chapter;
        let q = qc.q;
        let qid = q.id.clone();
        let multi = q.answer.is_multi();
        let answer_letters = q.answer.letters();
        let sel = answers.with(|a| a.get(&qid).cloned().unwrap_or_default());
        let state = checked.with(|c| c.get(&qid).copied());
        let is_checked = state.is_some();
        let has_sel = !sel.is_empty();

        let opts = q
            .options
            .iter()
            .map(|(letter, text)| {
                let is_sel = sel.contains(letter);
                let is_ans = answer_letters.contains(letter);
                let cls = if is_checked {
                    if is_ans {
                        "opt good"
                    } else if is_sel {
                        "opt bad"
                    } else {
                        "opt locked"
                    }
                } else if is_sel {
                    "opt sel"
                } else {
                    "opt"
                };
                let qid2 = qid.clone();
                let letter2 = letter.clone();
                view! {
                    <div class=cls on:click=move |_| {
                        if checked.with_untracked(|c| c.contains_key(&qid2)) {
                            return;
                        }
                        toggle_select(answers, &qid2, multi, &letter2);
                    }>
                        <span class="l">{letter.clone()}</span>
                        <div inner_html=rich_text(text)></div>
                    </div>
                }
            })
            .collect_view();

        view! {
            <div class="stage">
                <div class="qtop">
                    <span class="qnum">{format!("{:02}", i + 1)}</span>
                    <span class="qof">{format!("/ {total}")}</span>
                    <div class="tags">
                        <span class="pill hot">{q.tag.clone()}</span>
                        <span class="pill">{difficulty_label(q.difficulty)}</span>
                        <span class="pill">{format!("§ {}", q.section)}</span>
                        {multi.then(|| view! { <span class="pill multi">"MULTI-SELECT"</span> })}
                        {(mode != Mode::Chapter).then(|| view! { <span class="pill">{format!("Ch {chapter}")}</span> })}
                    </div>
                </div>
                <h2 class="qtext" inner_html=rich_text(&q.prompt)></h2>
                {q.code.clone().map(|c| view! { <pre class="code"><code inner_html=highlight_rust(&c)></code></pre> })}
                <div class="opts">{opts}</div>
                {state.map(|ok| view! {
                    <div class=if ok { "feedback good" } else { "feedback bad" }>
                        <div class="fb-head">
                            {if ok { "✓ CORRECT" } else { "✗ INCORRECT" }}
                            {(!ok).then(|| view! { <span class="fb-ans">{format!("answer: {}", answer_letters.join(", "))}</span> })}
                        </div>
                        <div class="fb-body" inner_html=rich_text(&q.explanation)></div>
                    </div>
                })}
                <div class="qfoot">
                    <span class="kbd">
                        <b>"1-5"</b>" select · "<b>"↵"</b>
                        {if instant { " check / next · " } else { " next · " }}
                        <b>"← →"</b>" navigate · "<b>"Esc"</b>" exit"
                    </span>
                    <div class="btn-row">
                        <button class="btn" prop:disabled={i == 0} on:click=move |_| prev()>"← Prev"</button>
                        {if is_checked || !instant {
                            let label = if i + 1 < total {
                                "Next →"
                            } else if graded.get() {
                                "Results"
                            } else if instant {
                                "Finish ✓"
                            } else {
                                "Submit ✓"
                            };
                            view! {
                                <button class="btn primary" on:click=move |_| next()>{label}</button>
                            }.into_view()
                        } else {
                            view! {
                                <button class="btn primary" prop:disabled={!has_sel} on:click=move |_| check_current()>
                                    "Check"
                                </button>
                            }.into_view()
                        }}
                    </div>
                </div>
            </div>
        }
    };

    let results = move || {
        let right = checked.with(|c| c.values().filter(|v| **v).count());
        let attempted = answers.with(|a| a.len());
        let pct = ((right as f64 / total.max(1) as f64) * 100.0).round() as u32;
        let mut per: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
        questions.with_value(|qs| {
            for qc in qs {
                let e = per.entry(qc.chapter).or_default();
                e.1 += 1;
                if checked.with(|c| c.get(&qc.q.id).copied().unwrap_or(false)) {
                    e.0 += 1;
                }
            }
        });
        view! {
            <div class="results">
                <div class="score-big">{format!("{pct}%")}</div>
                <div class="score-sub">{format!("{right} / {total} correct · {attempted} attempted")}</div>
                <p class="mastery">{mastery_message(pct)}</p>
                {(per.len() > 1).then(|| view! {
                    <div class="breakdown">
                        {per.iter().map(|(ch, (r, t))| {
                            view! {
                                <div class="bd-row">
                                    <span>{format!("Chapter {ch}")}</span>
                                    <strong>{format!("{r} / {t}")}</strong>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                })}
                <div class="btn-row center">
                    <button class="btn" on:click=move |_| {
                        finished.set(false);
                        idx.set(0);
                    }>"Review answers"</button>
                    <button class="btn" on:click=move |_| {
                        idx.set(0);
                        answers.set(HashMap::new());
                        checked.set(HashMap::new());
                        graded.set(false);
                        finished.set(false);
                    }>"Retry"</button>
                    <button class="btn primary big" on:click=move |_| route.set(Route::Home)>"Done ✓"</button>
                </div>
            </div>
        }
    };

    view! {
        <div class="quiz">
            <div class="quiz-head">
                <button class="btn ghost" on:click=move |_| route.set(Route::Home)>"← Exit"</button>
                <span class="quiz-title">{title.clone()}</span>
                <div class="quiz-prog">{progress}</div>
            </div>
            <div class="qwrap">
                {move || if finished.get() { results().into_view() } else { question_panel().into_view() }}
            </div>
            {move || confirm_submit.get().then(|| {
                let unanswered = total - answers.with(|a| a.len());
                view! {
                    <div class="modal-back" on:click=move |_| confirm_submit.set(false)>
                        <div class="modal" on:click=|e| e.stop_propagation()>
                            <h3>"Submit exam?"</h3>
                            <p class="muted">
                                {format!("{unanswered} question(s) are still unanswered and will count as incorrect.")}
                            </p>
                            <div class="btn-row center">
                                <button class="btn" on:click=move |_| confirm_submit.set(false)>"Keep working"</button>
                                <button class="btn primary" on:click=move |_| {
                                    confirm_submit.set(false);
                                    finish();
                                }>"Submit ✓"</button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
