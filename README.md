# Rust Book Quiz 🦀

中文 | [English](README.en.md)

《The Rust Programming Language》（官方 Rust Book）配套刷题应用。**一套 Rust 代码，四端交付**：Windows / Linux 桌面 + Android + Web 浏览器。无 JS 框架、无后端、离线可用。

- **394 道题**覆盖全书 ch1–ch20（英文题干，贴合官方文档原文）
- 三种模式：**章节练习**（逐题即时判分 + 解析）、**随机模拟考**（整卷交卷统一判分 + 回看）、**错题本**（答错自动收录，答对移出）
- 题型：单选 / 多选 / 代码输出预测 / 找错题，附难度与章节小节标签
- 桌面端**加题免编译**：往可执行文件旁的 `bank/` 目录放 JSON，重启即生效

## 快速开始（不编译，直接用）

| 平台 | 获取方式 | 使用方法 |
|---|---|---|
| **在线体验** | [浏览器直接访问](https://MilesYang-debug.github.io/rust-book-quiz/) | 打开即用，进度存本地浏览器 |
| **Windows** | [Releases](../../releases) 下载 `RustBookQuiz.exe` | 双击运行（约 6MB，依赖系统自带 WebView2） |
| **Linux (Debian/Ubuntu)** | [Releases](../../releases) 下载 `RustBookQuiz_amd64.deb` | `sudo apt install ./RustBookQuiz_amd64.deb` |
| **Linux (任意发行版)** | [Releases](../../releases) 下载 `RustBookQuiz_x86_64.AppImage` | `chmod +x` 后直接运行 |
| **Linux (裸二进制)** | [Releases](../../releases) 下载 `RustBookQuiz-linux` | 需系统装有 `libwebkit2gtk-4.1`，`chmod +x` 后运行 |
| **Android** | [Releases](../../releases) 下载 `RustBookQuiz.apk` | 传到手机安装（约 12MB，arm64，需允许"未知来源"） |
| **自部署 Web 版** | [Releases](../../releases) 下载 `RustBookQuiz-web.zip` | 解压丢到任意静态服务器，见 [4. Web 版](#4-web-版) |

所有产物由 [GitHub Actions 自动构建](#自动构建与发布github-actions)；想自己编译的话见文末的[手工编译指南](#手工编译指南)。

## 题库：自己加题 / 贡献题目

### 题库规范（bank/chNN.json）

每章一个 JSON 文件，单个 Chapter 对象：

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
      "prompt": "题干，可用 `反引号` 标记行内代码",
      "code": "可选 Rust 片段；换行写 \\n，引号写 \\\"",
      "options": { "A": "...", "B": "...", "C": "...", "D": "..." },
      "answer": "B",
      "explanation": "..."
    }
  ]
}
```

- `tag`：Concept | Behavior | Code Output | Spot the Bug | Misconception
- `difficulty`：1 易 / 2 中 / 3 难
- 多选题：`"answer": ["A","C"]`，5 个选项 A–E

### 加题流程

**桌面端（零编译）**：写 `bank/chNN.json` → `cd app && cargo run -p quiz-bank` 校验 → 把 `bank/` 放到可执行文件旁 → 重启应用即生效。

**同步内嵌快照**（影响 APK、Web 版和无 bank 目录时的回退，改完题库后执行并重新编译）：

```bash
cd app && cargo run -p quiz-bank -- --sync    # 校验通过后重新生成 app/assets/bank.json
```

> 校验器与应用共用同一套 serde 数据类型（`app/quiz-bank`）——校验通过即保证应用能加载。

欢迎 PR 补充题目——只需改 `bank/` 下的 JSON 并确保 `cargo run -p quiz-bank`（或 `cargo test -p quiz-bank`）通过。

## 开发工作流

```bash
cd app && trunk serve            # 热重载开发（http://127.0.0.1:8080，走内嵌题库回退路径）
cargo check --target wasm32-unknown-unknown        # 前端类型检查（app/ 下执行）
cd src-tauri && cargo check                        # 壳类型检查
cargo run -p quiz-bank                             # 题库校验（app/ 下执行；--sync 同步内嵌快照）
cargo test -p quiz-bank                            # 同一校验的测试形式（CI 门禁用）
```

> 前置：Rust stable 工具链 + `wasm32-unknown-unknown` 目标 + trunk，安装步骤见[手工编译指南 · 公共准备](#公共准备所有平台都要做一次)。日常开发只需要这三样，**不需要 Node.js**。

### 数据存储

进度存于各端 WebView 的 localStorage（互相独立，不同步）：
`rustQuizRs.scores`（章节最高/最近分）、`rustQuizRs.wrong`（错题本）、`rustQuizRs.theme`（深浅主题）。

## 技术架构

```
┌─────────────────────────────────────────────┐
│ 前端：Leptos 0.6 (Rust → WASM)               │
│  - 全部业务逻辑：判分/多选/抽题/错题本        │
│  - 纯 Rust 语法高亮器（无 JS 依赖）           │
│  - 内嵌字体 Inter + JetBrains Mono（多端一致）│
├─────────────────────────────────────────────┤
│ 壳：Tauri 2                                  │
│  - Windows: WebView2   Linux: WebKitGTK      │
│  - Android: 系统 WebView   Web: 无壳直接跑    │
│  - 命令: load_bank_files / open_url / 窗控    │
├─────────────────────────────────────────────┤
│ 题库：bank/chNN.json（每章一个文件）          │
│  - 桌面: 运行时读可执行文件旁的 bank/ 目录    │
│  - 移动/Web/回退: 编译期内嵌 assets/bank.json │
└─────────────────────────────────────────────┘
```

## 目录结构

```
bank/chNN.json            题库源数据（唯一数据源）
.github/workflows/        CI：release.yml（四端产物）+ deploy-pages.yml（在线版）
app/                      源码
  quiz-bank/              题库 schema 类型 + 校验器（cargo run -p quiz-bank）
  index.html              trunk 构建入口（声明字体/图标资源拷贝）
  style.css               暗色科技风主题 + 浅色主题 + 移动端媒体查询
  fonts/  ferris.png      内嵌资源（OFL 字体 / 自绘 Ferris）
  assets/bank.json        内嵌题库快照（生成物，勿手改）
  src/main.rs             应用壳：路由/标题栏/主题/抽屉
  src/model.rs            题库加载、语法高亮（schema 类型复用 quiz-bank）
  src/storage.rs          localStorage 持久化（分数/错题/主题）
  src/views.rs            Sidebar/Home/Exam/Wrong/Quiz 组件
  src-tauri/              Tauri 壳
    src/lib.rs            入口 + 全部命令（移动端入口在此）
    src/main.rs           桌面入口（调用 lib::run）
    tauri.conf.json       窗口配置 + 打包配置（deb/AppImage）
    capabilities/         窗口拖拽权限
    icons/                icon.ico（Windows）/ icon.png（移动 + Linux 打包）
    gen/android/          Android 工程（已提交，无需重新 init）
```

## 自动构建与发布（GitHub Actions）

日常开发**不需要**在本地凑齐四个平台的环境——CI 全部代劳：

| 工作流 | 触发 | 产出 |
|---|---|---|
| [release.yml](.github/workflows/release.yml) | 推送 `v*` 标签 | exe / deb / AppImage / Linux 裸二进制 / apk / web zip，自动挂到 Releases |
| [deploy-pages.yml](.github/workflows/deploy-pages.yml) | 推送 main（app/ 有改动） | 在线版自动更新到 GitHub Pages |
| [validate.yml](.github/workflows/validate.yml) | push / PR 涉及 bank/ 或校验器 | 校验题库 + 确认内嵌快照已同步（忘跑 `--sync` 会失败） |

发一个版本只需要：

```bash
git tag v0.1.0
git push origin v0.1.0
# ✅ 等 CI 跑完（约 20 分钟），Releases 页面出现全部产物
```

一次性配置（仓库建好后做一次）：**Settings → Pages → Source 选 "GitHub Actions"**，在线版即生效。

---

# 手工编译指南

> 每个平台小节都是**自包含**的：按顺序执行完"公共准备"+ 对应平台小节即得产物。
> 每一步标注了 ✅ 预期结果，实际输出对不上时先查[疑难排查](#疑难排查)。

## 公共准备（所有平台都要做一次）

**第 1 步**：安装 Rust 工具链（stable）。访问 <https://rustup.rs/> 按提示安装，然后验证：

```bash
rustc --version        # ✅ 输出形如 rustc 1.8x.0
cargo --version        # ✅ 输出形如 cargo 1.8x.0
```

**第 2 步**：添加 WASM 编译目标（前端编译成 WebAssembly 用）：

```bash
rustup target add wasm32-unknown-unknown
rustup target list --installed | grep wasm
# ✅ 输出 wasm32-unknown-unknown
```

**第 3 步**：安装前端打包器 trunk：

```bash
cargo install trunk --locked      # 首次编译约 10 分钟
trunk --version                   # ✅ 输出形如 trunk 0.2x.x
```

**第 4 步**：克隆仓库并构建前端（**所有平台产物都依赖这一步**）：

```bash
git clone https://github.com/MilesYang-debug/rust-book-quiz.git
cd rust-book-quiz/app
trunk build --release
ls dist/
# ✅ dist/ 下出现 index.html、*.wasm、*.js、fonts/ 等文件（约 4MB）
```

> 本项目不依赖 Node.js——题库校验与内嵌快照生成也是 Rust（`cargo run -p quiz-bank`）。

## 1. Windows 桌面版

前置：完成[公共准备](#公共准备所有平台都要做一次)。

```bash
cd app/src-tauri
cargo build --release
# ✅ 产物: target/release/rust-book-quiz-desktop.exe
```

把 exe 拷出来改名 `RustBookQuiz.exe` 即可分发——它完全自包含（前端、字体、内嵌题库全在里面），拷走即用。旁边放 `bank/` 目录可覆盖内嵌题库（见[加题流程](#加题流程)）。

## 2. Linux 桌面版（deb / AppImage / 裸二进制）

前置：完成[公共准备](#公共准备所有平台都要做一次)，且以下操作**在 Linux 上执行**。

**第 1 步**：安装系统依赖（Ubuntu/Debian 为例）：

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libgtk-3-dev librsvg2-dev libxdo-dev
```

**第 2 步**：安装 Tauri CLI（二选一）：

```bash
npm install -g @tauri-apps/cli@^2     # 快，预编译二进制（后续命令写 tauri ...）
# 或
cargo install tauri-cli --locked      # 慢（约 10 分钟），无需 Node（后续命令写 cargo tauri ...）
```

**第 3 步**：构建并打包：

```bash
cd app
cargo tauri build --bundles deb appimage    # npm 版则是: tauri build --bundles deb appimage
```

✅ 产物（在 `app/src-tauri/target/release/` 下）：

| 路径 | 说明 |
|---|---|
| `bundle/deb/*.deb` | Debian/Ubuntu 安装包，`sudo apt install ./xxx.deb` |
| `bundle/appimage/*.AppImage` | 免安装通用包，`chmod +x` 后运行 |
| `rust-book-quiz-desktop` | 裸二进制，需系统有 `libwebkit2gtk-4.1` |

> ⚠️ 只跑 `cargo build --release` **只能得到裸二进制，打不出 deb**——deb/AppImage 必须走
> `cargo tauri build`（打包器已在 `tauri.conf.json` 的 `bundle` 段启用）。

## 3. Android 版

前置：完成[公共准备](#公共准备所有平台都要做一次)。环境搭建步骤多，但每步都是一次性的。

### 3.1 一次性环境搭建

**第 1 步**：安装 JDK 17（[Temurin](https://adoptium.net/) 等发行版均可），验证：

```bash
java -version    # ✅ 输出含 "17."
```

**第 2 步**：安装 Android SDK 命令行工具。从
<https://developer.android.com/studio#command-line-tools-only> 下载 commandline-tools，
解压成如下布局（**目录名必须是 `latest`**）：

```
<sdk>/cmdline-tools/latest/bin/sdkmanager
```

**第 3 步**：接受许可并安装组件（约 3GB 下载）：

```bash
cd <sdk>
yes | cmdline-tools/latest/bin/sdkmanager --licenses
cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-36" \
  "build-tools;36.0.0" "ndk;27.1.12297006"
```

> Windows 下建议用 Git Bash 执行（`sdkmanager` 换成 `sdkmanager.bat`）。公司代理环境
> Gradle 连不上网的话，见[疑难排查](#疑难排查)最后两条。

**第 4 步**：Rust Android 目标 + Tauri CLI：

```bash
rustup target add aarch64-linux-android
cargo install tauri-cli --locked        # 或 npm install -g @tauri-apps/cli@^2
```

**第 5 步**：环境变量（每次构建前 export，或写入系统变量）：

```bash
export JAVA_HOME=<jdk17 安装目录>
export ANDROID_HOME=<sdk 目录>
export NDK_HOME=$ANDROID_HOME/ndk/27.1.12297006
```

### 3.2 构建 APK

```bash
cd app && trunk build --release     # 前端产物（会嵌入 APK）
cd src-tauri
cargo tauri android build --apk --target aarch64
# ✅ 产物: gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk
```

> - 本仓库已提交 `gen/android/` 工程，**不需要**执行 `cargo tauri android init`
> - Linux/macOS 下若报 gradlew Permission denied：`chmod +x gen/android/gradlew`
> - ⚠️ 不要用 `--debug`：debug 版含未剥离符号约 238MB，release 版仅约 12MB

### 3.3 签名（release APK 必须签名才能安装）

```bash
# 用 debug keystore（gradle 首次构建自动生成于 ~/.android/debug.keystore），个人自用足够
$ANDROID_HOME/build-tools/36.0.0/apksigner sign \
  --ks ~/.android/debug.keystore --ks-pass pass:android \
  --key-pass pass:android --ks-key-alias androiddebugkey \
  --out RustBookQuiz.apk app-arm64-release-unsigned.apk

$ANDROID_HOME/build-tools/36.0.0/apksigner verify RustBookQuiz.apk   # ✅ 无报错即签名有效
```

> 上架应用商店需 `keytool -genkeypair` 生成正式密钥另签。

APK 传到手机直接安装（允许"未知来源"）。手机端用内嵌题库；UI 自动切换移动布局（☰ 抽屉侧边栏、系统状态栏、单列触控）。

### 3.4 改壳代码时的移动端要点

- 入口在 `lib.rs`：`#[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run()`；`main.rs` 只是桌面薄壳
- 窗口 API（minimize/maximize 等）必须 `#[cfg(desktop)]` 门控——Android 目标上这些方法不存在，否则 E0599
- 外链统一走 `tauri-plugin-opener`（桌面 + Android 通用），命令实现在 `lib.rs` 的 `open_url`
- 移动端必须有 `icons/icon.png`（桌面用 .ico，移动用 .png）

## 4. Web 版

前端本身就是纯静态 WASM 应用，[公共准备第 4 步](#公共准备所有平台都要做一次)产出的 `app/dist/` 就是全部部署物——**无需后端、无需数据库**，任何能托管静态文件的地方都行。

本地预览：

```bash
python -m http.server 8000 --directory app/dist
# ✅ 浏览器访问 http://localhost:8000 可正常刷题
```

部署到静态服务器（nginx 示例）：

```nginx
server {
    listen 80;
    root /var/www/rust-book-quiz;    # dist/ 的内容
    index index.html;
    types { application/wasm wasm; } # 老版本 nginx 需手动加 wasm MIME
    gzip on;                         # WASM 体积较大，开压缩收益明显
    gzip_types application/wasm application/javascript text/css;
}
```

部署到**子路径**（如 GitHub Pages 的 `https://MilesYang-debug.github.io/rust-book-quiz/`）需在构建时指定资源前缀：

```bash
trunk build --release --public-url /rust-book-quiz/
```

Web 版行为差异：

- 题库固定为编译期内嵌快照（`app/assets/bank.json`）——改题后需重新构建部署，没有桌面端的 `bank/` 热加载
- 标题栏自动隐藏窗口控制按钮（最小化/关闭交给浏览器），主题切换保留
- 进度存在访问者各自浏览器的 localStorage 里，服务器不存任何用户数据
- 移动浏览器访问自动切换移动布局（与 APK 同一套响应式 CSS）

---

# 疑难排查

| 症状 | 原因与解法 |
|---|---|
| `can't find crate for core`（wasm 目标） | 没装 WASM 目标：`rustup target add wasm32-unknown-unknown` |
| `error: linking with cc failed` / 找不到 `webkit2gtk` | Linux 系统依赖没装齐，重跑 [2. Linux 第 1 步](#2-linux-桌面版deb--appimage--裸二进制)的 `apt install` |
| `icon.ico not found` | `app/src-tauri/icons/icon.ico` 缺失（tauri-build 生成 Windows 资源必需），从仓库恢复 |
| `cargo tauri build` 没产出 deb | 确认命令是 `cargo tauri build` 而不是 `cargo build`，且 `tauri.conf.json` 里 `bundle.active: true` |
| AppImage 打包卡在下载工具 | Tauri 首次打包会下载 linuxdeploy，网络不通时挂代理重试 |
| Android 构建 `gradlew: Permission denied` | gradlew 丢失可执行位（从 Windows 提交的常见问题）：`chmod +x app/src-tauri/gen/android/gradlew`；在 git 里根治：`git update-index --chmod=+x app/src-tauri/gen/android/gradlew` |
| Android 构建报 E0599 找不到窗口方法 | 桌面窗口 API 没加 `#[cfg(desktop)]` 门控，见 [3.4 移动端要点](#34-改壳代码时的移动端要点) |
| Web 版部署后一片空白 | 子路径部署没加 `--public-url /<子路径>/`；或服务器 .wasm 的 MIME 不对（需 `application/wasm`） |
| Web 版图标/字体 404 | 资源必须用相对路径引用（`ferris.png`、`fonts/...`），绝对路径 `/xxx` 在子路径部署下会指向域名根 |
| Gradle 连不上网（公司代理环境） | JVM 不读 `HTTPS_PROXY` 环境变量。写 `~/.gradle/gradle.properties`：四行 `systemProp.http(s).proxyHost/proxyPort=<代理主机/端口>`，另加 `systemProp.http(s).nonProxyHosts=localhost\|127.0.0.1\|10.*\|172.16.*\|192.168.*` |
| Gradle wrapper 下载发行版失败（代理环境） | wrapper 下载在读到代理配置前执行：手动 `curl -L -o <本地目录>/gradle-8.14.3-bin.zip https://services.gradle.org/distributions/gradle-8.14.3-bin.zip`，把 `gen/android/gradle/wrapper/gradle-wrapper.properties` 的 `distributionUrl` 临时改成 `file:///<本地路径>`（**勿提交此改动**，可用 `git update-index --skip-worktree` 让 git 忽略） |

# 已知限制

- APK 为 debug 密钥签名，不可直接上架应用商店
- 各端进度不互通（无云同步）
- 未覆盖 ch21（Web 服务器实战——纯项目走读章节，不适合出题）

# 资源许可

- 字体 Inter、JetBrains Mono：SIL OFL 1.1，随应用分发合规
- Ferris 图标为本项目程序化绘制（官方 Ferris 本身 CC0）
- 题目基于[官方教程](https://doc.rust-lang.org/book/)原创编写
