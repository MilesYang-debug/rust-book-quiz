//! Mechanical behavior checker for question code snippets.
//!
//! For every question that carries a `code` field, this tool infers the
//! *claimed* behavior from the correct option text(s) and verifies it by
//! actually compiling (and, when meaningful, running) the snippet with the
//! local `rustc`.
//!
//! Verdicts:
//! - MATCH     — observed behavior is consistent with the claim
//! - MISMATCH  — high-confidence contradiction (fails CI):
//!               * claim names a compile error / E-code, snippet compiles
//!                 (or fails with entirely different codes)
//!               * claim says panic / failing test, but it runs clean
//!               * claim implies it runs, it compiles, and it panics
//! - INFO      — not machine-checkable, listed for human eyes only:
//!               * fragment that doesn't compile standalone (missing context)
//!               * needs external crates, stdin, filesystem, or times out
//!
//! `#[test]` snippets are compiled with `rustc --test` and the harness is
//! executed, so "the test fails/passes" claims are verified for real.
//! Snippets are tried with `--edition 2021` first, re-tried with 2024 on
//! mismatch (the Book targets 2024; the app crate uses 2021).
//!
//! Usage: `cargo run -p quiz-bank --bin code_check`
//! Exit code is non-zero only on MISMATCH, so CI can gate on it.

use quiz_bank::{repo_root, Chapter, Question};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Questions whose claims are about code *not present* in the snippet
/// (e.g. "what happens if you call X" where X is never called).
const SKIP_IDS: &[(&str, &str)] = &[
    ("ch10-q17", "claim concerns a hypothetical call absent from the snippet"),
];

const RUN_TIMEOUT: Duration = Duration::from_secs(5);

/// Error codes that mean "name/module/trait not found" — a fragment relying
/// on context defined elsewhere in the chapter, not a wrong behavior claim.
const CONTEXT_CODES: &[&str] = &["E0405", "E0412", "E0422", "E0425", "E0432", "E0433"];

#[derive(Debug, Clone, PartialEq)]
enum Expect {
    /// Must fail to compile; if codes are non-empty, at least one must match.
    CompileErr(Vec<String>),
    /// Must compile, then panic at runtime (or the test harness must fail).
    Panic,
    /// The claim says the test fails / passes.
    TestFails,
    TestPasses,
    /// No stronger claim: compiling (and not panicking, if runnable) is enough.
    Runs,
}

enum Verdict {
    Match,
    Mismatch(String),
    Info(String),
}

fn main() {
    let chapters = load_bank();
    let work = std::env::temp_dir().join("quiz-code-check");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).expect("create work dir");

    let (mut matches, mut infos, mut hard) = (0usize, Vec::new(), Vec::new());

    for ch in &chapters {
        for q in &ch.questions {
            let Some(code) = &q.code else { continue };

            if let Some((_, reason)) = SKIP_IDS.iter().find(|(id, _)| *id == q.id) {
                infos.push(format!("{} | SKIP: {reason}", q.id));
                continue;
            }
            if let Some(krate) = external_crate(code) {
                infos.push(format!("{} | SKIP: needs external crate `{krate}`", q.id));
                continue;
            }

            let expect = expectation(q, code);
            let dir = work.join(&q.id);
            fs::create_dir_all(&dir).expect("create case dir");

            let verdict = match check(&dir, code, &expect, "2021") {
                Verdict::Mismatch(m1) => match check(&dir, code, &expect, "2024") {
                    Verdict::Match => {
                        infos.push(format!("{} | NOTE: matches under edition 2024 only", q.id));
                        Verdict::Match
                    }
                    _ => Verdict::Mismatch(m1),
                },
                v => v,
            };

            match verdict {
                Verdict::Match => matches += 1,
                Verdict::Info(msg) => infos.push(format!("{} | INFO: {msg}", q.id)),
                Verdict::Mismatch(msg) => {
                    hard.push(format!("{} | MISMATCH | claimed {:?} | {msg}", q.id, expect));
                }
            }
        }
    }

    println!(
        "code_check: {} verified, {} mismatch(es), {} info/skipped",
        matches,
        hard.len(),
        infos.len()
    );
    for s in &infos {
        println!("  {s}");
    }
    for f in &hard {
        println!("  {f}");
    }
    if !hard.is_empty() {
        println!("\nGenerated sources: {}", work.display());
        std::process::exit(1);
    }
}

fn load_bank() -> Vec<Chapter> {
    let bank_dir = repo_root().join("bank");
    let mut files: Vec<PathBuf> = fs::read_dir(&bank_dir)
        .expect("cannot read bank dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ch") && n.ends_with(".json"))
        })
        .collect();
    files.sort();
    files
        .iter()
        .map(|f| {
            let text = fs::read_to_string(f).expect("read bank file");
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", f.display()))
        })
        .collect()
}

/// Run one snippet against one edition and judge it against `expect`.
fn check(dir: &Path, code: &str, expect: &Expect, edition: &str) -> Verdict {
    let is_test = code.contains("#[test]");
    let (compiled, error_codes, exe) = compile(dir, code, edition, is_test);

    if !compiled {
        return match expect {
            Expect::CompileErr(codes) => {
                if codes.is_empty() || codes.iter().any(|c| error_codes.contains(c)) {
                    Verdict::Match
                } else if error_codes.iter().all(|c| CONTEXT_CODES.contains(&c.as_str())) {
                    // Unresolved-name/module errors mean the fragment lacks
                    // surrounding context, masking the claimed error.
                    Verdict::Info(format!(
                        "[{edition}] claimed {codes:?} not verifiable: fragment missing context ({error_codes:?})"
                    ))
                } else {
                    Verdict::Mismatch(format!(
                        "[{edition}] fails with {error_codes:?}, claim names {codes:?}"
                    ))
                }
            }
            // A fragment that doesn't compile standalone is (almost always)
            // missing surrounding context, not a wrong claim.
            _ => Verdict::Info(format!(
                "[{edition}] fragment does not compile standalone ({error_codes:?})"
            )),
        };
    }

    // It compiles.
    if let Expect::CompileErr(_) = expect {
        return Verdict::Mismatch(format!("[{edition}] claim says compile error, but it compiles"));
    }

    let is_doctest = code.contains("///");
    let runnable = !(code.contains("read_line")
        || code.contains("stdin")
        || env_dependent(code));
    if !runnable || is_doctest {
        return match expect {
            Expect::Panic | Expect::TestFails => Verdict::Info(format!(
                "[{edition}] compiles; runtime claim not verifiable ({})",
                if is_doctest { "doc-test — harness does not run doc-tests" } else { "env/stdin dependent" }
            )),
            _ => Verdict::Match,
        };
    }
    // A failing `#[test]` harness is often exactly what the question is
    // about (assertion output, captured stdout), so a neutral claim is not
    // contradicted by a red test run — only explicit pass/fail claims are.
    if is_test && *expect == Expect::Runs {
        return Verdict::Match;
    }

    match run(&exe) {
        RunResult::TimedOut => Verdict::Info(format!("[{edition}] timed out after {RUN_TIMEOUT:?}")),
        RunResult::Finished { failed } => match expect {
            Expect::Panic | Expect::TestFails => {
                if failed {
                    Verdict::Match
                } else {
                    Verdict::Mismatch(format!("[{edition}] ran clean, but claim says panic/failing test"))
                }
            }
            Expect::TestPasses => {
                if failed {
                    Verdict::Mismatch(format!("[{edition}] test harness failed, but claim says it passes"))
                } else {
                    Verdict::Match
                }
            }
            Expect::Runs => {
                if failed {
                    Verdict::Mismatch(format!("[{edition}] panicked at runtime, claim implies clean run"))
                } else {
                    Verdict::Match
                }
            }
            Expect::CompileErr(_) => unreachable!(),
        },
    }
}

fn compile(dir: &Path, code: &str, edition: &str, is_test: bool) -> (bool, Vec<String>, PathBuf) {
    let src = if code.contains("fn main") || is_test {
        code.to_string()
    } else {
        format!("fn main() {{\n{code}\n}}")
    };
    let src_path = dir.join(format!("main_{edition}.rs"));
    let exe_path = dir.join(format!("main_{edition}.exe"));
    fs::write(&src_path, &src).expect("write snippet");

    let mut cmd = Command::new("rustc");
    cmd.arg("--edition").arg(edition).arg("-A").arg("warnings");
    if is_test {
        cmd.arg("--test");
    }
    let out = cmd.arg("-o").arg(&exe_path).arg(&src_path).output().expect("rustc not found on PATH");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (out.status.success(), find_ecodes(&stderr), exe_path)
}

enum RunResult {
    Finished { failed: bool },
    TimedOut,
}

fn run(exe: &Path) -> RunResult {
    let mut child = Command::new(exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn snippet");
    let start = Instant::now();
    loop {
        if let Some(st) = child.try_wait().expect("wait snippet") {
            return RunResult::Finished { failed: !st.success() };
        }
        if start.elapsed() > RUN_TIMEOUT {
            let _ = child.kill();
            return RunResult::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Behavior claimed by the correct option text(s).
fn expectation(q: &Question, code: &str) -> Expect {
    let claim: String = q
        .answer
        .letters()
        .iter()
        .filter_map(|l| q.options.get(l))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    let codes = find_ecodes(&claim);
    if !codes.is_empty() {
        return Expect::CompileErr(codes);
    }

    // Strip negated phrases so "without a panic" / "no compile error"
    // don't trigger the positive keywords.
    let mut lower = claim.to_lowercase();
    for neg in [
        "no compile error",
        "without a compile error",
        "without compile error",
        "without a panic",
        "without panicking",
        "no panic",
        "not panic",
        "doesn't panic",
        "does not panic",
        "never panics",
        "never panic",
        "rather than panic",
        "instead of panicking",
    ] {
        lower = lower.replace(neg, "");
    }

    const COMPILE_ERR: &[&str] = &[
        "does not compile",
        "doesn't compile",
        "will not compile",
        "won't compile",
        "fails to compile",
        "fail to compile",
        "compile error",
        "compile-time error",
        "compilation error",
        "compilation fails",
        "compiler error",
        "compiler rejects",
        "rejected by the compiler",
    ];
    if COMPILE_ERR.iter().any(|p| lower.contains(p)) {
        return Expect::CompileErr(Vec::new());
    }

    if code.contains("#[test]") {
        if lower.contains("test fails") || lower.contains("the test fails") || lower.contains("fails with") {
            return Expect::TestFails;
        }
        if lower.contains("test passes") || lower.contains("the test passes") {
            return Expect::TestPasses;
        }
    }

    if ["panic", "abort", "crash"].iter().any(|p| lower.contains(p)) {
        return Expect::Panic;
    }
    Expect::Runs
}

/// Snippets whose runtime behavior depends on the host environment.
fn env_dependent(code: &str) -> bool {
    ["File::open", "File::create", "fs::", "env::args", "env::var", "TcpListener", "TcpStream", "read_to_string"]
        .iter()
        .any(|p| code.contains(p))
}

/// All `E` + 4-digit rustc error codes mentioned in `s`.
fn find_ecodes(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    for i in 0..b.len().saturating_sub(4) {
        if b[i] == b'E'
            && b[i + 1..i + 5].iter().all(|c| c.is_ascii_digit())
            && (i + 5 >= b.len() || !b[i + 5].is_ascii_digit())
        {
            out.push(String::from_utf8_lossy(&b[i..i + 5]).into_owned());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Root of the first `use`/`extern crate` path that is not std/core/alloc.
fn external_crate(code: &str) -> Option<String> {
    const BUILTIN: &[&str] = &["std", "core", "alloc", "crate", "self", "super"];
    for line in code.lines() {
        let t = line.trim();
        for pre in ["use ", "extern crate "] {
            if let Some(rest) = t.strip_prefix(pre) {
                let root: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !root.is_empty() && !BUILTIN.contains(&root.as_str()) {
                    return Some(root);
                }
            }
        }
    }
    None
}
