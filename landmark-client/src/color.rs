#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// sRGB-converted representation of a color.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RawColor {
    r: f32,
    g: f32,
    b: f32,
}

impl From<Color> for RawColor {
    fn from(value: Color) -> Self {
        Self {
            r: f32::powf((value.r as f32 / 255.0 + 0.055) / 1.055, 2.4),
            g: f32::powf((value.g as f32 / 255.0 + 0.055) / 1.055, 2.4),
            b: f32::powf((value.b as f32 / 255.0 + 0.055) / 1.055, 2.4),
        }
    }
}
