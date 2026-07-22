//! Link checker: verifies every source link the app can render still works
//! against the live The Book site.
//!
//! For each question it derives the page URL from `section` (via book_toc)
//! exactly like the app does, then checks:
//! - the page responds with HTTP 200 (a renamed/removed section -> 404), and
//! - the question's `anchor`, if any, still exists as an `id="..."` on that
//!   page (The Book restructures headings from time to time).
//! Chapter-level `link` fields from the bank files are checked for 200 too.
//!
//! Needs the network — run with:  cargo run -p quiz-bank --features net --bin link_check
//! HTTPS_PROXY/HTTP_PROXY are respected. Caveat: on corporate networks that
//! re-sign TLS with an in-house CA (MITM proxies), certificate validation
//! fails locally with UnknownIssuer — run it via the link-check.yml workflow
//! (Actions -> Link Check -> Run workflow) instead; GitHub runners are clean.
//! Deliberately NOT part of the PR gate (validate.yml); a scheduled workflow
//! (link-check.yml) runs it weekly so book drift is noticed within days,
//! not when a user hits a dead link.

use quiz_bank::{book_toc, repo_root, Chapter};
use std::collections::{BTreeMap, BTreeSet};

fn agent() -> ureq::Agent {
    // use HTTPS_PROXY/HTTP_PROXY when set (corporate networks); direct connect otherwise
    let mut b = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(20));
    if let Ok(url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("HTTP_PROXY")) {
        if let Ok(p) = ureq::Proxy::new(&url) {
            eprintln!("using proxy from env: {url}");
            b = b.proxy(p);
        }
    }
    b.build()
}

fn fetch(agent: &ureq::Agent, url: &str) -> Result<(u16, String), String> {
    // one retry to ride out transient network hiccups
    for attempt in 0..2 {
        match agent.get(url).call() {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.into_string().map_err(|e| e.to_string())?;
                return Ok((status, body));
            }
            Err(ureq::Error::Status(code, _)) => return Ok((code, String::new())),
            Err(e) if attempt == 0 => {
                eprintln!("  retrying {url}: {e}");
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    unreachable!()
}

fn ids_in(html: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for part in html.split("id=\"").skip(1) {
        if let Some(end) = part.find('"') {
            ids.insert(part[..end].to_string());
        }
    }
    ids
}

fn main() {
    let bank_dir = repo_root().join("bank");
    let mut chapters: Vec<Chapter> = std::fs::read_dir(&bank_dir)
        .expect("bank/ directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .map(|e| {
            let text = std::fs::read_to_string(e.path()).expect("readable bank file");
            serde_json::from_str(&text)
                .unwrap_or_else(|err| panic!("{}: {err} (run validate first)", e.path().display()))
        })
        .collect();
    chapters.sort_by_key(|c| c.chapter);

    // url -> [(question id or chapter-link marker, optional anchor)]
    let mut wanted: BTreeMap<String, Vec<(String, Option<String>)>> = BTreeMap::new();
    for ch in &chapters {
        wanted
            .entry(ch.link.clone())
            .or_default()
            .push((format!("ch{:02} chapter link", ch.chapter), None));
        for q in &ch.questions {
            let url = book_toc::book_url(ch.chapter, &q.section)
                .expect("validate guarantees every section maps");
            wanted.entry(url).or_default().push((q.id.clone(), q.anchor.clone()));
        }
    }

    let mut problems = Vec::new();
    let total_pages = wanted.len();
    let agent = agent();
    for (i, (url, users)) in wanted.iter().enumerate() {
        eprintln!("[{}/{total_pages}] {url}", i + 1);
        match fetch(&agent, url) {
            Ok((200, body)) => {
                let ids = ids_in(&body);
                for (who, anchor) in users {
                    if let Some(a) = anchor {
                        if !ids.contains(a) {
                            problems.push(format!("{who}: anchor #{a} gone from {url}"));
                        }
                    }
                }
            }
            Ok((code, _)) => {
                for (who, _) in users {
                    problems.push(format!("{who}: HTTP {code} for {url}"));
                }
            }
            Err(e) => {
                for (who, _) in users {
                    problems.push(format!("{who}: fetch failed for {url}: {e}"));
                }
            }
        }
        // be polite to doc.rust-lang.org
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    let anchors: usize = wanted.values().flatten().filter(|(_, a)| a.is_some()).count();
    println!("checked {total_pages} pages / {anchors} anchors");
    if problems.is_empty() {
        println!("all links and anchors are alive");
        return;
    }
    println!("{} problem(s):", problems.len());
    for p in &problems {
        println!("  {p}");
    }
    std::process::exit(1);
}
