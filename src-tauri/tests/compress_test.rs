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
