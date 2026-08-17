use crate::detect::{detect_ghostscript, GhostscriptInfo};
use crate::engine::{engine_for, CompressOptions, Preset};
use crate::stats::CompressionResult;
use serde::Serialize;
use std::path::Path;
use tauri::Emitter;

/// 单个文件的压缩结果（成功或失败均返回，失败携带错误消息）
#[derive(Clone, Serialize)]
pub struct FileResult {
    pub success: bool,
    #[serde(flatten)]
    pub data: Option<CompressionResult>,
    pub error: Option<String>,
}

/// 压缩进度事件负载
#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    /// 当前处理到第几个（从 1 开始）
    pub current: usize,
    /// 总文件数
    pub total: usize,
    /// 当前文件名
    pub file: String,
    /// 当前文件是否已完成（true = 刚完成一个）
    pub done: bool,
}

pub fn compress_all(
    paths: Vec<String>,
    preset_id: String,
    opts: CompressOptions,
    on_progress: impl Fn(ProgressPayload),
) -> Vec<FileResult> {
    let total = paths.len();
    paths
        .iter()
        .enumerate()
        .map(|(i, p)| {
            on_progress(ProgressPayload {
                current: i + 1,
                total,
                file: p.clone(),
                done: false,
            });
            let r = compress_one(Path::new(p), &preset_id, &opts);
            on_progress(ProgressPayload {
                current: i + 1,
                total,
                file: p.clone(),
                done: true,
            });
            r
        })
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
        Ok(r) => FileResult {
            success: true,
            data: Some(r),
            error: None,
        },
        Err(e) => FileResult {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
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
    engine_for(&ext.to_lowercase())
        .map(|e| e.presets())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn compress_files(
    app: tauri::AppHandle,
    paths: Vec<String>,
    preset_id: String,
    opts: CompressOptions,
) -> Vec<FileResult> {
    let app_handle = app.clone();
    let results = tauri::async_runtime::spawn_blocking(move || {
        compress_all(paths, preset_id, opts, move |p| {
            let _ = app_handle.emit("compress-progress", p);
        })
    })
    .await
    .unwrap_or_default();
    results
}
