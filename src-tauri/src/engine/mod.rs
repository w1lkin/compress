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
