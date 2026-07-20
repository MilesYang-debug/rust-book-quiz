//! Bank validator CLI. Run from anywhere inside the repo:
//!   cargo run -p quiz-bank              validate bank/*.json
//!   cargo run -p quiz-bank -- --sync    validate, then regenerate app/assets/bank.json

use quiz_bank::{repo_root, sync_snapshot, validate_dir};

fn main() {
    let sync = std::env::args().any(|a| a == "--sync");
    let root = repo_root();
    let bank_dir = root.join("bank");

    let report = validate_dir(&bank_dir);
    println!(
        "Loaded {} files, {} chapter entries.",
        report.files, report.chapters
    );
    println!("Total questions: {}", report.questions);

    if !report.problems.is_empty() {
        println!("\n{} problem(s):", report.problems.len());
        for p in &report.problems {
            println!("  - {p}");
        }
        std::process::exit(1);
    }
    println!("All checks passed.");

    if sync {
        let out = root.join("app/assets/bank.json");
        match sync_snapshot(&bank_dir, &out) {
            Ok(()) => println!("Snapshot written: {}", out.display()),
            Err(e) => {
                eprintln!("Snapshot failed: {e}");
                std::process::exit(1);
            }
        }
    }
}
