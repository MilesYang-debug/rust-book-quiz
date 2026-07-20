/// Bank schema types live in the `quiz-bank` crate (shared with the
/// validator: `cargo run -p quiz-bank`) — re-exported here so the rest of
/// the app keeps using `crate::model::{Chapter, Question}`. (`Answer` is
/// only used through its methods; name it via `quiz_bank::Answer` if needed.)
pub use quiz_bank::{Chapter, Question};

/// Embedded snapshot of the bank — fallback when no external bank/ folder
/// is found (or when running outside Tauri, e.g. `trunk serve`).
fn embedded_bank() -> Vec<Chapter> {
    serde_json::from_str(include_str!("../assets/bank.json")).expect("invalid bank.json")
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(
        js_namespace = ["window", "__TAURI__", "core"],
        js_name = invoke,
        catch
    )]
    async fn tauri_invoke(
        cmd: &str,
        args: wasm_bindgen::JsValue,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

/// True when running inside a Tauri WebView (desktop/mobile shell);
/// false in a plain browser (web deployment / `trunk serve`).
pub fn in_tauri() -> bool {
    js_sys::Reflect::has(&leptos::window(), &wasm_bindgen::JsValue::from_str("__TAURI__"))
        .unwrap_or(false)
}

/// Load the question bank. On desktop this reads `bank/*.json` next to the
/// exe at runtime (drop in a new chNN.json and restart — no rebuild needed);
/// otherwise it falls back to the embedded snapshot.
pub async fn load_bank() -> Vec<Chapter> {
    if in_tauri() {
        if let Ok(v) = tauri_invoke("load_bank_files", wasm_bindgen::JsValue::UNDEFINED).await {
            if let Ok(files) = serde_wasm_bindgen::from_value::<Vec<String>>(v) {
                let mut bank: Vec<Chapter> = files
                    .iter()
                    .filter_map(|s| serde_json::from_str(s).ok())
                    .collect();
                if !bank.is_empty() {
                    bank.sort_by_key(|c| c.chapter);
                    return bank;
                }
            }
        }
    }
    embedded_bank()
}

#[derive(serde::Serialize)]
struct UrlArgs<'a> {
    url: &'a str,
}

/// Open a link in the system browser (via the Tauri backend on desktop).
pub fn open_url(url: &str) {
    if in_tauri() {
        if let Ok(args) = serde_wasm_bindgen::to_value(&UrlArgs { url }) {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = tauri_invoke("open_url", args).await;
            });
        }
    } else {
        let _ = leptos::window().open_with_url_and_target(url, "_blank");
    }
}

/// Fire-and-forget window control command (custom titlebar buttons).
pub fn win_cmd(cmd: &'static str) {
    if in_tauri() {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tauri_invoke(cmd, wasm_bindgen::JsValue::UNDEFINED).await;
        });
    }
}

pub fn difficulty_label(d: u8) -> &'static str {
    match d {
        1 => "Easy",
        2 => "Medium",
        _ => "Hard",
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Escape HTML, then render `backtick spans` as inline <code>.
pub fn rich_text(s: &str) -> String {
    let esc = escape_html(s);
    let mut out = String::with_capacity(esc.len() + 32);
    for (i, part) in esc.split('`').enumerate() {
        if i % 2 == 1 {
            out.push_str("<code class=\"inline\">");
            out.push_str(part);
            out.push_str("</code>");
        } else {
            out.push_str(part);
        }
    }
    out
}

pub fn mastery_message(percent: u32) -> &'static str {
    match percent {
        93..=100 => "Outstanding mastery. You understand both the explicit content and the hidden traps.",
        82..=92 => "Strong mastery. Your understanding is solid, with only a few finer points left to sharpen.",
        68..=81 => "Moderate mastery. You know the core material, but some distinctions still need reinforcement.",
        50..=67 => "Partial mastery. You have some correct intuitions, but several important ideas are not yet stable.",
        _ => "Low mastery. It would be worth rereading the relevant sections carefully before retrying.",
    }
}

/* ================= Rust syntax highlighting ================= */

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while",
];

fn esc_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

fn span_into(out: &mut String, cls: &str, s: &str) {
    out.push_str("<span class=\"");
    out.push_str(cls);
    out.push_str("\">");
    esc_into(out, s);
    out.push_str("</span>");
}

/// Minimal single-pass Rust highlighter for quiz snippets.
/// Emits escaped HTML with hl-* spans (keywords, types, strings, chars,
/// macros, comments, numbers, lifetimes, attributes).
pub fn highlight_rust(code: &str) -> String {
    let ch: Vec<char> = code.chars().collect();
    let n = ch.len();
    let mut out = String::with_capacity(code.len() * 2);
    let sub = |a: usize, b: usize| ch[a..b].iter().collect::<String>();
    let mut i = 0;

    while i < n {
        let c = ch[i];

        // line + block comments
        if c == '/' && i + 1 < n && ch[i + 1] == '/' {
            let start = i;
            while i < n && ch[i] != '\n' {
                i += 1;
            }
            span_into(&mut out, "hl-com", &sub(start, i));
            continue;
        }
        if c == '/' && i + 1 < n && ch[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < n && !(ch[i] == '*' && ch[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(n);
            span_into(&mut out, "hl-com", &sub(start, i));
            continue;
        }

        // string literals
        if c == '"' {
            let start = i;
            i += 1;
            while i < n {
                if ch[i] == '\\' {
                    i += 2;
                    continue;
                }
                if ch[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            span_into(&mut out, "hl-str", &sub(start, i.min(n)));
            continue;
        }

        // attributes like #[derive(Debug)]
        if c == '#' && i + 1 < n && ch[i + 1] == '[' {
            let start = i;
            while i < n && ch[i] != ']' {
                i += 1;
            }
            i = (i + 1).min(n);
            span_into(&mut out, "hl-attr", &sub(start, i));
            continue;
        }

        // char literal vs lifetime
        if c == '\'' {
            let is_char = (i + 3 < n && ch[i + 1] == '\\' && ch[i + 3] == '\'')
                || (i + 2 < n && ch[i + 1] != '\\' && ch[i + 2] == '\'');
            if is_char {
                let end = if ch[i + 1] == '\\' { i + 4 } else { i + 3 };
                span_into(&mut out, "hl-str", &sub(i, end.min(n)));
                i = end.min(n);
            } else {
                let start = i;
                i += 1;
                while i < n && (ch[i].is_alphanumeric() || ch[i] == '_') {
                    i += 1;
                }
                span_into(&mut out, "hl-life", &sub(start, i));
            }
            continue;
        }

        // numbers (also swallows 0..5 ranges — close enough for snippets)
        if c.is_ascii_digit() {
            let start = i;
            while i < n && (ch[i].is_ascii_alphanumeric() || ch[i] == '_' || ch[i] == '.') {
                i += 1;
            }
            span_into(&mut out, "hl-num", &sub(start, i));
            continue;
        }

        // identifiers: macro! / keyword / Type / plain
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (ch[i].is_alphanumeric() || ch[i] == '_') {
                i += 1;
            }
            let word = sub(start, i);
            if i < n && ch[i] == '!' && (i + 1 >= n || ch[i + 1] != '=') {
                i += 1;
                span_into(&mut out, "hl-mac", &format!("{word}!"));
            } else if KEYWORDS.contains(&word.as_str()) {
                span_into(&mut out, "hl-kw", &word);
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                span_into(&mut out, "hl-ty", &word);
            } else {
                esc_into(&mut out, &word);
            }
            continue;
        }

        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
        i += 1;
    }
    out
}
