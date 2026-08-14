use crate::error::AppResult;
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
