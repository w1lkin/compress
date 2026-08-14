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
        Self {
            message: e.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
