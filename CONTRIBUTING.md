# Contributing to Rust Book Quiz

English | [中文](CONTRIBUTING.zh.md)

Thanks for helping make the question bank better! Issues and PRs in English or
Chinese are both welcome.

The two most valuable contributions are **reporting a bad question** (wrong
answer, ambiguous options) and **adding new questions**. Both go through the
same machinery: everything under `bank/` is validated by CI on every PR.

## Reporting a bad question

Open an issue with the question `id` (shown as `chNN-qNN` in the JSON; the
in-app section pill and chapter tell you where to look). Say what you believe
is wrong and — ideally — quote the passage of The Book that settles it. Every
question shows a "Read in The Book" link with its source section, so the
ground truth is one click away.

## Adding questions

The JSON schema, file layout, and sync workflow are documented in
[README.md → Question Bank](README.md#question-bank-adding--contributing-questions).
The short version:

```bash
# 1. Edit bank/chNN.json
# 2. Validate + regenerate the embedded snapshot (commit both!)
cd app && cargo run -p quiz-bank -- --sync
# 3. Mechanically verify code-snippet behavior claims with real rustc
cargo run -p quiz-bank --bin code_check
```

CI runs exactly these on your PR (`.github/workflows/validate.yml`), so a PR
that passes locally passes CI.

### What CI checks automatically

- Schema: unique `id`, known `tag`, `difficulty` 1–3, options `A`–`F` (≥3;
  multi-select needs 5), `answer` letters that exist, non-empty texts.
- `section` must be `"N.M Title"` and map to a real section of The Book
  (chapter number must match the file). The app derives the per-question
  "Read in The Book" link from this field — no URL to maintain by hand.
  The TOC snapshot lives in `app/quiz-bank/src/book_toc.rs`.
- `anchor` (optional) must be a lowercase-hyphen URL fragment; when present,
  the source link jumps straight to that heading on the section's page.
- `app/assets/bank.json` must be in sync (forgetting `--sync` fails CI).
- For questions with a `code` field, `code_check` compiles (and runs) the
  snippet with local `rustc` and cross-checks the claim in the correct
  option: "does not compile (E0382)", "panics", "the test fails" are all
  verified for real.

### What needs human judgment: question quality

Automated checks cannot tell a good question from a bad one. Before
submitting, check each question against these:

1. **Exactly one defensible answer.** The strongest failure mode of a quiz is
   an official answer that a careful reader can legitimately dispute. If a
   distractor is "arguably also true", sharpen it until it is strictly wrong.
2. **Distractors are plausible misconceptions**, not filler. The best wrong
   options are things people actually believe (that's what the
   `Misconception` tag is for).
3. **The explanation teaches.** Write it for the reader who just picked the
   wrong option — answer "why did I get this wrong?", don't just confirm the
   right answer. State why the answer is right *and* why the tempting
   distractor is wrong.
4. **Code snippets are self-contained.** They should compile (or fail) the
   way the question claims without invisible context; `code_check` will call
   you out otherwise.
5. **Anchor to the book.** The `section` field is a claim that The Book
   covers this — don't quiz folklore or unstable features.

## Anything else

App code (Leptos frontend, Tauri shell), CI, docs — normal PR flow, no
special rules. For substantial changes, open an issue first so the design
can be discussed before you invest time.
