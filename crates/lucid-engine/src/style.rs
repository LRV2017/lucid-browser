/// First-stage style model. CSS tokenization, parsing, cascade and selector matching
/// are separate upcoming milestones; these values are not yet a CSS implementation.
#[derive(Debug, Clone, PartialEq)]
pub enum Display { Block, Inline, None }

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: [u8; 4],
    pub background: [u8; 4],
    pub font_size_px: f32,
    pub margin_px: [f32; 4],
    pub padding_px: [f32; 4],
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            color: [24, 24, 28, 255],
            background: [0, 0, 0, 0],
            font_size_px: 16.0,
            margin_px: [0.0; 4],
            padding_px: [0.0; 4],
        }
    }
}
