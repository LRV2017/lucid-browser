/// Geometry primitives for Lucid's future formatting and layout algorithms.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBox {
    pub content: Rect,
    pub margin: Edges,
    pub padding: Edges,
    pub border: Edges,
}

impl LayoutBox {
    pub fn outer_width(&self) -> f32 {
        self.content.width + self.margin.left + self.margin.right
            + self.padding.left + self.padding.right
            + self.border.left + self.border.right
    }
}
