# Rust Book Quiz 🦀

《The Rust Programming Language》（官方 Rust Book）配套刷题应用。**一套 Rust 代码，四端交付**：Windows / Linux 桌面 + Android + Web 浏览器，无 JS 框架、无后端、单文件即用。

- **394 道题**覆盖全书 ch1–ch20（英文题干，贴合官方文档原文）
- 三种模式：**章节练习**（逐题即时判分 + 解析）、**随机模拟考**（整卷交卷统一判分 + 回看）、**错题本**（答错自动收录，答对移出）
- 题型：单选 / 多选 / 代码输出预测 / 找错题，附难度与章节小节标签
- 桌面端**加题免编译**：往 exe 旁的 `bank/` 目录放 JSON，重启即生效

## 下载 / 在线使用

从 [Releases](../../releases) 页面获取预编译产物：

| 产物 | 说明 |
|---|---|
| `RustBookQuiz.exe` | Windows 桌面版（约 6MB，双击即用，依赖系统自带 WebView2） |
| `RustBookQuiz-linux` | Linux 桌面版（x86_64，需系统安装 `libwebkit2gtk-4.1`，`chmod +x` 后运行） |
| `RustBookQuiz.apk` | Android 版（约 12MB，arm64，debug 密钥签名，含内嵌题库） |
| Web 版 | 无需下载，[在线访问](https://<你的用户名>.github.io/rust-book-quiz/)；也可自行部署，见 [4. Web 页面版](#4-web-页面版部署到服务器浏览器直接访问) |

也可以按下方[构建指南](#构建指南)从源码自行编译。

## 技术架构

```
┌─────────────────────────────────────────────┐
│ 前端：Leptos 0.6 (Rust → WASM)               │
│  - 全部业务逻辑：判分/多选/抽题/错题本        │
│  - 纯 Rust 语法高亮器（无 JS 依赖）           │
│  - 内嵌字体 Inter + JetBrains Mono（三端一致）│
├─────────────────────────────────────────────┤
│ 壳：Tauri 2                                  │
│  - Windows: WebView2   Linux: WebKitGTK      │
│  - Android: 系统 WebView                     │
│  - 命令: load_bank_files / open_url / 窗控    │
├─────────────────────────────────────────────┤
│ 题库：bank/chNN.json（每章一个文件）          │
│  - 桌面: 运行时读 exe 旁 bank/ 目录           │
│  - 移动/回退: 编译期内嵌 app/assets/bank.json │
└─────────────────────────────────────────────┘
```

## 目录结构

```
bank/chNN.json            题库源数据（唯一数据源）
validate.js               题库校验器：node validate.js
app/                      源码
  index.html              trunk 构建入口（声明字体/图标资源拷贝）
  style.css               暗色科技风主题 + 浅色主题 + 移动端媒体查询
  fonts/  ferris.png      内嵌资源（OFL 字体 / 自绘 Ferris）
  assets/bank.json        内嵌题库快照（生成物，勿手改）
  src/main.rs             应用壳：路由/标题栏/主题/抽屉
  src/model.rs            数据模型、题库加载、语法高亮
  src/storage.rs          localStorage 持久化（分数/错题/主题）
  src/views.rs            Sidebar/Home/Exam/Wrong/Quiz 组件
  src-tauri/              Tauri 壳
    src/lib.rs            入口 + 全部命令（移动端入口在此）
    src/main.rs           桌面入口（调用 lib::run）
    tauri.conf.json       无边框窗口配置、frontendDist
    capabilities/         窗口拖拽权限
    icons/icon.ico|png    桌面(.ico) / 移动(.png) 图标
    gen/android/          tauri android init 生成的 Android 工程
```

---

# 构建指南

## 0. 公共前置（所有平台）

需要 [Rust 工具链](https://rustup.rs/)（stable）。然后：

```bash
rustup target add wasm32-unknown-unknown   # WASM 前端目标
cargo install trunk --locked               # 前端打包器（首次编译约 10 分钟）
# Node.js 仅用于 validate.js / 题库快照生成，非运行时依赖
```

前端构建（**所有平台的第一步**）：

```bash
cd app
trunk build --release        # 产物 → app/dist/（会被嵌入各端二进制）
```

开发调试用 `trunk serve`（热重载，浏览器访问 http://127.0.0.1:8080，走内嵌题库回退路径）。

## 1. Windows 桌面

```bash
cd app/src-tauri
cargo build --release
# 产物: target/release/rust-book-quiz-desktop.exe，拷出来改名即可
```

要点：

- `icons/icon.ico` 必须存在（tauri-build 生成 Windows 资源用），缺失会报 `icon.ico not found`
- exe 完全自包含（dist、字体、内嵌题库全在里面），拷走即用；`bank/` 放旁边可覆盖内嵌题库

## 2. Linux 桌面

```bash
# 一次性系统依赖（Ubuntu/Debian）
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libgtk-3-dev librsvg2-dev

cd app && trunk build --release
cd src-tauri && cargo build --release
# 产物: target/release/rust-book-quiz-desktop（bank/ 放旁边，用法同 Windows）
```

## 3. Android

### 3.1 一次性环境搭建

```bash
# ① JDK 17（Temurin 等发行版均可）

# ② Android SDK：下载 commandline-tools 解压为如下布局（目录名必须是 latest）
#    https://developer.android.com/studio#command-line-tools-only
#    <sdk>/cmdline-tools/latest/bin/sdkmanager

# ③ 接受许可并安装组件（约 3GB 下载）
cd <sdk>
yes | cmdline-tools/latest/bin/sdkmanager --licenses
cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;27.1.12297006"

# ④ Rust Android 目标 + tauri-cli
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
cargo install tauri-cli --locked

# ⑤ 环境变量（每次构建前 export，或写入系统变量）
export JAVA_HOME=<jdk17 安装目录>
export ANDROID_HOME=<sdk 目录>
export NDK_HOME=$ANDROID_HOME/ndk/27.1.12297006
```

> Windows 下建议用 Git Bash 执行以上命令（`sdkmanager` 换成 `sdkmanager.bat`）。

### 3.2 代理环境专项（公司内网等场景，直连网络可跳过）

**根因**：curl 读 `HTTPS_PROXY` 环境变量所以下载正常，但 **JVM/Gradle 不读环境变量**，导致 Gradle 拉取发行版/插件时连不上外网。

**① JVM 代理** — 写 `~/.gradle/gradle.properties`（全局，一次性）：

```properties
systemProp.http.proxyHost=<代理主机>
systemProp.http.proxyPort=<端口>
systemProp.https.proxyHost=<代理主机>
systemProp.https.proxyPort=<端口>
systemProp.http.nonProxyHosts=localhost|127.0.0.1|10.*|172.16.*|192.168.*
systemProp.https.nonProxyHosts=localhost|127.0.0.1|10.*|172.16.*|192.168.*
```

**② Gradle 发行版本地化**（wrapper 下载在配代理前就会执行，需绕过）：

```bash
curl -L -o <本地目录>/gradle-8.14.3-bin.zip https://services.gradle.org/distributions/gradle-8.14.3-bin.zip
# 改 app/src-tauri/gen/android/gradle/wrapper/gradle-wrapper.properties：
# distributionUrl=file:///<本地目录>/gradle-8.14.3-bin.zip
```

> 注意：`cargo tauri android init` 重新生成工程会还原 wrapper 配置，重跑 init 后需重改。

### 3.3 初始化与构建

```bash
cd app && trunk build --release            # 前端产物（会嵌入 APK）
cd src-tauri
cargo tauri android init                   # 生成 gen/android（仅首次/结构变更时）

# 构建 release APK（⚠️ 不要用 --debug：debug 版含未剥离符号，238MB；release 仅 12MB）
cargo tauri android build --apk --target aarch64
# 产物: gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
```

### 3.4 签名（release 版必须签名才能安装）

```bash
# 用 debug keystore（gradle 首次构建自动生成于 ~/.android/debug.keystore），个人自用足够
$ANDROID_HOME/build-tools/34.0.0/apksigner sign \
  --ks ~/.android/debug.keystore --ks-pass pass:android \
  --key-pass pass:android --ks-key-alias androiddebugkey \
  --out RustBookQuiz.apk app-universal-release-unsigned.apk

$ANDROID_HOME/build-tools/34.0.0/apksigner verify RustBookQuiz.apk
```

> 上架应用商店需 `keytool -genkeypair` 生成正式密钥另签。

### 3.5 安装

APK 传到手机直接安装（允许"未知来源"）。手机端用内嵌题库；UI 自动切换为移动布局（☰ 抽屉侧边栏、系统状态栏、单列触控）。

### 3.6 移动端代码要点（改壳代码时注意）

- 入口在 `lib.rs`：`#[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run()`；`main.rs` 只是桌面薄壳
- 窗口 API（minimize/maximize 等）必须 `#[cfg(desktop)]` 门控——Android 目标上这些方法不存在，否则 E0599
- `open` crate 不支持 Android：Cargo.toml 中限定 `[target.'cfg(not(target_os = "android"))'.dependencies]`
- 移动端必须有 `icons/icon.png`（桌面用 .ico，移动用 .png）

## 4. Web 页面版（部署到服务器，浏览器直接访问）

前端本身就是纯静态的 WASM 应用，`trunk build` 的产物即可直接部署——**无需后端、无需数据库**，任何能托管静态文件的地方都行。

```bash
cd app
trunk build --release        # 产物 → app/dist/（index.html + wasm + js + 字体，约 4MB）
```

把 `app/dist/` 整个目录扔到静态服务器即可。本地快速预览：

```bash
python -m http.server 8000 --directory dist    # 访问 http://localhost:8000
```

**nginx 示例**：

```nginx
server {
    listen 80;
    root /var/www/rust-book-quiz;    # dist/ 的内容
    index index.html;
    # 确保 .wasm 以正确 MIME 返回（主流发行版的 mime.types 已包含，老版本需手动加）
    types { application/wasm wasm; }
    # WASM 体积较大，开压缩收益明显
    gzip on;
    gzip_types application/wasm application/javascript text/css;
}
```

**部署到子路径**（如 GitHub Pages 的 `https://<user>.github.io/rust-book-quiz/`）需在构建时指定资源前缀：

```bash
trunk build --release --public-url /rust-book-quiz/
```

Web 版行为差异：

- 题库固定为编译期内嵌快照（`app/assets/bank.json`）——改题后需重新构建部署，没有桌面端的 `bank/` 目录热加载
- 标题栏自动隐藏窗口控制按钮（最小化/关闭交给浏览器），主题切换保留
- 进度存在访问者各自浏览器的 localStorage 里，服务器不存任何用户数据
- 移动浏览器访问自动切换移动布局（与 APK 同一套响应式 CSS）

---

# 题库：自己加题 / 贡献题目

## 题库规范（bank/chNN.json）

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

## 加题流程

**桌面端（零编译）**：写 `bank/chNN.json` → `node validate.js` 校验 → 把 `bank/` 放到 exe 旁 → 重启应用即生效。

**同步内嵌快照**（影响 APK 和无 bank 目录时的回退，改完题库后按需执行并重新编译）：

```bash
node -e "const fs=require('fs');const b=fs.readdirSync('bank').filter(f=>/^ch\d+\.json$/.test(f)).sort().map(f=>JSON.parse(fs.readFileSync('bank/'+f,'utf8'))).sort((x,y)=>x.chapter-y.chapter);fs.writeFileSync('app/assets/bank.json',JSON.stringify(b,null,1));console.log('ok')"
```

欢迎 PR 补充题目——只需改 `bank/` 下的 JSON 并确保 `node validate.js` 通过。

---

## 开发工作流

```bash
cd app && trunk serve            # 热重载开发（http://127.0.0.1:8080）
cargo check --target wasm32-unknown-unknown        # 前端类型检查（app/ 下）
cd src-tauri && cargo check                        # 壳类型检查
node validate.js                                   # 题库结构校验（项目根目录）
```

## 数据存储

进度存于各端 WebView 的 localStorage（互相独立，不同步）：
`rustQuizRs.scores`（章节最高/最近分）、`rustQuizRs.wrong`（错题本）、`rustQuizRs.theme`（深浅主题）。

## 已知限制

- Android 端 "read ↗" 外链无响应（需接 tauri-plugin-opener，未做）
- APK 为 debug 密钥签名，不可直接上架
- 各端进度不互通（无云同步）
- 未覆盖 ch21（Web 服务器实战——纯项目走读章节，不适合出题）

## 资源许可

- 字体 Inter、JetBrains Mono：SIL OFL 1.1，随应用分发合规
- Ferris 图标为本项目程序化绘制（官方 Ferris 本身 CC0）
- 题目基于[官方教程](https://doc.rust-lang.org/book/)原创编写
