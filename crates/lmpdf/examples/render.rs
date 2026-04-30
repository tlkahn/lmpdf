use std::path::PathBuf;

use lmpdf::{Pdfium, RenderConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: render <input.pdf> <output_prefix>");
        eprintln!("  Renders each page to <output_prefix>_page<N>.jpg");
        eprintln!("  Requires PDFIUM_PATH env var pointing to libpdfium");
        std::process::exit(1);
    }

    let input = &args[1];
    let output_prefix = &args[2];

    let pdfium_path =
        std::env::var("PDFIUM_PATH").expect("Set PDFIUM_PATH to the path of libpdfium");
    let pdfium = Pdfium::open(&pdfium_path).expect("Failed to open pdfium library");
    let doc = pdfium
        .open_document(input, None)
        .expect("Failed to open PDF");

    let config = RenderConfig::new().scale(2.0);

    for i in 0..doc.page_count() {
        let page_ref = doc.page(i).expect("Failed to load page");
        let bitmap = doc
            .render_page(page_ref, &config)
            .expect("Failed to render page");

        let img = bitmap.to_image();
        let path = PathBuf::from(format!("{output_prefix}_page{}.jpg", i + 1));
        img.save(&path).expect("Failed to save image");
        println!(
            "Saved {} ({}x{})",
            path.display(),
            bitmap.width(),
            bitmap.height()
        );
    }
}
