# Task 1: 项目脚手架（Tauri 2 + Vue 3）

**Files:**
- Create: `package.json`, `vite.config.ts`, `index.html`, `tsconfig.json`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src/main.ts`, `src/App.vue`

**Interfaces:**
- Consumes: 无（首个任务）。
- Produces: 可运行的空 Tauri 应用骨架，`npm run tauri dev` 能弹出窗口。

**环境说明（重要）：**
- 当前机器：macOS (darwin)，Node v22.15.1 / npm 10.9.2 已装，Rust 工具链未装，Ghostscript 未装（本任务不涉及 gs）。
- 安装 Rust 工具链：优先用 `brew install rust`（更快、无需交互）；若 brew 不可用则用 rustup。
- 本任务无需 Ghostscript。

**Step 1: 安装 Rust 工具链**

```bash
brew install rust
# 或备选：curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustc -V   # 期望 >= 1.77
cargo -V
```

**Step 2: 创建前端工程文件**

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

**Step 3: 创建 Rust 后端文件**

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

[lib]
name = "pdf_compressor_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
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

**Step 4: 安装前端依赖并验证启动**

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

**Step 5: 生成应用图标并验证 `tauri dev`**

```bash
mkdir -p src-tauri/icons
```

放置一个 512x512 的 PNG 图标到 `src-tauri/icons/icon.png`（可用任意纯色占位 PNG；可用 `npm run tauri icon` 生成，或手动生成一张）。

验证（Tauri dev 需要 GUI 窗口，可能无法在无显示环境运行；至少验证编译通过）：

```bash
npm run build          # 前端构建应无错误
cd src-tauri && cargo check   # Rust 编译应无错误
```

如果 `npm run tauri dev` 无法弹出窗口（无图形环境），以 `cargo check` + `npm run build` 通过作为验收标准，并在报告中说明。

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + Vue 3 project"
```

注意：提交时不要包含 `node_modules/`、`src-tauri/target/`、`dist/`。建议创建 `.gitignore` 排除它们（node_modules, dist, src-tauri/target）。
