use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PdfInfo {
    pub page_count: u32,
    pub file_size: u64,
}

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
