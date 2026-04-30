use std::fmt;
use std::os::raw::c_int;

use lmpdf_sys::{FPDFBitmap_BGR, FPDFBitmap_BGRA, FPDFBitmap_BGRx, FPDFBitmap_Gray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitmapFormat {
    Gray,
    Bgr,
    BgrX,
    #[default]
    Bgra,
}

impl BitmapFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            BitmapFormat::Gray => 1,
            BitmapFormat::Bgr => 3,
            BitmapFormat::BgrX | BitmapFormat::Bgra => 4,
        }
    }

    pub fn from_raw(raw: c_int) -> Option<Self> {
        match raw {
            x if x == FPDFBitmap_Gray => Some(BitmapFormat::Gray),
            x if x == FPDFBitmap_BGR => Some(BitmapFormat::Bgr),
            x if x == FPDFBitmap_BGRx => Some(BitmapFormat::BgrX),
            x if x == FPDFBitmap_BGRA => Some(BitmapFormat::Bgra),
            _ => None,
        }
    }

    pub fn to_raw(self) -> c_int {
        match self {
            BitmapFormat::Gray => FPDFBitmap_Gray,
            BitmapFormat::Bgr => FPDFBitmap_BGR,
            BitmapFormat::BgrX => FPDFBitmap_BGRx,
            BitmapFormat::Bgra => FPDFBitmap_BGRA,
        }
    }

    pub fn has_alpha(self) -> bool {
        matches!(self, BitmapFormat::Bgra)
    }
}

pub struct Bitmap {
    data: Vec<u8>,
    width: u32,
    height: u32,
    stride: u32,
    format: BitmapFormat,
}

impl Bitmap {
    pub fn new(data: Vec<u8>, width: u32, height: u32, stride: u32, format: BitmapFormat) -> Self {
        Self {
            data,
            width,
            height,
            stride,
            format,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn format(&self) -> BitmapFormat {
        self.format
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("format", &self.format)
            .field("data_len", &self.data.len())
            .finish()
    }
}

#[cfg(feature = "image")]
impl Bitmap {
    pub fn to_image(&self) -> image::DynamicImage {
        use image::{DynamicImage, GrayImage, RgbaImage, RgbImage};

        let w = self.width as usize;
        let h = self.height as usize;
        let stride = self.stride as usize;
        let bpp = self.format.bytes_per_pixel();

        match self.format {
            BitmapFormat::Gray => {
                let mut buf = Vec::with_capacity(w * h);
                for row in 0..h {
                    let start = row * stride;
                    buf.extend_from_slice(&self.data[start..start + w]);
                }
                DynamicImage::ImageLuma8(
                    GrayImage::from_raw(self.width, self.height, buf).unwrap(),
                )
            }
            BitmapFormat::Bgr => {
                let mut buf = Vec::with_capacity(w * h * 3);
                for row in 0..h {
                    let row_start = row * stride;
                    for col in 0..w {
                        let px = row_start + col * bpp;
                        buf.push(self.data[px + 2]); // R
                        buf.push(self.data[px + 1]); // G
                        buf.push(self.data[px]);      // B
                    }
                }
                DynamicImage::ImageRgb8(
                    RgbImage::from_raw(self.width, self.height, buf).unwrap(),
                )
            }
            BitmapFormat::BgrX => {
                let mut buf = Vec::with_capacity(w * h * 3);
                for row in 0..h {
                    let row_start = row * stride;
                    for col in 0..w {
                        let px = row_start + col * bpp;
                        buf.push(self.data[px + 2]); // R
                        buf.push(self.data[px + 1]); // G
                        buf.push(self.data[px]);      // B
                    }
                }
                DynamicImage::ImageRgb8(
                    RgbImage::from_raw(self.width, self.height, buf).unwrap(),
                )
            }
            BitmapFormat::Bgra => {
                let mut buf = Vec::with_capacity(w * h * 4);
                for row in 0..h {
                    let row_start = row * stride;
                    for col in 0..w {
                        let px = row_start + col * bpp;
                        buf.push(self.data[px + 2]); // R
                        buf.push(self.data[px + 1]); // G
                        buf.push(self.data[px]);      // B
                        buf.push(self.data[px + 3]); // A
                    }
                }
                DynamicImage::ImageRgba8(
                    RgbaImage::from_raw(self.width, self.height, buf).unwrap(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_format_bytes_per_pixel() {
        assert_eq!(BitmapFormat::Gray.bytes_per_pixel(), 1);
        assert_eq!(BitmapFormat::Bgr.bytes_per_pixel(), 3);
        assert_eq!(BitmapFormat::BgrX.bytes_per_pixel(), 4);
        assert_eq!(BitmapFormat::Bgra.bytes_per_pixel(), 4);
    }

    #[test]
    fn bitmap_format_from_raw_round_trip() {
        for fmt in [
            BitmapFormat::Gray,
            BitmapFormat::Bgr,
            BitmapFormat::BgrX,
            BitmapFormat::Bgra,
        ] {
            assert_eq!(BitmapFormat::from_raw(fmt.to_raw()), Some(fmt));
        }
    }

    #[test]
    fn bitmap_format_from_raw_unknown() {
        assert_eq!(BitmapFormat::from_raw(0), None);
        assert_eq!(BitmapFormat::from_raw(99), None);
    }

    #[test]
    fn bitmap_format_default_is_bgra() {
        assert_eq!(BitmapFormat::default(), BitmapFormat::Bgra);
    }

    #[test]
    fn bitmap_format_has_alpha() {
        assert!(!BitmapFormat::Gray.has_alpha());
        assert!(!BitmapFormat::Bgr.has_alpha());
        assert!(!BitmapFormat::BgrX.has_alpha());
        assert!(BitmapFormat::Bgra.has_alpha());
    }

    #[test]
    fn bitmap_construction_and_accessors() {
        let data = vec![0u8; 400];
        let bm = Bitmap::new(data.clone(), 10, 10, 40, BitmapFormat::Bgra);
        assert_eq!(bm.width(), 10);
        assert_eq!(bm.height(), 10);
        assert_eq!(bm.stride(), 40);
        assert_eq!(bm.format(), BitmapFormat::Bgra);
        assert_eq!(bm.data(), &data[..]);
    }

    #[test]
    fn bitmap_into_data_ownership() {
        let data = vec![1u8; 100];
        let bm = Bitmap::new(data.clone(), 10, 10, 10, BitmapFormat::Gray);
        let owned = bm.into_data();
        assert_eq!(owned, data);
    }

    #[test]
    fn bitmap_debug_no_pixel_dump() {
        let data = vec![0u8; 400];
        let bm = Bitmap::new(data, 10, 10, 40, BitmapFormat::Bgra);
        let dbg = format!("{:?}", bm);
        assert!(dbg.contains("Bitmap"));
        assert!(dbg.contains("width: 10"));
        assert!(dbg.contains("data_len: 400"));
        assert!(!dbg.contains("[0, 0, 0"));
    }
}
