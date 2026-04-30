use bitflags::bitflags;

use crate::bitmap::BitmapFormat;
use crate::error::{Error, RenderError};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RenderFlags: i32 {
        const ANNOTATIONS       = 0x01;
        const LCD_TEXT          = 0x02;
        const NO_NATIVETEXT    = 0x04;
        const GRAYSCALE        = 0x08;
        const DEBUG_INFO       = 0x80;
        const NO_CATCH         = 0x100;
        const RENDER_LIMITEDIMAGECACHE = 0x200;
        const RENDER_FORCEHALFTONE     = 0x400;
        const PRINTING         = 0x800;
        const RENDER_NO_SMOOTHTEXT    = 0x1000;
        const RENDER_NO_SMOOTHIMAGE   = 0x2000;
        const RENDER_NO_SMOOTHPATH    = 0x4000;
        const REVERSE_BYTE_ORDER      = 0x10;
    }
}

impl Default for RenderFlags {
    fn default() -> Self {
        RenderFlags::ANNOTATIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Degrees90,
    Degrees180,
    Degrees270,
}

impl Rotation {
    pub fn to_raw(self) -> i32 {
        match self {
            Rotation::None => 0,
            Rotation::Degrees90 => 1,
            Rotation::Degrees180 => 2,
            Rotation::Degrees270 => 3,
        }
    }

    pub fn swaps_dimensions(self) -> bool {
        matches!(self, Rotation::Degrees90 | Rotation::Degrees270)
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::None
    }
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) scale: Option<f32>,
    pub(crate) max_width: Option<u32>,
    pub(crate) max_height: Option<u32>,
    pub(crate) rotation: Rotation,
    pub(crate) background_color: lmpdf_sys::FPDF_DWORD,
    pub(crate) flags: RenderFlags,
    pub(crate) format: BitmapFormat,
}

impl RenderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, w: u32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn height(mut self, h: u32) -> Self {
        self.height = Some(h);
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.scale = Some(s);
        self
    }

    pub fn max_width(mut self, w: u32) -> Self {
        self.max_width = Some(w);
        self
    }

    pub fn max_height(mut self, h: u32) -> Self {
        self.max_height = Some(h);
        self
    }

    pub fn rotation(mut self, r: Rotation) -> Self {
        self.rotation = r;
        self
    }

    pub fn background_color(mut self, color: lmpdf_sys::FPDF_DWORD) -> Self {
        self.background_color = color;
        self
    }

    pub fn flags(mut self, flags: RenderFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn format(mut self, format: BitmapFormat) -> Self {
        self.format = format;
        self
    }

    pub fn no_annotations(mut self) -> Self {
        self.flags.remove(RenderFlags::ANNOTATIONS);
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            scale: None,
            max_width: None,
            max_height: None,
            rotation: Rotation::None,
            background_color: 0xFFFFFFFF,
            flags: RenderFlags::default(),
            format: BitmapFormat::default(),
        }
    }
}

pub fn compute_target_dimensions(
    page_w: f32,
    page_h: f32,
    config: &RenderConfig,
) -> Result<(u32, u32), Error> {
    if page_w <= 0.0 || page_h <= 0.0 {
        return Err(RenderError::InvalidDimensions { width: 0, height: 0 }.into());
    }

    let (pw, ph) = if config.rotation.swaps_dimensions() {
        (page_h, page_w)
    } else {
        (page_w, page_h)
    };

    let (mut w, mut h) = if let Some(scale) = config.scale {
        (pw * scale, ph * scale)
    } else {
        (pw, ph)
    };

    match (config.width, config.height) {
        (Some(ew), Some(eh)) => {
            w = ew as f32;
            h = eh as f32;
        }
        (Some(ew), None) => {
            let ratio = ew as f32 / w;
            w = ew as f32;
            h *= ratio;
        }
        (None, Some(eh)) => {
            let ratio = eh as f32 / h;
            h = eh as f32;
            w *= ratio;
        }
        (None, None) => {}
    }

    if let Some(max_w) = config.max_width {
        if w > max_w as f32 {
            let ratio = max_w as f32 / w;
            w = max_w as f32;
            h *= ratio;
        }
    }

    if let Some(max_h) = config.max_height {
        if h > max_h as f32 {
            let ratio = max_h as f32 / h;
            h = max_h as f32;
            w *= ratio;
        }
    }

    let fw = (w.round() as u32).max(1);
    let fh = (h.round() as u32).max(1);

    Ok((fw, fh))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_flags_default_is_annotations() {
        assert_eq!(RenderFlags::default(), RenderFlags::ANNOTATIONS);
    }

    #[test]
    fn render_flags_bit_values() {
        assert_eq!(RenderFlags::ANNOTATIONS.bits(), 0x01);
        assert_eq!(RenderFlags::LCD_TEXT.bits(), 0x02);
        assert_eq!(RenderFlags::PRINTING.bits(), 0x800);
    }

    #[test]
    fn render_flags_combine() {
        let combined = RenderFlags::ANNOTATIONS | RenderFlags::PRINTING;
        assert!(combined.contains(RenderFlags::ANNOTATIONS));
        assert!(combined.contains(RenderFlags::PRINTING));
        assert!(!combined.contains(RenderFlags::GRAYSCALE));
    }

    #[test]
    fn rotation_to_raw() {
        assert_eq!(Rotation::None.to_raw(), 0);
        assert_eq!(Rotation::Degrees90.to_raw(), 1);
        assert_eq!(Rotation::Degrees180.to_raw(), 2);
        assert_eq!(Rotation::Degrees270.to_raw(), 3);
    }

    #[test]
    fn rotation_default() {
        assert_eq!(Rotation::default(), Rotation::None);
    }

    #[test]
    fn render_config_defaults() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.width, None);
        assert_eq!(cfg.height, None);
        assert_eq!(cfg.scale, None);
        assert_eq!(cfg.max_width, None);
        assert_eq!(cfg.max_height, None);
        assert_eq!(cfg.rotation, Rotation::None);
        assert_eq!(cfg.background_color, 0xFFFFFFFF);
        assert_eq!(cfg.flags, RenderFlags::ANNOTATIONS);
        assert_eq!(cfg.format, BitmapFormat::Bgra);
    }

    #[test]
    fn render_config_builder_methods() {
        let cfg = RenderConfig::new()
            .width(800)
            .height(600)
            .scale(2.0)
            .max_width(1920)
            .max_height(1080)
            .rotation(Rotation::Degrees90)
            .background_color(0xFF000000)
            .flags(RenderFlags::PRINTING)
            .format(BitmapFormat::Bgr);

        assert_eq!(cfg.width, Some(800));
        assert_eq!(cfg.height, Some(600));
        assert_eq!(cfg.scale, Some(2.0));
        assert_eq!(cfg.max_width, Some(1920));
        assert_eq!(cfg.max_height, Some(1080));
        assert_eq!(cfg.rotation, Rotation::Degrees90);
        assert_eq!(cfg.background_color, 0xFF000000);
        assert_eq!(cfg.flags, RenderFlags::PRINTING);
        assert_eq!(cfg.format, BitmapFormat::Bgr);
    }

    #[test]
    fn render_config_no_annotations() {
        let cfg = RenderConfig::new().no_annotations();
        assert!(!cfg.flags.contains(RenderFlags::ANNOTATIONS));
    }

    #[test]
    fn compute_default_uses_page_size() {
        let cfg = RenderConfig::default();
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 612);
        assert_eq!(h, 792);
    }

    #[test]
    fn compute_explicit_width_height() {
        let cfg = RenderConfig::new().width(800).height(600);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
    }

    #[test]
    fn compute_scale_factor() {
        let cfg = RenderConfig::new().scale(2.0);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 1224);
        assert_eq!(h, 1584);
    }

    #[test]
    fn compute_single_dim_preserves_aspect() {
        let cfg = RenderConfig::new().width(306);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 306);
        assert_eq!(h, 396);
    }

    #[test]
    fn compute_single_height_preserves_aspect() {
        let cfg = RenderConfig::new().height(396);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 306);
        assert_eq!(h, 396);
    }

    #[test]
    fn compute_max_width_clamps() {
        let cfg = RenderConfig::new().scale(2.0).max_width(612);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 612);
        assert_eq!(h, 792);
    }

    #[test]
    fn compute_max_height_clamps() {
        let cfg = RenderConfig::new().scale(2.0).max_height(792);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 612);
        assert_eq!(h, 792);
    }

    #[test]
    fn compute_rotation_swaps_dims() {
        let cfg = RenderConfig::new().rotation(Rotation::Degrees90);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 792);
        assert_eq!(h, 612);
    }

    #[test]
    fn compute_rotation_180_no_swap() {
        let cfg = RenderConfig::new().rotation(Rotation::Degrees180);
        let (w, h) = compute_target_dimensions(612.0, 792.0, &cfg).unwrap();
        assert_eq!(w, 612);
        assert_eq!(h, 792);
    }

    #[test]
    fn compute_zero_page_size_errors() {
        let cfg = RenderConfig::default();
        assert!(compute_target_dimensions(0.0, 792.0, &cfg).is_err());
        assert!(compute_target_dimensions(612.0, 0.0, &cfg).is_err());
        assert!(compute_target_dimensions(0.0, 0.0, &cfg).is_err());
    }

    #[test]
    fn compute_ensures_minimum_one() {
        let cfg = RenderConfig::new().scale(0.001);
        let (w, h) = compute_target_dimensions(1.0, 1.0, &cfg).unwrap();
        assert!(w >= 1);
        assert!(h >= 1);
    }
}
