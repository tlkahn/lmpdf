# lmpdf

Safe, ergonomic Rust bindings to [PDFium](https://pdfium.googlesource.com/pdfium/) — Google's open-source PDF renderer. Load PDFs from files or memory, render pages to bitmaps, and export images with a builder-style API and zero raw pointers in user code.

## Features

- **Dynamic loading** — load `libpdfium` at runtime; no compile-time linking required
- **Safe API** — newtype handles and arena-based ownership eliminate use-after-free and double-free classes of bugs
- **Render to bitmap** — render any page to BGRA/BGR/grayscale pixel buffers with configurable scale, rotation, max dimensions, and render flags
- **Image export** — optional `image` feature converts bitmaps directly to `image::DynamicImage` for saving as PNG, JPEG, etc.
- **Version-gated bindings** — target specific PDFium releases (`pdfium_7811`, `pdfium_7763`, `pdfium_7543`) or use `pdfium_latest`
- **Proc-macro generated FFI** — a custom `pdfium_ffi!` macro generates both the trait definition and backend implementations, keeping bindings in sync automatically

## Crate structure

| Crate | Purpose |
|---|---|
| `lmpdf` | High-level API: `Pdfium`, `Document`, `Bitmap`, `RenderConfig` |
| `lmpdf-sys` | Raw FFI bindings (via `bindgen`), safe wrappers, dynamic loading |
| `lmpdf-sys-macros` | Proc-macro that generates `PdfiumBindings` trait + backends |
| `lmpdf-bundle` | Helpers for bundling a PDFium binary with your application |

## Quick start

### Prerequisites

Download a pre-built PDFium binary for your platform from [pdfium-binaries](https://github.com/nicehash/pdfium-binaries/releases) (or build from source). Set the `PDFIUM_PATH` environment variable to the shared library path.

### Usage

Add `lmpdf` to your `Cargo.toml`:

```toml
[dependencies]
lmpdf = { git = "https://github.com/tlkahn/lmpdf.git" }
```

Render every page of a PDF to PNG (the default config renders at 144 DPI with high-quality flags):

```rust
use lmpdf::{Pdfium, RenderConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdfium = Pdfium::open(std::env::var("PDFIUM_PATH")?)?;
    let doc = pdfium.open_document("input.pdf", None)?;
    let config = RenderConfig::new(); // 144 DPI, PRINTING + LCD_TEXT + ANNOTATIONS

    for i in 0..doc.page_count() {
        let page = doc.page(i)?;
        let bitmap = doc.render_page(page, &config)?;
        let img = bitmap.to_image(); // requires `image` feature
        img.save(PathBuf::from(format!("page_{}.png", i + 1)))?;
    }
    Ok(())
}
```

### Render configuration

`RenderConfig` uses a builder pattern for full control over output:

```rust
use lmpdf::{RenderConfig, RenderFlags, Rotation};
use lmpdf::BitmapFormat;

let config = RenderConfig::new()
    .dpi(300)                           // 300 DPI (default is 144)
    .max_width(4096)                    // clamp width, preserve aspect ratio
    .rotation(Rotation::Degrees90)      // rotate 90°
    .format(BitmapFormat::Bgr)          // 24-bit BGR (no alpha)
    .flags(RenderFlags::PRINTING | RenderFlags::ANNOTATIONS)
    .background_color(0xFFFFFFFF);      // white
```

## Building from source

```sh
git clone https://github.com/tlkahn/lmpdf.git
cd lmpdf
cargo build --workspace
cargo test --workspace
```

`bindgen` requires `libclang` — on Debian/Ubuntu: `sudo apt install libclang-dev`.

## Roadmap

- [ ] Text extraction API
- [ ] Form filling support
- [ ] Annotation reading/writing
- [ ] Static linking support (`lmpdf-bundle`)
- [ ] WASM backend (groundwork in `gen_wasm.rs`)
- [ ] `crates.io` publishing

## License

PDFium is licensed under the Apache License 2.0 and the BSD 3-Clause License. See the [PDFium project](https://pdfium.googlesource.com/pdfium/) for details.
