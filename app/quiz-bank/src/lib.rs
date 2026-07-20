//! Question-bank data model and validator.
//!
//! The `Chapter`/`Question`/`Answer` types here are the single source of
//! truth for the bank schema: the app deserializes with them at runtime
//! (see `app/src/model.rs`), and the validator parses with the exact same
//! types — so "passes validation" always means "the app can load it".
//!
//! Entry points:
//! - `cargo run -p quiz-bank`            validate bank/*.json
//! - `cargo run -p quiz-bank -- --sync`  validate, then regenerate app/assets/bank.json
//! - `cargo test -p quiz-bank`           same validation as a test gate

use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Chapter {
    pub chapter: u32,
    pub title: String,
    #[serde(default)]
    pub link: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Question {
    pub id: String,
    pub section: String,
    pub tag: String,
    pub difficulty: u8,
    pub prompt: String,
    #[serde(default)]
    pub code: Option<String>,
    /// BTreeMap keeps option letters in A..E order.
    pub options: BTreeMap<String, String>,
    pub answer: Answer,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Single(String),
    Multi(Vec<String>),
}

impl Answer {
    pub fn is_multi(&self) -> bool {
        matches!(self, Answer::Multi(_))
    }

    /// Sorted answer letters, e.g. ["A", "C"].
    pub fn letters(&self) -> Vec<String> {
        match self {
            Answer::Single(s) => vec![s.clone()],
            Answer::Multi(v) => {
                let mut v = v.clone();
                v.sort();
                v
            }
        }
    }
}

pub const TAGS: [&str; 5] = [
    "Concept",
    "Behavior",
    "Code Output",
    "Spot the Bug",
    "Misconception",
];

pub struct Report {
    pub files: usize,
    pub chapters: usize,
    pub questions: usize,
    pub problems: Vec<String>,
}

/// Repository root, resolved from this crate's location (app/quiz-bank).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `chNN.json` files in `dir`, sorted by name.
fn bank_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read bank dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| {
                    n.strip_prefix("ch")
                        .and_then(|r| r.strip_suffix(".json"))
                        .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
                })
        })
        .collect();
    files.sort();
    files
}

/// Validate every `bank/chNN.json` in `dir` with the app's own schema types,
/// then apply the business rules (unique ids, tag set, difficulty range,
/// option/answer consistency).
pub fn validate_dir(dir: &Path) -> Report {
    let files = bank_files(dir);
    let mut problems = Vec::new();
    let mut chapters = Vec::new();

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                problems.push(format!("{name}: cannot read — {e}"));
                continue;
            }
        };
        // Typed parse — exactly what the app does at runtime.
        match serde_json::from_str::<Chapter>(&text) {
            Ok(ch) => chapters.push(ch),
            Err(e) => problems.push(format!("{name}: does not parse as a Chapter — {e}")),
        }
    }

    let mut seen_ids = HashSet::new();
    let mut questions = 0;

    for ch in &chapters {
        let label = format!("ch{:02}", ch.chapter);
        if ch.title.trim().is_empty() {
            problems.push(format!("{label}: empty title"));
        }
        if ch.questions.is_empty() {
            problems.push(format!("{label}: no questions"));
            continue;
        }
        questions += ch.questions.len();

        for q in &ch.questions {
            let mut err = |msg: String| problems.push(format!("{label}/{}: {msg}", q.id));

            if q.id.trim().is_empty() {
                problems.push(format!("{label}/?: missing id"));
                continue;
            }
            if !seen_ids.insert(q.id.clone()) {
                err("duplicate id".into());
            }
            if q.prompt.trim().is_empty() {
                err("empty prompt".into());
            }
            if q.section.trim().is_empty() {
                err("empty section".into());
            }
            if !TAGS.contains(&q.tag.as_str()) {
                err(format!("bad tag \"{}\"", q.tag));
            }
            if !(1..=3).contains(&q.difficulty) {
                err(format!("bad difficulty {}", q.difficulty));
            }
            if q.explanation.trim().is_empty() {
                err("empty explanation".into());
            }
            if let Some(code) = &q.code {
                if code.trim().is_empty() {
                    err("code field present but empty".into());
                }
            }

            let keys: Vec<&String> = q.options.keys().collect();
            let valid_keys = keys
                .iter()
                .all(|k| k.len() == 1 && matches!(k.as_bytes()[0], b'A'..=b'F'));
            if !valid_keys || keys.len() < 3 {
                err(format!("bad option keys {keys:?}"));
            }
            for (k, v) in &q.options {
                if v.trim().is_empty() {
                    err(format!("empty option {k}"));
                }
            }

            match &q.answer {
                Answer::Single(a) => {
                    if !q.options.contains_key(a) {
                        err(format!("answer \"{a}\" not in options"));
                    }
                }
                Answer::Multi(v) => {
                    if v.len() < 2 {
                        err("multi answer has <2 entries".into());
                    }
                    for a in v {
                        if !q.options.contains_key(a) {
                            err(format!("answer \"{a}\" not in options"));
                        }
                    }
                    if v.iter().collect::<HashSet<_>>().len() != v.len() {
                        err("duplicate answer letters".into());
                    }
                    if q.options.len() < 5 {
                        err(format!(
                            "multi-select should have 5 options, has {}",
                            q.options.len()
                        ));
                    }
                }
            }
        }
    }

    Report {
        files: files.len(),
        chapters: chapters.len(),
        questions,
        problems,
    }
}

/// Regenerate the embedded snapshot (`app/assets/bank.json`) from `bank/*.json`.
/// Files are concatenated as raw JSON values (field order preserved), sorted by
/// chapter number, and written with 1-space indent — same shape the old Node
/// one-liner produced.
pub fn sync_snapshot(bank_dir: &Path, out_file: &Path) -> std::io::Result<()> {
    let mut values: Vec<serde_json::Value> = bank_files(bank_dir)
        .iter()
        .map(|p| {
            let text = fs::read_to_string(p)?;
            serde_json::from_str(&text)
                .map_err(|e| std::io::Error::other(format!("{}: {e}", p.display())))
        })
        .collect::<std::io::Result<_>>()?;
    values.sort_by_key(|v| v["chapter"].as_i64().unwrap_or(i64::MAX));

    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(&values, &mut ser).map_err(std::io::Error::other)?;
    fs::write(out_file, buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `cargo test -p quiz-bank` gates the bank exactly like the CLI does.
    #[test]
    fn bank_is_valid() {
        let report = validate_dir(&repo_root().join("bank"));
        assert!(report.files > 0, "no bank files found");
        assert!(
            report.problems.is_empty(),
            "{} problem(s):\n  {}",
            report.problems.len(),
            report.problems.join("\n  ")
        );
    }
}
