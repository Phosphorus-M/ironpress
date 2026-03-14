/// Page size in points (1 pt = 1/72 inch).
#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

impl PageSize {
    pub const A4: Self = Self {
        width: 595.28,
        height: 841.89,
    };
    pub const LETTER: Self = Self {
        width: 612.0,
        height: 792.0,
    };
}

impl Default for PageSize {
    fn default() -> Self {
        Self::A4
    }
}

/// Page margins in points.
#[derive(Debug, Clone, Copy)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn uniform(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self::uniform(72.0) // 1 inch
    }
}

/// A rectangle defined by position and size.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Edge sizes for margin, padding, border.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// RGBA color.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn to_f32_rgb(self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}
