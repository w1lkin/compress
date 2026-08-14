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
            Preset {
                id: "light".into(),
                label: "无损/轻度".into(),
                gs_settings: "/prepress".into(),
                rasterize: false,
            },
            Preset {
                id: "balanced".into(),
                label: "平衡（默认）".into(),
                gs_settings: "/ebook".into(),
                rasterize: false,
            },
            Preset {
                id: "extreme".into(),
                label: "极致压缩".into(),
                gs_settings: "/screen".into(),
                rasterize: false,
            },
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
