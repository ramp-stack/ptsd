use prism::canvas;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color(canvas::Color);

impl Default for Color {
    fn default() -> Color {Color::from_hex("#fa00d5", 255)}
}

impl Color {
    pub const WHITE: Color = Color(canvas::Color(255, 255, 255, 255));
    pub const BLACK: Color = Color(canvas::Color(0, 0, 0, 255));
    pub const TRANSPARENT: Color = Color(canvas::Color(0, 0, 0, 0));

    pub fn from_hex(color: &str, alpha: u8) -> Self {
        let ce = "Color was not a Hex Value";
        let c = hex::decode(color.strip_prefix('#').unwrap_or(color)).expect(ce);
        Color(canvas::Color(c[0], c[1], c[2], alpha))
    }

    pub fn darken(&self, factor: f32) -> Color {
        let c: canvas::Color = (*self).into();
        Color(canvas::Color((c.0 as f32 * (factor * 1.3)) as u8, (c.1 as f32 * (factor * 1.1)) as u8, (c.2 as f32 * factor) as u8, c.3))
    }

    pub fn is_high_contrast(&self) -> bool {
        let c: canvas::Color = (*self).into();
        0.299*(c.0 as f32) + 0.587*(c.1 as f32) + 0.114*(c.2 as f32) > 128.0
    }

    pub fn contrasted(&self) -> Color {
        let canvas::Color(r, g, b, _) = self.0;
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;

        let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let saturation = r.max(g.max(b)) - r.min(g.min(b));

        match luminance < 0.6 || (luminance < 0.75 && saturation > 0.25) {
            true => Color::WHITE,
            false => Color::BLACK
        }
    }

    pub fn from_canvas(color: canvas::Color) -> Self {
        Color(color)
    }
}

impl From<Color> for canvas::Color {
    fn from(val: Color) -> Self {
        val.0
    }
}