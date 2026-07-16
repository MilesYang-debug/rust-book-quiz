use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SCORES_KEY: &str = "rustQuizRs.scores";
const WRONG_KEY: &str = "rustQuizRs.wrong";
const THEME_KEY: &str = "rustQuizRs.theme";

pub fn theme_light() -> bool {
    LocalStorage::get::<String>(THEME_KEY).map(|s| s == "light").unwrap_or(false)
}

pub fn save_theme(light: bool) {
    let _ = LocalStorage::set(THEME_KEY, if light { "light" } else { "dark" });
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub best: u32,
    pub last: u32,
}

pub fn scores() -> HashMap<u32, ScoreEntry> {
    LocalStorage::get(SCORES_KEY).unwrap_or_default()
}

pub fn save_score(chapter: u32, percent: u32) {
    let mut s = scores();
    let entry = s.entry(chapter).or_default();
    entry.last = percent;
    entry.best = entry.best.max(percent);
    let _ = LocalStorage::set(SCORES_KEY, &s);
}

/// Wrong answer book: question id -> times missed.
pub fn wrong_book() -> HashMap<String, u32> {
    LocalStorage::get(WRONG_KEY).unwrap_or_default()
}

pub fn record_wrong(id: &str) {
    let mut w = wrong_book();
    *w.entry(id.to_string()).or_insert(0) += 1;
    let _ = LocalStorage::set(WRONG_KEY, &w);
}

pub fn clear_wrong(id: &str) {
    let mut w = wrong_book();
    if w.remove(id).is_some() {
        let _ = LocalStorage::set(WRONG_KEY, &w);
    }
}

pub fn clear_all_wrong() {
    let _ = LocalStorage::set(WRONG_KEY, &HashMap::<String, u32>::new());
}
