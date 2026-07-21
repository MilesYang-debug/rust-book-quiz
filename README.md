<div align="center">

<img src="app/ferris.png" width="90" alt="Ferris"/>

# Rust Book Quiz

**A quiz app for *The Rust Programming Language* (the official Rust Book)**

One Rust codebase, four targets: Windows / Linux desktop · Android · web browser
No JS framework · no backend · works offline

[![Rust](https://img.shields.io/badge/Rust-100%25-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/Leptos-0.6-ef3939)](https://leptos.dev/)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Questions](https://img.shields.io/badge/questions-394-blue)](bank/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

[中文](README.zh.md) | English | [🎮 Try it online](https://MilesYang-debug.github.io/rust-book-quiz/)

</div>

- **394 questions** covering the whole book, ch1–ch20 (English prompts, faithful to the official text)
- Three modes: **chapter practice** (per-question instant grading + explanations), **random mock exam** (submit the whole paper, graded at once, with review), and a **wrong-answer book** (misses are collected automatically, removed once answered correctly)
- Question types: single choice / multiple choice / code-output prediction / spot-the-bug, tagged with difficulty and book section
- Desktop supports **adding questions without recompiling**: drop JSON files into a `bank/` directory next to the executable and restart

## Where This Fits in the Rust Learning Path

The official learning resources each cover one part of the loop; this project fills the **self-testing** slot none of them covers:

| Resource | How you learn | What it trains |
|---|---|---|
| [The Book](https://doc.rust-lang.org/book/) | Systematic reading | Building the concept framework |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Reading runnable examples | Recognizing idiomatic code |
| [rustlings](https://github.com/rust-lang/rustlings) | Fixing code until it compiles | Hands-on fights with the borrow checker |
| **This project** | Quizzes + explanations + wrong-answer book | **Verifying understanding, exposing misconceptions** |

Reading alone fades fast, and rustlings always has the compiler as a crutch. The **Code Output** (predict without running) and **Misconception** question types here force you to simulate Rust's semantics with no compiler feedback — active-recall testing, which is how counter-intuitive details like ownership moves, `Deref` coercion, and `match` exhaustiveness actually stick. Missed questions are collected automatically and leave the book once answered correctly, closing the review loop.

Recommended usage (questions are aligned to the Book's chapters and sections, so this slots straight into a reading schedule):

> **Read a Book chapter → quiz that chapter here → re-read the sections you missed → get hands-on with rustlings / a small project**

Quizzing is *testing*, writing code is *training* — treat this as the checkpoint between reading and building.

## Quick Start (no build required)

| Platform | Get it | How to use |
|---|---|---|
| **Online** | [Open in your browser](https://MilesYang-debug.github.io/rust-book-quiz/) | Works immediately; progress is stored in your browser |
| **Windows** | Download `RustBookQuiz.exe` from [Releases](../../releases) | Double-click to run (~6MB, uses the system WebView2) |
| **Linux (Debian/Ubuntu)** | Download `RustBookQuiz_amd64.deb` from [Releases](../../releases) | `sudo apt install ./RustBookQuiz_amd64.deb` |
| **Linux (any distro)** | Download `RustBookQuiz_x86_64.AppImage` from [Releases](../../releases) | `chmod +x`, then run |
| **Linux (bare binary)** | Download `RustBookQuiz-linux` from [Releases](../../releases) | Requires `libwebkit2gtk-4.1`; `chmod +x`, then run |
| **Android** | Download `RustBookQuiz.apk` from [Releases](../../releases) | Install on your phone (~12MB, arm64, allow "unknown sources") |
| **Self-hosted web** | Download `RustBookQuiz-web.zip` from [Releases](../../releases) | Unzip onto any static file server, see [4. Web](#4-web) |

All artifacts are built automatically by [GitHub Actions](#automated-builds-and-releases-github-actions); if you'd rather build them yourself, see the [manual build guide](#manual-build-guide) at the end.

## Question Bank: Adding / Contributing Questions

### Bank format (bank/chNN.json)

One JSON file per chapter, containing a single Chapter object:

```json
{
  "chapter": 11,
  "title": "Writing Automated Tests",
  "link": "https://doc.rust-lang.org/book/ch11-00-testing.html",
  "questions": [
    {
      "id": "ch11-q01",
      "section": "11.1 How to Write Tests",
      "tag": "Concept",
      "difficulty": 1,
      "prompt": "Question text; inline code goes in `backticks`",
      "code": "Optional Rust snippet; newlines as \\n, quotes as \\\"",
      "options": { "A": "...", "B": "...", "C": "...", "D": "..." },
      "answer": "B",
      "explanation": "..."
    }
  ]
}
```

- `tag`: Concept | Behavior | Code Output | Spot the Bug | Misconception
- `difficulty`: 1 easy / 2 medium / 3 hard
- Multiple choice: `"answer": ["A","C"]`, with 5 options A–E

### Workflow for adding questions

**Desktop (zero compilation)**: write `bank/chNN.json` → validate with `cd app && cargo run -p quiz-bank` → put the `bank/` directory next to the executable → restart the app.

**Regenerate the embedded snapshot** (affects the APK, the web build, and the fallback when no `bank/` directory exists; run after editing the bank, then rebuild):

```bash
cd app && cargo run -p quiz-bank -- --sync    # validates, then regenerates app/assets/bank.json
```

> The validator shares the app's own serde data types (`app/quiz-bank`) — passing validation guarantees the app can load the file.

PRs adding questions are welcome — just edit the JSON under `bank/` and make sure `cargo run -p quiz-bank` (or `cargo test -p quiz-bank`) passes.

## Development Workflow

```bash
cd app && trunk serve            # hot-reload dev server (http://127.0.0.1:8080, uses the embedded-bank fallback)
cargo check --target wasm32-unknown-unknown        # front-end type check (run inside app/)
cd src-tauri && cargo check                        # shell type check
cargo run -p quiz-bank                             # bank validation (inside app/; --sync regenerates the snapshot)
cargo test -p quiz-bank                            # same validation as a test (CI gate)
```

> Prerequisites: stable Rust toolchain + the `wasm32-unknown-unknown` target + trunk — see [Manual build guide · Common setup](#common-setup-once-for-every-platform). That's all day-to-day development needs; **no Node.js required**.

### Data storage

Progress lives in each platform's WebView localStorage (independent, not synced):
`rustQuizRs.scores` (best/latest chapter scores), `rustQuizRs.wrong` (wrong-answer book), `rustQuizRs.theme` (dark/light theme).

## Architecture

```mermaid
flowchart LR
    bank[("📚 bank/chNN.json<br/>bank source data")]
    snap["assets/bank.json<br/>embedded snapshot (generated)"]
    front["Leptos 0.6 front end<br/>Rust → WASM"]
    shell["Tauri 2 shell"]
    win["Windows<br/>WebView2"]
    linux["Linux<br/>WebKitGTK"]
    android["Android<br/>system WebView"]
    web["Web<br/>pure static · no shell"]

    bank -->|sync snapshot| snap
    snap -- embedded at compile time --> front
    bank -. hot-loaded at runtime on desktop .-> shell
    front --> shell
    front --> web
    shell --> win
    shell --> linux
    shell --> android
```

| Layer | Tech | Responsibilities |
|---|---|---|
| **Front end** | Leptos 0.6 (Rust → WASM) | All business logic (grading / multi-select / question drawing / wrong-answer book) · pure-Rust syntax highlighter (no JS deps) · embedded Inter + JetBrains Mono fonts for identical rendering everywhere |
| **Shell** | Tauri 2 | `load_bank_files` / `open_url` / window-control commands · WebView2 on Windows, WebKitGTK on Linux, system WebView on Android · the web build runs without a shell |
| **Bank** | JSON (one file per chapter) | Single source of truth `bank/chNN.json` · desktop reads `bank/` next to the executable at runtime · mobile / web / fallback use the compile-time embedded snapshot |

## Repository Layout

| Path | Description |
|---|---|
| [`bank/`](bank/) | Question bank source data `chNN.json`, one file per chapter (**single source of truth**) |
| [`.github/workflows/`](.github/workflows/) | CI: `release.yml` (four-platform artifacts) · `deploy-pages.yml` (online version) · `validate.yml` (bank gate) |
| [`app/quiz-bank/`](app/quiz-bank/) | Bank schema types + validator (`cargo run -p quiz-bank`) |
| [`app/index.html`](app/index.html) | trunk build entry (declares font/icon asset copies) |
| [`app/style.css`](app/style.css) | Dark tech theme + light theme + mobile media queries |
| [`app/fonts/`](app/fonts/) · [`ferris.png`](app/ferris.png) | Embedded assets (OFL fonts / hand-drawn Ferris) |
| [`app/assets/bank.json`](app/assets/bank.json) | Embedded bank snapshot (**generated — do not edit by hand**) |
| [`app/src/main.rs`](app/src/main.rs) | App shell: routing / title bar / theme / drawer |
| [`app/src/model.rs`](app/src/model.rs) | Bank loading, syntax highlighting (schema types re-used from quiz-bank) |
| [`app/src/storage.rs`](app/src/storage.rs) | localStorage persistence (scores / wrong answers / theme) |
| [`app/src/views.rs`](app/src/views.rs) | Sidebar / Home / Exam / Wrong / Quiz components |
| [`app/src-tauri/src/lib.rs`](app/src-tauri/src/lib.rs) | Tauri shell entry + all commands (mobile entry point lives here) |
| [`app/src-tauri/src/main.rs`](app/src-tauri/src/main.rs) | Desktop entry (calls `lib::run`) |
| [`app/src-tauri/tauri.conf.json`](app/src-tauri/tauri.conf.json) | Window config + bundling config (deb / AppImage) |
| [`app/src-tauri/capabilities/`](app/src-tauri/capabilities/) | Window drag permission |
| [`app/src-tauri/icons/`](app/src-tauri/icons/) | `icon.ico` (Windows) / `icon.png` (mobile + Linux bundling) |
| [`app/src-tauri/gen/android/`](app/src-tauri/gen/android/) | Android project (committed — no need to re-init) |

## Automated Builds and Releases (GitHub Actions)

Day-to-day development does **not** require setting up all four platform toolchains locally — CI does it all:

| Workflow | Trigger | Output |
|---|---|---|
| [release.yml](.github/workflows/release.yml) | Push a `v*` tag | exe / deb / AppImage / Linux bare binary / apk / web zip, attached to Releases automatically |
| [deploy-pages.yml](.github/workflows/deploy-pages.yml) | Push to main (changes under app/) | Online version auto-deployed to GitHub Pages |
| [validate.yml](.github/workflows/validate.yml) | push / PR touching bank/ or the validator | Validates the bank + checks the embedded snapshot is in sync (fails if `--sync` was forgotten) |

Shipping a release is just:

```bash
git tag v0.1.0
git push origin v0.1.0
# ✅ Wait for CI (~20 minutes); all artifacts appear on the Releases page
```

One-time setup (once after creating the repo): **Settings → Pages → Source → "GitHub Actions"** — the online version goes live.

---

# Manual Build Guide

> Each platform section is **self-contained**: run "Common setup" plus the section for your platform and you get the artifact.
> Every step lists its ✅ expected result; if yours doesn't match, check [Troubleshooting](#troubleshooting) first.

## Common setup (once, for every platform)

**Step 1**: Install the Rust toolchain (stable). Follow <https://rustup.rs/>, then verify:

```bash
rustc --version        # ✅ prints something like rustc 1.8x.0
cargo --version        # ✅ prints something like cargo 1.8x.0
```

**Step 2**: Add the WASM compilation target (the front end compiles to WebAssembly):

```bash
rustup target add wasm32-unknown-unknown
rustup target list --installed | grep wasm
# ✅ prints wasm32-unknown-unknown
```

**Step 3**: Install the trunk bundler:

```bash
cargo install trunk --locked      # first build takes ~10 minutes
trunk --version                   # ✅ prints something like trunk 0.2x.x
```

**Step 4**: Clone the repo and build the front end (**every platform artifact depends on this**):

```bash
git clone https://github.com/MilesYang-debug/rust-book-quiz.git
cd rust-book-quiz/app
trunk build --release
ls dist/
# ✅ dist/ contains index.html, *.wasm, *.js, fonts/, etc. (~4MB)
```

> This project does not depend on Node.js — bank validation and snapshot generation are Rust too (`cargo run -p quiz-bank`).

## 1. Windows desktop

Prerequisite: complete [Common setup](#common-setup-once-for-every-platform).

```bash
cd app/src-tauri
cargo build --release
# ✅ artifact: target/release/rust-book-quiz-desktop.exe
```

Copy the exe out and rename it `RustBookQuiz.exe` — it's fully self-contained (front end, fonts, and embedded bank included). Placing a `bank/` directory next to it overrides the embedded bank (see [Workflow for adding questions](#workflow-for-adding-questions)).

## 2. Linux desktop (deb / AppImage / bare binary)

Prerequisite: complete [Common setup](#common-setup-once-for-every-platform); the steps below must run **on Linux**.

**Step 1**: Install system dependencies (Ubuntu/Debian shown):

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libgtk-3-dev librsvg2-dev libxdo-dev
```

**Step 2**: Install the Tauri CLI (pick one):

```bash
npm install -g @tauri-apps/cli@^2     # fast, prebuilt binary (subsequent commands: tauri ...)
# or
cargo install tauri-cli --locked      # slow (~10 minutes), no Node needed (subsequent commands: cargo tauri ...)
```

**Step 3**: Build and bundle:

```bash
cd app
cargo tauri build --bundles deb appimage    # npm variant: tauri build --bundles deb appimage
```

✅ Artifacts (under `app/src-tauri/target/release/`):

| Path | Notes |
|---|---|
| `bundle/deb/*.deb` | Debian/Ubuntu package: `sudo apt install ./xxx.deb` |
| `bundle/appimage/*.AppImage` | Portable, no install: `chmod +x`, then run |
| `rust-book-quiz-desktop` | Bare binary; requires `libwebkit2gtk-4.1` on the system |

> ⚠️ Running only `cargo build --release` gives you **just the bare binary — no deb**. deb/AppImage require
> `cargo tauri build` (bundling is already enabled in the `bundle` section of `tauri.conf.json`).

## 3. Android

Prerequisite: complete [Common setup](#common-setup-once-for-every-platform). The environment setup has many steps, but each is one-time.

### 3.1 One-time environment setup

**Step 1**: Install JDK 17 (any distribution, e.g. [Temurin](https://adoptium.net/)), then verify:

```bash
java -version    # ✅ output contains "17."
```

**Step 2**: Install the Android SDK command-line tools. Download commandline-tools from
<https://developer.android.com/studio#command-line-tools-only> and unpack into this layout
(**the directory must be named `latest`**):

```
<sdk>/cmdline-tools/latest/bin/sdkmanager
```

**Step 3**: Accept licenses and install components (~3GB download):

```bash
cd <sdk>
yes | cmdline-tools/latest/bin/sdkmanager --licenses
cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-36" \
  "build-tools;36.0.0" "ndk;27.1.12297006"
```

> On Windows, use Git Bash (and `sdkmanager.bat` instead of `sdkmanager`). If Gradle can't reach the
> network behind a corporate proxy, see the last two rows of [Troubleshooting](#troubleshooting).

**Step 4**: Rust Android target + Tauri CLI:

```bash
rustup target add aarch64-linux-android
cargo install tauri-cli --locked    # or cargo binstall tauri-cli (prebuilt, seconds)
```

> ⚠️ Android builds **require the cargo edition of tauri-cli**: the gradle task in `gen/android`
> (buildSrc/BuildTask.kt) calls back into `cargo tauri`, so the npm `@tauri-apps/cli` alone fails
> with ``no such command: `tauri` ``.

**Step 5**: Environment variables (export before each build, or set them system-wide):

```bash
export JAVA_HOME=<jdk17 install dir>
export ANDROID_HOME=<sdk dir>
export NDK_HOME=$ANDROID_HOME/ndk/27.1.12297006
```

### 3.2 Building the APK

```bash
cd app && trunk build --release     # front-end assets (embedded into the APK)
cd src-tauri
cargo tauri android build --apk --target aarch64
# ✅ artifact: gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk
```

> - The `gen/android/` project is committed to this repo — do **not** run `cargo tauri android init`
> - On Linux/macOS, if you get gradlew Permission denied: `chmod +x gen/android/gradlew`
> - ⚠️ Don't use `--debug`: the debug build carries unstripped symbols (~238MB) vs ~12MB for release

### 3.3 Signing (a release APK must be signed to install)

```bash
# Use the debug keystore (auto-generated by gradle at ~/.android/debug.keystore on first build) — fine for personal use
$ANDROID_HOME/build-tools/36.0.0/apksigner sign \
  --ks ~/.android/debug.keystore --ks-pass pass:android \
  --key-pass pass:android --ks-key-alias androiddebugkey \
  --out RustBookQuiz.apk app-arm64-release-unsigned.apk

$ANDROID_HOME/build-tools/36.0.0/apksigner verify RustBookQuiz.apk   # ✅ no errors means a valid signature
```

> Publishing to an app store requires a proper key generated with `keytool -genkeypair`.

Transfer the APK to your phone and install (allow "unknown sources"). The phone uses the embedded bank; the UI switches to the mobile layout automatically (☰ drawer sidebar, system status bar, single-column touch layout).

### 3.4 Mobile notes when changing the shell code

- The entry point is in `lib.rs`: `#[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run()`; `main.rs` is just a thin desktop wrapper
- Window APIs (minimize/maximize, etc.) must be gated with `#[cfg(desktop)]` — they don't exist on the Android target, otherwise you get E0599
- External links go through `tauri-plugin-opener` (works on desktop and Android alike); the command lives in `open_url` in `lib.rs`
- Mobile requires `icons/icon.png` (desktop uses .ico, mobile uses .png)

## 4. Web

The front end is a plain static WASM app: the `app/dist/` produced in [Common setup step 4](#common-setup-once-for-every-platform) is the entire deployable — **no backend, no database**; anything that serves static files works.

Local preview:

```bash
python -m http.server 8000 --directory app/dist
# ✅ open http://localhost:8000 and start quizzing
```

Deploying to a static server (nginx example):

```nginx
server {
    listen 80;
    root /var/www/rust-book-quiz;    # contents of dist/
    index index.html;
    types { application/wasm wasm; } # older nginx needs the wasm MIME type added manually
    gzip on;                         # WASM is large; compression pays off
    gzip_types application/wasm application/javascript text/css;
}
```

Deploying under a **subpath** (e.g. GitHub Pages at `https://MilesYang-debug.github.io/rust-book-quiz/`) requires setting the asset prefix at build time:

```bash
trunk build --release --public-url /rust-book-quiz/
```

Behavioral differences of the web build:

- The bank is fixed to the compile-time embedded snapshot (`app/assets/bank.json`) — after editing questions you must rebuild and redeploy; there's no desktop-style `bank/` hot loading
- The title bar hides the window controls automatically (minimize/close are the browser's job); theme switching stays
- Progress lives in each visitor's own browser localStorage; the server stores no user data
- Mobile browsers get the mobile layout automatically (same responsive CSS as the APK)

---

# Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `can't find crate for core` (wasm target) | WASM target not installed: `rustup target add wasm32-unknown-unknown` |
| `error: linking with cc failed` / `webkit2gtk` not found | Missing Linux system deps; re-run the `apt install` from [2. Linux step 1](#2-linux-desktop-deb--appimage--bare-binary) |
| `icon.ico not found` | `app/src-tauri/icons/icon.ico` is missing (required by tauri-build for Windows resources); restore it from the repo |
| `cargo tauri build` produces no deb | Make sure the command is `cargo tauri build`, not `cargo build`, and that `bundle.active: true` in `tauri.conf.json` |
| AppImage bundling hangs downloading tools | Tauri downloads linuxdeploy on first bundling; retry behind a proxy if the network is blocked |
| Android build: `gradlew: Permission denied` | gradlew lost its executable bit (common when committed from Windows): `chmod +x app/src-tauri/gen/android/gradlew`; fix it in git for good with `git update-index --chmod=+x app/src-tauri/gen/android/gradlew` |
| Android build: ``no such command: `tauri` `` | gradle calls back into `cargo tauri`; the npm CLI is not enough: `cargo install tauri-cli --locked` (or `cargo binstall tauri-cli`) |
| Android build fails with E0599 on window methods | Desktop window APIs missing the `#[cfg(desktop)]` gate; see [3.4 Mobile notes](#34-mobile-notes-when-changing-the-shell-code) |
| Web deploy shows a blank page | Subpath deploy missing `--public-url /<subpath>/`; or the server serves .wasm with the wrong MIME type (must be `application/wasm`) |
| Web icon/fonts 404 | Assets must be referenced with relative paths (`ferris.png`, `fonts/...`); absolute paths like `/xxx` point at the domain root under a subpath deploy |
| Gradle can't reach the network (corporate proxy) | The JVM ignores the `HTTPS_PROXY` env var. Write `~/.gradle/gradle.properties` with four lines `systemProp.http(s).proxyHost/proxyPort=<proxy host/port>`, plus `systemProp.http(s).nonProxyHosts=localhost\|127.0.0.1\|10.*\|172.16.*\|192.168.*` |
| Gradle wrapper fails to download the distribution (proxy) | The wrapper download runs before proxy config is read: manually `curl -L -o <local dir>/gradle-8.14.3-bin.zip https://services.gradle.org/distributions/gradle-8.14.3-bin.zip`, then temporarily point `distributionUrl` in `gen/android/gradle/wrapper/gradle-wrapper.properties` at `file:///<local path>` (**don't commit this change** — `git update-index --skip-worktree` makes git ignore it) |

# Known Limitations

- The APK is signed with a debug key and can't be published to app stores as-is
- Progress doesn't sync across platforms (no cloud sync)
- ch21 isn't covered (the web-server capstone — a pure project walkthrough, not quiz material)

# Asset Licenses

- Fonts Inter and JetBrains Mono: SIL OFL 1.1, redistributable with the app
- The Ferris icon is drawn programmatically by this project (the official Ferris itself is CC0)
- Questions are written originally, based on [the official book](https://doc.rust-lang.org/book/)
