# PDF 压缩轻量小程序 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个跨平台（Windows/macOS）的轻量 PDF 压缩桌面应用，本地离线、可调档位、批量处理、默认另存新文件。

**Architecture:** Tauri 2 桌面框架（Rust 后端 + Vue 3 前端），后端通过"可插拔压缩引擎"接口抽象压缩逻辑，PDF 引擎运行时调用系统安装的 Ghostscript（`gs` 命令）。前端按文件类型自适应渲染档位 UI。

**Tech Stack:** Tauri 2、Rust、Vue 3、TypeScript、Vite、Ghostscript（外部依赖，运行时检测）。

## Global Constraints

- 版本下限：Node >= 22、Rust >= 1.77（Tauri 2 要求）、Ghostscript >= 10.0。
- Ghostscript **不随应用分发**，运行时检测系统 `gs`，缺失则引导安装（规避 AGPL 合规风险）。
- 压缩档位三档：`/prepress`（无损/轻度）、`/ebook`（平衡，默认）、`/screen`（极致压缩）。
- 文字层默认保留；仅"极致压缩"档位允许勾选"栅格化去除文字层"。
- 输出默认另存新文件：`原名_compressed.pdf`，保留原文件。
- 批量处理：单个文件失败不中断队列，标记失败并继续。
- 所有 UI 文案使用简体中文。

---

## File Structure

```
compress/
├── package.json                 # 前端依赖与脚本
├── vite.config.ts               # Vite 配置
├── index.html                   # 前端入口
├── tsconfig.json                # TS 配置
├── src/                         # 前端源码
│   ├── main.ts                  # Vue 入口
│   ├── App.vue                  # 根组件
│   ├── types.ts                 # 前端类型定义（与后端 IPC 契约一致）
│   ├── api.ts                   # Tauri invoke 封装
│   └── components/
│       ├── FileList.vue         # 文件列表 + 进度 + 结果详情
│       ├── PresetPicker.vue     # 压缩档位选择
│       └── OutputDirPicker.vue  # 输出目录选择
├── src-tauri/                   # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json          # Tauri 配置
│   ├── build.rs
│   ├── icons/                   # 应用图标
│   └── src/
│       ├── main.rs              # 入口，注册命令
│       ├── lib.rs               # 模块导出 + run()
│       ├── error.rs             # 统一错误类型
│       ├── detect.rs            # 外部依赖检测（gs）
│       ├── engine/
│       │   ├── mod.rs           # CompressionEngine trait + 注册表
│       │   └── pdf.rs           # Ghostscript PDF 引擎
│       ├── task.rs              # 压缩队列调度 + 进度事件
│       └── stats.rs             # 结果统计（大小/压缩率/页数）
└── docs/superpowers/
    ├── specs/2026-08-14-pdf-compressor-design.md
    └── plans/2026-08-14-pdf-compressor.md
```

---

## Task 1: 项目脚手架（Tauri 2 + Vue 3）

**Files:**
- Create: `package.json`, `vite.config.ts`, `index.html`, `tsconfig.json`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: 无（首个任务）。
- Produces: 可运行的空 Tauri 应用骨架，`npm run tauri dev` 能弹出窗口。

- [ ] **Step 1: 安装 Rust 工具链**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc -V   # 期望 >= 1.77
```

- [ ] **Step 2: 创建前端工程文件**

创建 `package.json`：

```json
{
  "name": "pdf-compressor",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.5.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@tauri-apps/api": "^2.0.0",
    "@vitejs/plugin-vue": "^5.2.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vue-tsc": "^2.1.0"
  }
}
```

创建 `vite.config.ts`：

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
```

创建 `index.html`：

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PDF 压缩器</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

创建 `tsconfig.json`：

```json
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "resolveJsonModule": true,
    "esModuleInterop": true,
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

- [ ] **Step 3: 创建 Rust 后端文件**

创建 `src-tauri/Cargo.toml`：

```toml
[package]
name = "pdf-compressor"
version = "0.1.0"
description = "轻量 PDF 压缩工具"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

创建 `src-tauri/build.rs`：

```rust
fn main() {
    tauri_build::build()
}
```

创建 `src-tauri/src/main.rs`：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pdf_compressor_lib::run()
}
```

创建 `src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> 注：lib crate 名需与 `main.rs` 一致。将 `src-tauri/Cargo.toml` 的 `[package]` 改为 `[lib]` 结构，见下。

创建 `src-tauri/Cargo.toml` 的 lib 段（追加到文件末尾）：

```toml
[lib]
name = "pdf_compressor_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

创建 `src-tauri/tauri.conf.json`：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "PDF压缩器",
  "version": "0.1.0",
  "identifier": "com.example.pdf-compressor",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "PDF 压缩器",
        "width": 900,
        "height": 640
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/icon.png"]
  }
}
```

- [ ] **Step 4: 安装前端依赖并验证启动**

```bash
npm install
```

创建 `src/main.ts`：

```ts
import { createApp } from "vue";
import App from "./App.vue";

createApp(App).mount("#app");
```

创建 `src/App.vue`：

```vue
<template>
  <div class="container">
    <h1>PDF 压缩器</h1>
  </div>
</template>

<style>
body { font-family: system-ui, sans-serif; margin: 0; }
.container { padding: 24px; }
</style>
```

- [ ] **Step 5: 生成应用图标并验证 `tauri dev`**

```bash
mkdir -p src-tauri/icons
# 放置一个 512x512 的 PNG 图标到 src-tauri/icons/icon.png
npm run tauri dev
```

期望：弹出标题为「PDF 压缩器」的窗口，显示标题文字。

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + Vue 3 project"
```

---

## Task 2: 统一错误类型与压缩领域模型

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/stats.rs`

**Interfaces:**
- Consumes: 无（独立模块）。
- Produces:
  - `AppError`（实现 `serde::Serialize`，供 IPC 序列化错误）。
  - `CompressionResult { input_path, output_path, original_size, compressed_size, page_count, preset, text_layer_preserved, duration_ms }`。
  - `stats::analyze(path) -> Result<PdfInfo>`，`PdfInfo { page_count, file_size }`。

- [ ] **Step 1: 编写错误类型**

创建 `src-tauri/src/error.rs`：

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未检测到 Ghostscript，请先安装: {0}")]
    GhostscriptNotFound(String),
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    #[error("压缩失败: {0}")]
    CompressFailed(String),
    #[error("读取 PDF 信息失败: {0}")]
    ParseFailed(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct AppErrorDto {
    pub message: String,
}

impl From<AppError> for AppErrorDto {
    fn from(e: AppError) -> Self {
        Self { message: e.to_string() }
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

> 注：`thiserror` 需加入 `Cargo.toml` 依赖（见 Step 2）。

- [ ] **Step 2: 添加依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 追加：

```toml
thiserror = "1"
```

- [ ] **Step 3: 编写结果统计模块**

创建 `src-tauri/src/stats.rs`：

```rust
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PdfInfo {
    pub page_count: u32,
    pub file_size: u64,
}

/// 通过简单扫描 `/Type /Page` 对象统计页数，文件大小直接取元数据。
pub fn analyze(path: &std::path::Path) -> AppResult<PdfInfo> {
    let meta = std::fs::metadata(path).map_err(AppError::Io)?;
    let bytes = std::fs::read(path).map_err(AppError::Io)?;
    let text = String::from_utf8_lossy(&bytes);
    let page_count = text.matches("/Type /Page").count() as u32;
    Ok(PdfInfo {
        page_count,
        file_size: meta.len(),
    })
}
```

- [ ] **Step 4: 编写单元测试**

创建 `src-tauri/src/stats.rs` 末尾追加测试（或新建 `tests/stats_test.rs`）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_pages_in_minimal_pdf() {
        let dir = std::env::temp_dir().join("pdfc_test_stats");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("sample.pdf");
        let body = b"%PDF-1.4\n1 0 obj\n<< /Type /Page >>\nendobj\n";
        std::fs::write(&p, body).unwrap();
        let info = analyze(&p).unwrap();
        assert_eq!(info.page_count, 1);
        assert!(info.file_size > 0);
    }
}
```

- [ ] **Step 5: 运行测试**

```bash
cd src-tauri && cargo test stats
```

期望：PASS。

- [ ] **Step 6: 在 `lib.rs` 中声明模块**

修改 `src-tauri/src/lib.rs`：

```rust
mod error;
mod stats;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat: add error type and pdf stats module"
```

---

## Task 3: 外部依赖检测（Ghostscript）

**Files:**
- Create: `src-tauri/src/detect.rs`

**Interfaces:**
- Consumes: `crate::error::{AppError, AppResult}`。
- Produces:
  - `detect::detect_ghostscript() -> AppResult<GhostscriptInfo>`，`GhostscriptInfo { found: bool, path: Option<String>, version: Option<String>, install_hint: String }`。

- [ ] **Step 1: 编写检测逻辑**

创建 `src-tauri/src/detect.rs`：

```rust
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct GhostscriptInfo {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub install_hint: String,
}

#[cfg(target_os = "windows")]
fn candidate_names() -> Vec<&'static str> {
    vec!["gswin64c.exe", "gswin32c.exe", "gs.exe"]
}

#[cfg(not(target_os = "windows"))]
fn candidate_names() -> Vec<&'static str> {
    vec!["gs"]
}

fn install_hint() -> String {
    if cfg!(target_os = "windows") {
        "请从 https://ghostscript.com/releases/gsdnld.html 下载 Windows 版 Ghostscript 并安装。".to_string()
    } else {
        "请通过 Homebrew 安装: brew install ghostscript，或从 https://ghostscript.com/releases/ 下载 macOS 版。".to_string()
    }
}

pub fn detect_ghostscript() -> AppResult<GhostscriptInfo> {
    // 1. 尝试 PATH
    for name in candidate_names() {
        if let Ok(out) = Command::new(name).arg("--version").output() {
            if out.status.success() {
                return Ok(GhostscriptInfo {
                    found: true,
                    path: Some(name.to_string()),
                    version: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
                    install_hint: String::new(),
                });
            }
        }
    }
    // 2. 尝试常见安装目录
    let mut extra_dirs: Vec<PathBuf> = vec![];
    #[cfg(target_os = "windows")]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            extra_dirs.push(PathBuf::from(pf).join("gs"));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for d in ["/usr/local/bin", "/opt/homebrew/bin", "/opt/local/bin"] {
            extra_dirs.push(PathBuf::from(d));
        }
    }
    for dir in extra_dirs {
        for name in candidate_names() {
            let full = dir.join(name);
            if full.exists() {
                if let Ok(out) = Command::new(&full).arg("--version").output() {
                    if out.status.success() {
                        return Ok(GhostscriptInfo {
                            found: true,
                            path: Some(full.to_string_lossy().to_string()),
                            version: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
                            install_hint: String::new(),
                        });
                    }
                }
            }
        }
    }
    Ok(GhostscriptInfo {
        found: false,
        path: None,
        version: None,
        install_hint: install_hint(),
    })
}
```

- [ ] **Step 2: 在 `lib.rs` 声明模块**

修改 `src-tauri/src/lib.rs`，在 `mod stats;` 后加：

```rust
mod detect;
```

- [ ] **Step 3: 编译检查**

```bash
cd src-tauri && cargo check
```

期望：无错误。

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add ghostscript detection module"
```

---

## Task 4: 可插拔压缩引擎（trait + 注册表 + PDF 引擎）

**Files:**
- Create: `src-tauri/src/engine/mod.rs`
- Create: `src-tauri/src/engine/pdf.rs`

**Interfaces:**
- Consumes: `crate::error::{AppError, AppResult}`、`crate::stats`。
- Produces:
  - `Preset { id: String, label: String, gs_settings: String, rasterize: bool }`。
  - `CompressOptions { rasterize_text_layer: bool, output_dir: String }`。
  - `trait CompressionEngine`（见设计文档 §4.2）。
  - `engine::registry() -> &'static CompressionRegistry`，`CompressionRegistry::get(ext) -> Option<&dyn CompressionEngine>`。
  - `engine::pdf::PdfEngine`。

- [ ] **Step 1: 定义引擎 trait 与注册表**

创建 `src-tauri/src/engine/mod.rs`：

```rust
use crate::error::AppResult;
use crate::stats::CompressionResult;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub id: String,
    pub label: String,
    pub gs_settings: String,
    pub rasterize: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompressOptions {
    pub rasterize_text_layer: bool,
    pub output_dir: String,
}

pub trait CompressionEngine: Send + Sync {
    fn supported_extensions(&self) -> Vec<&'static str>;
    fn presets(&self) -> Vec<Preset>;
    fn compress(
        &self,
        input: &Path,
        preset: &Preset,
        opts: &CompressOptions,
    ) -> AppResult<CompressionResult>;
}

pub struct CompressionRegistry {
    engines: HashMap<&'static str, Box<dyn CompressionEngine>>,
}

impl CompressionRegistry {
    pub fn new() -> Self {
        Self { engines: HashMap::new() }
    }

    pub fn register(&mut self, engine: Box<dyn CompressionEngine>) {
        for ext in engine.supported_extensions() {
            self.engines.insert(ext, engine_ref(engine.as_ref()));
        }
    }
}
```

> 注：为简化，采用「每个扩展名持有独立 boxed 引擎」的方案，避免 `Rc`/多次借用问题。见 Step 2 修正版。

- [ ] **Step 2: 修正注册表实现（每个扩展名独立实例）**

将 `src-tauri/src/engine/mod.rs` 整体替换为：

```rust
use crate::error::AppResult;
use crate::stats::CompressionResult;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub mod pdf;

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub id: String,
    pub label: String,
    pub gs_settings: String,
    pub rasterize: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompressOptions {
    pub rasterize_text_layer: bool,
    pub output_dir: String,
}

pub trait CompressionEngine: Send + Sync {
    fn supported_extensions(&self) -> Vec<&'static str>;
    fn presets(&self) -> Vec<Preset>;
    fn compress(
        &self,
        input: &Path,
        preset: &Preset,
        opts: &CompressOptions,
    ) -> AppResult<CompressionResult>;
}

type EngineMap = HashMap<&'static str, Arc<dyn CompressionEngine>>;

fn build_registry() -> EngineMap {
    let mut m: EngineMap = HashMap::new();
    let pdf_engine = Arc::new(pdf::PdfEngine);
    for ext in pdf_engine.supported_extensions() {
        m.insert(ext, pdf_engine.clone());
    }
    m
}

pub fn registry() -> &'static EngineMap {
    static REG: OnceLock<EngineMap> = OnceLock::new();
    REG.get_or_init(build_registry)
}

pub fn engine_for(ext: &str) -> Option<Arc<dyn CompressionEngine>> {
    registry().get(ext).cloned()
}
```

- [ ] **Step 3: 实现 PDF 引擎**

创建 `src-tauri/src/engine/pdf.rs`：

```rust
use crate::engine::{CompressOptions, CompressionEngine, Preset};
use crate::error::{AppError, AppResult};
use crate::stats::{analyze, CompressionResult};
use std::path::Path;
use std::process::Command;

pub struct PdfEngine;

impl CompressionEngine for PdfEngine {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }

    fn presets(&self) -> Vec<Preset> {
        vec![
            Preset { id: "light".into(), label: "无损/轻度".into(), gs_settings: "/prepress".into(), rasterize: false },
            Preset { id: "balanced".into(), label: "平衡（默认）".into(), gs_settings: "/ebook".into(), rasterize: false },
            Preset { id: "extreme".into(), label: "极致压缩".into(), gs_settings: "/screen".into(), rasterize: false },
        ]
    }

    fn compress(
        &self,
        input: &Path,
        preset: &Preset,
        opts: &CompressOptions,
    ) -> AppResult<CompressionResult> {
        let start = std::time::Instant::now();
        let stem = input
            .file_stem()
            .ok_or_else(|| AppError::CompressFailed("无法解析文件名".into()))?
            .to_string_lossy()
            .to_string();
        let output = Path::new(&opts.output_dir).join(format!("{}_compressed.pdf", stem));

        let mut cmd = Command::new("gs");
        cmd.arg("-q")
            .arg("-dNOPAUSE")
            .arg("-dBATCH")
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-dPDFSETTINGS={}", preset.gs_settings));
        if opts.rasterize_text_layer || preset.rasterize {
            cmd.arg("-dCompatibilityLevel=1.4");
        }
        cmd.arg(format!("-sOutputFile={}", output.display()))
            .arg(input.to_string_lossy().to_string());

        let out = cmd.output().map_err(AppError::Io)?;
        if !out.status.success() {
            let msg = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(AppError::CompressFailed(msg));
        }

        let info = analyze(&output)?;
        let original_size = std::fs::metadata(input).map_err(AppError::Io)?.len();
        Ok(CompressionResult {
            input_path: input.to_string_lossy().to_string(),
            output_path: output.to_string_lossy().to_string(),
            original_size,
            compressed_size: info.file_size,
            page_count: info.page_count,
            preset: preset.id.clone(),
            text_layer_preserved: !(opts.rasterize_text_layer || preset.rasterize),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

- [ ] **Step 4: 补充 `stats.rs` 的 `CompressionResult` 定义**

修改 `src-tauri/src/stats.rs`，在 `PdfInfo` 后添加：

```rust
#[derive(Debug, Serialize)]
pub struct CompressionResult {
    pub input_path: String,
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub page_count: u32,
    pub preset: String,
    pub text_layer_preserved: bool,
    pub duration_ms: u64,
}
```

- [ ] **Step 5: 在 `lib.rs` 声明模块**

修改 `src-tauri/src/lib.rs`，在 `mod detect;` 后加：

```rust
mod engine;
```

- [ ] **Step 6: 编译检查**

```bash
cd src-tauri && cargo check
```

期望：无错误。

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat: add pluggable compression engine with PDF support"
```

---

## Task 5: 压缩队列调度与 Tauri 命令

**Files:**
- Create: `src-tauri/src/task.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `crate::engine::{engine_for, CompressOptions, Preset}`、`crate::error::AppResult`、`crate::stats::CompressionResult`、`crate::detect::detect_ghostscript`。
- Produces:
  - Tauri 命令 `check_gs() -> GhostscriptInfo`。
  - Tauri 命令 `list_presets(ext: String) -> Vec<Preset>`。
  - Tauri 命令 `compress_files(paths: Vec<String>, preset_id: String, opts: CompressOptions) -> Vec<CompressionResult>`（逐个顺序执行；单项失败返回 `Ok` 列表元素标记失败，见 Step 1 说明）。

- [ ] **Step 1: 定义队列调度**

创建 `src-tauri/src/task.rs`：

```rust
use crate::detect::detect_ghostscript;
use crate::detect::GhostscriptInfo;
use crate::engine::{engine_for, CompressOptions, Preset};
use crate::error::{AppError, AppResult};
use crate::stats::CompressionResult;
use std::path::Path;

/// 单个文件的压缩结果（成功或失败均返回，失败携带错误消息）
#[derive(serde::Serialize)]
pub struct FileResult {
    pub success: bool,
    #[serde(flatten)]
    pub data: Option<CompressionResult>,
    pub error: Option<String>,
}

pub fn compress_all(
    paths: Vec<String>,
    preset_id: String,
    opts: CompressOptions,
) -> Vec<FileResult> {
    paths
        .iter()
        .map(|p| compress_one(Path::new(p), &preset_id, &opts))
        .collect()
}

fn compress_one(path: &Path, preset_id: &str, opts: &CompressOptions) -> FileResult {
    if !path.exists() {
        return FileResult {
            success: false,
            data: None,
            error: Some(format!("文件不存在: {}", path.display())),
        };
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let engine = match engine_for(&ext) {
        Some(e) => e,
        None => {
            return FileResult {
                success: false,
                data: None,
                error: Some(format!("不支持的文件类型: .{}", ext)),
            };
        }
    };
    let preset = match engine.presets().into_iter().find(|p| p.id == preset_id) {
        Some(p) => p,
        None => {
            return FileResult {
                success: false,
                data: None,
                error: Some(format!("未知压缩档位: {}", preset_id)),
            };
        }
    };
    match engine.compress(path, &preset, opts) {
        Ok(r) => FileResult { success: true, data: Some(r), error: None },
        Err(e) => FileResult { success: false, data: None, error: Some(e.to_string()) },
    }
}

#[tauri::command]
pub fn check_gs() -> GhostscriptInfo {
    detect_ghostscript().unwrap_or_else(|_| GhostscriptInfo {
        found: false,
        path: None,
        version: None,
        install_hint: "无法检测 Ghostscript，请确认已安装。".into(),
    })
}

#[tauri::command]
pub fn list_presets(ext: String) -> Vec<Preset> {
    engine_for(&ext.to_lowercase()).map(|e| e.presets()).unwrap_or_default()
}

#[tauri::command]
pub fn compress_files(
    paths: Vec<String>,
    preset_id: String,
    opts: CompressOptions,
) -> Vec<FileResult> {
    compress_all(paths, preset_id, opts)
}
```

- [ ] **Step 2: 注册命令到 `lib.rs`**

修改 `src-tauri/src/lib.rs`，替换 `run()` 函数：

```rust
mod detect;
mod engine;
mod error;
mod stats;
mod task;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            task::check_gs,
            task::list_presets,
            task::compress_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 编译检查**

```bash
cd src-tauri && cargo check
```

期望：无错误。

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add compression queue and tauri commands"
```

---

## Task 6: 前端类型、API 封装与 UI 组件

**Files:**
- Create: `src/types.ts`
- Create: `src/api.ts`
- Modify: `src/App.vue`
- Create: `src/components/FileList.vue`
- Create: `src/components/PresetPicker.vue`
- Create: `src/components/OutputDirPicker.vue`

**Interfaces:**
- Consumes: Tauri 命令 `check_gs` / `list_presets` / `compress_files`。
- Produces: 完整可交互的压缩 UI。

- [ ] **Step 1: 定义前端类型**

创建 `src/types.ts`：

```ts
export interface GhostscriptInfo {
  found: boolean;
  path: string | null;
  version: string | null;
  install_hint: string;
}

export interface Preset {
  id: string;
  label: string;
  gs_settings: string;
  rasterize: boolean;
}

export interface CompressOptions {
  rasterize_text_layer: boolean;
  output_dir: string;
}

export interface CompressionResult {
  input_path: string;
  output_path: string;
  original_size: number;
  compressed_size: number;
  page_count: number;
  preset: string;
  text_layer_preserved: boolean;
  duration_ms: number;
}

export interface FileResult {
  success: boolean;
  input_path?: string;
  output_path?: string;
  original_size?: number;
  compressed_size?: number;
  page_count?: number;
  preset?: string;
  text_layer_preserved?: boolean;
  duration_ms?: number;
  error?: string | null;
}
```

- [ ] **Step 2: 封装 API**

创建 `src/api.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  CompressOptions,
  FileResult,
  GhostscriptInfo,
  Preset,
} from "./types";

export const checkGs = () => invoke<GhostscriptInfo>("check_gs");
export const listPresets = (ext: string) => invoke<Preset[]>("list_presets", { ext });
export const compressFiles = (
  paths: string[],
  presetId: string,
  opts: CompressOptions,
) => invoke<FileResult[]>("compress_files", {
  paths,
  presetId,
  opts,
});
```

- [ ] **Step 3: 实现档位选择组件**

创建 `src/components/PresetPicker.vue`：

```vue
<script setup lang="ts">
import type { Preset } from "../types";

defineProps<{ presets: Preset[]; modelValue: string }>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();
</script>

<template>
  <div class="preset-picker">
    <label v-for="p in presets" :key="p.id" class="preset">
      <input
        type="radio"
        :value="p.id"
        :checked="p.id === modelValue"
        @change="emit('update:modelValue', p.id)"
      />
      {{ p.label }}
    </label>
  </div>
</template>

<style scoped>
.preset-picker { display: flex; gap: 16px; }
.preset { cursor: pointer; }
</style>
```

- [ ] **Step 4: 实现输出目录选择组件**

创建 `src/components/OutputDirPicker.vue`：

```vue
<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

defineProps<{ modelValue: string }>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

async function pick() {
  const dir = await open({ directory: true });
  if (typeof dir === "string") emit("update:modelValue", dir);
}
</script>

<template>
  <div class="output-picker">
    <span>输出目录：</span>
    <input :value="modelValue" readonly class="path" />
    <button type="button" @click="pick">选择…</button>
  </div>
</template>

<style scoped>
.output-picker { display: flex; gap: 8px; align-items: center; }
.path { flex: 1; }
</style>
```

- [ ] **Step 5: 实现文件列表与结果组件**

创建 `src/components/FileList.vue`：

```vue
<script setup lang="ts">
import type { FileResult } from "../types";

defineProps<{ items: FileResult[] }>();

function fmt(bytes?: number) {
  if (bytes == null) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function ratio(r: FileResult) {
  if (!r.success || r.original_size == null || r.compressed_size == null) return "-";
  return `${(((r.compressed_size - r.original_size) / r.original_size) * 100).toFixed(1)}%`;
}
</script>

<template>
  <table class="file-list">
    <thead>
      <tr>
        <th>文件</th><th>原大小</th><th>新大小</th><th>压缩率</th>
        <th>页数</th><th>文字层</th><th>耗时</th><th>状态</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(r, i) in items" :key="i">
        <td>{{ r.input_path }}</td>
        <td>{{ fmt(r.original_size) }}</td>
        <td>{{ fmt(r.compressed_size) }}</td>
        <td>{{ ratio(r) }}</td>
        <td>{{ r.page_count ?? "-" }}</td>
        <td>{{ r.text_layer_preserved == null ? "-" : r.text_layer_preserved ? "可搜索" : "已栅格化" }}</td>
        <td>{{ r.duration_ms != null ? r.duration_ms + " ms" : "-" }}</td>
        <td :class="r.success ? 'ok' : 'fail'">{{ r.success ? "成功" : r.error }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.file-list { width: 100%; border-collapse: collapse; }
th, td { border: 1px solid #ddd; padding: 6px 10px; text-align: left; font-size: 13px; }
.ok { color: #16a34a; }
.fail { color: #dc2626; }
</style>
```

- [ ] **Step 6: 组装 `App.vue` 主界面**

修改 `src/App.vue`：

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { checkGs, listPresets, compressFiles } from "./api";
import type { FileResult, Preset } from "./types";
import PresetPicker from "./components/PresetPicker.vue";
import OutputDirPicker from "./components/OutputDirPicker.vue";
import FileList from "./components/FileList.vue";

const presets = ref<Preset[]>([]);
const presetId = ref("");
const outputDir = ref("");
const files = ref<string[]>([]);
const results = ref<FileResult[]>([]);
const rasterize = ref(false);
const gsMsg = ref("");
const busy = ref(false);

onMounted(async () => {
  const gs = await checkGs();
  if (gs.found) {
    gsMsg.value = `Ghostscript ${gs.version}`;
  } else {
    gsMsg.value = gs.install_hint;
  }
  presets.value = await listPresets("pdf");
  if (presets.value.length) presetId.value = presets.value[0].id;
});

async function addFiles() {
  const selected = await open({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (Array.isArray(selected)) files.value.push(...(selected as string[]));
}

async function start() {
  if (!files.value.length || !presetId.value || !outputDir.value) return;
  busy.value = true;
  results.value = [];
  results.value = await compressFiles(files.value, presetId.value, {
    rasterize_text_layer: rasterize.value,
    output_dir: outputDir.value,
  });
  busy.value = false;
}
</script>

<template>
  <div class="container">
    <header>
      <h1>PDF 压缩器</h1>
      <span class="gs">{{ gsMsg }}</span>
    </header>

    <section class="controls">
      <button type="button" @click="addFiles">添加 PDF 文件…</button>
      <PresetPicker v-model="presetId" :presets="presets" />
      <OutputDirPicker v-model="outputDir" />
      <label class="raster">
        <input type="checkbox" v-model="rasterize" :disabled="presetId !== 'extreme'" />
        栅格化去除文字层（仅极致压缩可用）
      </label>
      <button type="button" class="primary" :disabled="busy" @click="start">
        {{ busy ? "压缩中…" : "开始压缩" }}
      </button>
    </section>

    <FileList :items="results" />
  </div>
</template>

<style>
body { font-family: system-ui, -apple-system, sans-serif; margin: 0; background: #f5f6f8; }
.container { padding: 24px; max-width: 1080px; margin: 0 auto; }
header { display: flex; align-items: baseline; gap: 16px; }
h1 { margin: 0 0 16px; }
.gs { color: #666; font-size: 13px; }
.controls { display: flex; flex-direction: column; gap: 16px; margin-bottom: 24px; }
.raster { font-size: 13px; color: #444; }
button { padding: 8px 16px; cursor: pointer; }
.primary { background: #2563eb; color: #fff; border: none; border-radius: 6px; }
</style>
```

- [ ] **Step 7: 添加 dialog 插件依赖**

```bash
npm install @tauri-apps/plugin-dialog
```

并在 `src-tauri/Cargo.toml` 追加：

```toml
tauri-plugin-dialog = "2"
```

在 `src-tauri/src/lib.rs` 的 `run()` 中注册插件：

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(...)
```

- [ ] **Step 8: 编译前端**

```bash
npm run build
```

期望：无 TS 错误。

- [ ] **Step 9: Commit**

```bash
git add -A && git commit -m "feat: implement compression UI"
```

---

## Task 7: 集成测试与端到端验证

**Files:**
- Create: `src-tauri/tests/compress_test.rs`

**Interfaces:**
- Consumes: `engine::pdf::PdfEngine`、`stats::analyze`。
- Produces: 验证三档压缩输出的集成测试（需本机已安装 Ghostscript）。

- [ ] **Step 1: 编写集成测试**

创建 `src-tauri/tests/compress_test.rs`：

```rust
use pdf_compressor_lib::engine::pdf::PdfEngine;
use pdf_compressor_lib::engine::{CompressOptions, CompressionEngine};
use std::path::Path;

fn sample_pdf() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("pdfc_it");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("sample.pdf");
    let body = b"%PDF-1.4\n1 0 obj\n<< /Type /Page /MediaBox [0 0 1920 1080] >>\nendobj\n";
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn compresses_with_balanced_preset() {
    let engine = PdfEngine;
    let presets = engine.presets();
    let balanced = presets.iter().find(|p| p.id == "balanced").unwrap();
    let dir = std::env::temp_dir().join("pdfc_out");
    std::fs::create_dir_all(&dir).unwrap();
    let opts = CompressOptions {
        rasterize_text_layer: false,
        output_dir: dir.to_string_lossy().to_string(),
    };
    let r = engine.compress(Path::new(&sample_pdf()), balanced, &opts);
    assert!(r.is_ok(), "compression failed: {:?}", r.err());
    let r = r.unwrap();
    assert!(Path::new(&r.output_path).exists());
}
```

- [ ] **Step 2: 使 lib crate 对集成测试可见**

在 `src-tauri/Cargo.toml` 的 `[lib]` 段确认 `crate-type` 含 `"rlib"`（Task 1 已设置）。同时 `engine` 等模块需为 `pub` 或通过重导出暴露。修改 `src-tauri/src/lib.rs` 追加：

```rust
pub mod engine;
```

（将 `mod engine;` 改为 `pub mod engine;`，其余模块按需保持私有。）

- [ ] **Step 3: 运行集成测试**

```bash
# 需先确保本机已安装 Ghostscript
cd src-tauri && cargo test --test compress_test
```

期望：PASS（`compresses_with_balanced_preset` 通过）。

- [ ] **Step 4: 手动端到端验证**

用真实大 PDF（如 `doc/LOOKBOOK.pdf`）验证三档压缩：

```bash
gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite -dPDFSETTINGS=/ebook \
   -sOutputFile=/tmp/out_ebook.pdf doc/LOOKBOOK.pdf
```

对比 `/tmp/out_ebook.pdf` 与原文件体积，确认压缩生效且可正常打开。

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "test: add integration test for compression"
```

---

## Task 8: 打包发布配置

**Files:**
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: 全部已实现功能。
- Produces: 可分发安装包（`.dmg`/`.app` for macOS，`.msi`/`.exe` for Windows）。

- [ ] **Step 1: 完善打包配置**

修改 `src-tauri/tauri.conf.json`，将 `bundle` 段替换为：

```json
"bundle": {
  "active": true,
  "targets": ["dmg", "msi"],
  "icon": ["icons/icon.png", "icons/icon.icns", "icons/icon.ico"],
  "category": "Utility",
  "shortDescription": "轻量 PDF 压缩工具",
  "longDescription": "本地离线的 PDF 批量压缩工具，支持多档位质量调节。"
}
```

- [ ] **Step 2: 生成图标资源**

```bash
# 使用 Tauri CLI 生成完整图标集
npm run tauri icon src-tauri/icons/icon.png
```

期望：生成 `icon.icns`、`icon.ico` 及多尺寸 PNG。

- [ ] **Step 3: 构建 macOS 安装包**

```bash
npm run tauri build
```

期望：在 `src-tauri/target/release/bundle/` 生成 `.app` 与 `.dmg`。

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: configure bundle for release"
```

---

## Self-Review 记录

- **Spec 覆盖**：档位（Task 4 预设）、文字层选项（Task 4/6）、离线（Ghostscript 本地调用）、批量（Task 5 `compress_all`）、另存新文件（Task 4 输出 `_compressed`）、详细结果（Task 6 表格）、可扩展引擎（Task 4 trait+注册表）、gs 检测（Task 3）、错误处理（Task 2/5）、测试（Task 2/7）。均覆盖。
- **占位符扫描**：无 TBD/TODO；所有步骤含完整代码。
- **类型一致性**：`CompressionResult`、`Preset`、`CompressOptions`、`GhostscriptInfo`、`FileResult` 前后端命名一致；`engine_for`/`registry`/`PdfEngine` 在 Task 4 定义、Task 5/7 引用，签名一致。

---

## 实现顺序建议

按 Task 1 → 8 顺序执行。Task 7 的集成测试依赖本机安装 Ghostscript，可在实现阶段一并安装（`brew install ghostscript`）。
