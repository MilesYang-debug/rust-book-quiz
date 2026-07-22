## What

<!-- One or two sentences. For bank changes: which chapters, how many questions. -->

## Checklist (bank changes)

- [ ] `cd app && cargo run -p quiz-bank -- --sync` passes and the regenerated
      `app/assets/bank.json` is committed
- [ ] `cargo run -p quiz-bank --bin code_check` reports no MISMATCH
- [ ] Each new/changed question has exactly one defensible answer
- [ ] `section` labels are `"N.M Title"` copied from The Book's TOC
- [ ] Explanations say why the answer is right *and* why the distractors are wrong

<!-- Not a bank change? Delete the checklist. See CONTRIBUTING.md for details. -->
