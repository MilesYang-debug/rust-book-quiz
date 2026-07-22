//! Static table of contents of The Book, used to derive a per-question
//! "read the source section" link from the `section` field — questions
//! do not store URLs themselves.
//!
//! Generated from rust-lang/book `src/SUMMARY.md`. If the validator
//! reports an unmapped section, either the section label has a typo or
//! The Book gained a section — regenerate/extend this table then.

pub const BOOK_BASE: &str = "https://doc.rust-lang.org/book/";

/// `"N.M"` section number -> page slug.
const SECTIONS: &[(&str, &str)] = &[
    ("1.1", "ch01-01-installation"),
    ("1.2", "ch01-02-hello-world"),
    ("1.3", "ch01-03-hello-cargo"),
    ("3.1", "ch03-01-variables-and-mutability"),
    ("3.2", "ch03-02-data-types"),
    ("3.3", "ch03-03-how-functions-work"),
    ("3.4", "ch03-04-comments"),
    ("3.5", "ch03-05-control-flow"),
    ("4.1", "ch04-01-what-is-ownership"),
    ("4.2", "ch04-02-references-and-borrowing"),
    ("4.3", "ch04-03-slices"),
    ("5.1", "ch05-01-defining-structs"),
    ("5.2", "ch05-02-example-structs"),
    ("5.3", "ch05-03-method-syntax"),
    ("6.1", "ch06-01-defining-an-enum"),
    ("6.2", "ch06-02-match"),
    ("6.3", "ch06-03-if-let"),
    ("7.1", "ch07-01-packages-and-crates"),
    ("7.2", "ch07-02-defining-modules-to-control-scope-and-privacy"),
    ("7.3", "ch07-03-paths-for-referring-to-an-item-in-the-module-tree"),
    ("7.4", "ch07-04-bringing-paths-into-scope-with-the-use-keyword"),
    ("7.5", "ch07-05-separating-modules-into-different-files"),
    ("8.1", "ch08-01-vectors"),
    ("8.2", "ch08-02-strings"),
    ("8.3", "ch08-03-hash-maps"),
    ("9.1", "ch09-01-unrecoverable-errors-with-panic"),
    ("9.2", "ch09-02-recoverable-errors-with-result"),
    ("9.3", "ch09-03-to-panic-or-not-to-panic"),
    ("10.1", "ch10-01-syntax"),
    ("10.2", "ch10-02-traits"),
    ("10.3", "ch10-03-lifetime-syntax"),
    ("11.1", "ch11-01-writing-tests"),
    ("11.2", "ch11-02-running-tests"),
    ("11.3", "ch11-03-test-organization"),
    ("12.1", "ch12-01-accepting-command-line-arguments"),
    ("12.2", "ch12-02-reading-a-file"),
    ("12.3", "ch12-03-improving-error-handling-and-modularity"),
    ("12.4", "ch12-04-testing-the-librarys-functionality"),
    ("12.5", "ch12-05-working-with-environment-variables"),
    ("12.6", "ch12-06-writing-to-stderr-instead-of-stdout"),
    ("13.1", "ch13-01-closures"),
    ("13.2", "ch13-02-iterators"),
    ("13.3", "ch13-03-improving-our-io-project"),
    ("13.4", "ch13-04-performance"),
    ("14.1", "ch14-01-release-profiles"),
    ("14.2", "ch14-02-publishing-to-crates-io"),
    ("14.3", "ch14-03-cargo-workspaces"),
    ("14.4", "ch14-04-installing-binaries"),
    ("14.5", "ch14-05-extending-cargo"),
    ("15.1", "ch15-01-box"),
    ("15.2", "ch15-02-deref"),
    ("15.3", "ch15-03-drop"),
    ("15.4", "ch15-04-rc"),
    ("15.5", "ch15-05-interior-mutability"),
    ("15.6", "ch15-06-reference-cycles"),
    ("16.1", "ch16-01-threads"),
    ("16.2", "ch16-02-message-passing"),
    ("16.3", "ch16-03-shared-state"),
    ("16.4", "ch16-04-extensible-concurrency-sync-and-send"),
    ("17.1", "ch17-01-futures-and-syntax"),
    ("17.2", "ch17-02-concurrency-with-async"),
    ("17.3", "ch17-03-more-futures"),
    ("17.4", "ch17-04-streams"),
    ("17.5", "ch17-05-traits-for-async"),
    ("17.6", "ch17-06-futures-tasks-threads"),
    ("18.1", "ch18-01-what-is-oo"),
    ("18.2", "ch18-02-trait-objects"),
    ("18.3", "ch18-03-oo-design-patterns"),
    ("19.1", "ch19-01-all-the-places-for-patterns"),
    ("19.2", "ch19-02-refutability"),
    ("19.3", "ch19-03-pattern-syntax"),
    ("20.1", "ch20-01-unsafe-rust"),
    ("20.2", "ch20-02-advanced-traits"),
    ("20.3", "ch20-03-advanced-types"),
    ("20.4", "ch20-04-advanced-functions-and-closures"),
    ("20.5", "ch20-05-macros"),
    ("21.1", "ch21-01-single-threaded"),
    ("21.2", "ch21-02-multithreaded"),
    ("21.3", "ch21-03-graceful-shutdown-and-cleanup"),
];

/// Chapter number -> chapter intro page slug (fallback for questions
/// whose section label carries no `N.M` number, e.g. ch02).
const CHAPTERS: &[(u32, &str)] = &[
    (0, "ch00-00-introduction"),
    (1, "ch01-00-getting-started"),
    (2, "ch02-00-guessing-game-tutorial"),
    (3, "ch03-00-common-programming-concepts"),
    (4, "ch04-00-understanding-ownership"),
    (5, "ch05-00-structs"),
    (6, "ch06-00-enums"),
    (7, "ch07-00-managing-growing-projects-with-packages-crates-and-modules"),
    (8, "ch08-00-common-collections"),
    (9, "ch09-00-error-handling"),
    (10, "ch10-00-generics"),
    (11, "ch11-00-testing"),
    (12, "ch12-00-an-io-project"),
    (13, "ch13-00-functional-features"),
    (14, "ch14-00-more-about-cargo"),
    (15, "ch15-00-smart-pointers"),
    (16, "ch16-00-concurrency"),
    (17, "ch17-00-async-await"),
    (18, "ch18-00-oop"),
    (19, "ch19-00-patterns"),
    (20, "ch20-00-advanced-features"),
    (21, "ch21-00-final-project-a-web-server"),
];

/// Leading `"N.M"` number of a section label, if present.
fn section_number(section: &str) -> Option<&str> {
    let tok = section.split_whitespace().next()?;
    let (a, b) = tok.split_once('.')?;
    let is_num = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    (is_num(a) && is_num(b)).then_some(tok)
}

/// URL of the section of The Book a question came from, or `None` when
/// the label maps to nothing (validator treats that as an error).
pub fn book_url(chapter: u32, section: &str) -> Option<String> {
    match section_number(section) {
        Some(num) => {
            let (ch, _) = num.split_once('.')?;
            if ch.parse::<u32>().ok()? != chapter {
                return None; // section number belongs to another chapter
            }
            let slug = SECTIONS.iter().find(|(n, _)| *n == num)?.1;
            Some(format!("{BOOK_BASE}{slug}.html"))
        }
        None => {
            let slug = CHAPTERS.iter().find(|(c, _)| *c == chapter)?.1;
            Some(format!("{BOOK_BASE}{slug}.html"))
        }
    }
}
