# PDF 压缩器

跨平台（Windows / macOS）的轻量 PDF 批量压缩桌面应用。本地离线运行，支持多档位质量调节。

## 特性

- **本地离线**：压缩完全在本地进行，不依赖任何云服务
- **批量处理**：一次导入多个 PDF 排队压缩
- **三档质量可调**：
  - 无损/轻度（`/prepress`）— 体积略降，质量零损失
  - 平衡（`/ebook`，默认）— 显著缩小，肉眼难辨
  - 极致压缩（`/screen`）— 压到很小，质量明显下降
- **文字层保留**：默认保留文字（可搜索/复制）；极致压缩档位可选「栅格化去除文字层」
- **详细结果**：展示原大小、新大小、压缩率、页数、文字层状态、耗时
- **输出目录记忆**：选择一次后自动记住，下次启动默认使用
- **另存新文件**：输出 `原名_compressed.pdf`，保留原文件

## 技术栈

- **框架**：Tauri 2（Rust 后端 + 系统 WebView）
- **前端**：Vue 3 + TypeScript + Vite
- **压缩引擎**：Ghostscript（`pdfwrite` 设备）

## 前置依赖

### Ghostscript（必需）

本应用**不捆绑** Ghostscript，需自行安装（这是出于许可合规考虑，Ghostscript 采用 AGPL 双许可，随应用分发需商业授权）。

| 平台 | 安装方式 |
|------|---------|
| macOS | `brew install ghostscript` |
| Windows | 从 [Ghostscript 官方下载页](https://ghostscript.com/releases/gsdnld.html) 下载并安装 |

> 应用启动时会自动检测 Ghostscript，未安装则显示安装引导。

## 从源码构建

### 通用前置

1. **Node.js** >= 22
2. **Rust** >= 1.77（安装：https://rustup.rs/）

> 国内网络建议配置镜像加速：
> - Rust crates：`~/.cargo/config.toml` 指向清华/rsproxy 镜像
> - Homebrew（macOS）：设置 `HOMEBREW_API_DOMAIN` / `HOMEBREW_BOTTLE_DOMAIN` 指向清华镜像

```bash
# 安装前端依赖
npm install

# 开发模式（热更新）
npm run tauri dev

# 构建生产安装包
npm run tauri build
```

### macOS 构建

```bash
npm install
npm run tauri build
```

产物位置：`src-tauri/target/release/bundle/dmg/PDF压缩器_*.dmg`

> 注意：macOS 上只能构建 `.dmg`，无法构建 Windows 的 `.msi`。

### Windows 构建

在 Windows 机器上执行：

```powershell
# 1. 安装 Node.js（https://nodejs.org）和 Rust（https://rustup.rs）
# 2. Windows 10/11 已内置 WebView2，无需额外安装

# 3. 配置国内镜像（可选但推荐）
#    ~/.cargo/config.toml 添加：
#    [source.crates-io]
#    replace-with = "rsproxy"
#    [source.rsproxy]
#    registry = "sparse+https://rsproxy.cn/index/"
#    [net]
#    git-fetch-with-cli = true

# 4. 克隆并构建
git clone git@github.com:w1lkin/pdf-compressor.git
cd pdf-compressor
npm install
npm run tauri build
```

产物位置：`src-tauri/target/release/bundle/msi/PDF压缩器_*.msi`

## 项目结构

```
compress/
├── src/                    # 前端源码（Vue 3 + TS）
│   ├── App.vue             # 主界面
│   ├── api.ts              # Tauri IPC 封装
│   ├── types.ts            # 类型定义
│   └── components/         # UI 组件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── engine/         # 可插拔压缩引擎（trait + 注册表）
│   │   │   ├── mod.rs      # CompressionEngine trait + 注册表
│   │   │   └── pdf.rs      # Ghostscript PDF 引擎
│   │   ├── detect.rs       # Ghostscript 检测
│   │   ├── task.rs         # 压缩队列调度 + Tauri 命令
│   │   ├── stats.rs        # PDF 统计
│   │   ├── error.rs        # 错误类型
│   │   ├── lib.rs          # 入口 + 命令注册
│   │   └── main.rs         # main
│   ├── capabilities/       # Tauri 权限配置
│   ├── tests/              # 集成测试
│   └── tauri.conf.json     # Tauri 配置
└── docs/superpowers/       # 设计文档与实现计划
```

## 可扩展架构

压缩引擎采用可插拔设计：每种格式实现 `CompressionEngine` trait 并注册即可，前端按文件类型自适应渲染。当前仅实现 PDF（Ghostscript），未来可扩展图片等多格式压缩。

## 许可证

本项目代码采用 MIT 许可（如需可补充 LICENSE 文件）。Ghostscript 为 AGPL 双许可，本应用通过"运行时调用用户自行安装的 Ghostscript"的方式规避分发合规问题。

## 免责声明

请勿使用本工具压缩受版权保护的内容（如商业画册、出版物）用于分发。
