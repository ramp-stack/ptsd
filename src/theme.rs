use prism::drawable::Svg;
use include_dir::include_dir;
use prism::drawable::text::Font;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use crate::utils::Assets;
pub use crate::color::Color;
pub use crate::colors::*;

#[derive(Debug)]
pub enum FontStyle {Heading, Text, Label}
impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "FontStyle::{self:?}")}
}

#[derive(Debug, Default, Copy, Clone)]
pub enum TextSize {Title, H1, H2, H3, H4, H5, H6, Xl, #[default] Lg, Md, Sm, Xs}
impl fmt::Display for TextSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "TextSize::{self:?}")}
}

#[derive(Default, Clone, Debug)]
pub struct Theme {
    pub colors: ColorResources,
    pub fonts: FontResources,
    pub icons: IconResources,
}

impl Theme {
    pub fn light(assets: &Assets, color: Color) -> Self {
        Self {
            colors: ColorResources::light(color), icons: IconResources::new(assets),
            fonts: FontResources::default(),
        }
    }

    pub fn dark(assets: &Assets, color: Color) -> Self {
        Self {
            colors: ColorResources::dark(color), icons: IconResources::new(assets),
            fonts: FontResources::default(),
        }
    }

    pub fn from(assets: &Assets, color: Color) -> (Self, bool) {
        let is_dark = color.is_high_contrast();
        (if is_dark {Self::dark(assets, color)} else {Self::light(assets, color)}, is_dark)
    }
}

#[derive(Debug, Clone)]
pub struct FontResources {
    sizes: HashMap<String, f32>,
    fonts: HashMap<String, Font>,
}

impl FontResources {
    pub fn insert_font<K: Display>(&mut self, key: K, font: Font) {
        self.fonts.insert(key.to_string(), font);
    }

    pub fn insert_size<K: Display>(&mut self, key: K, size: f32) {
        self.sizes.insert(key.to_string(), size);
    }

    pub fn get_font<K: Display>(&self, key: K) -> Option<&Font> {self.fonts.get(&key.to_string())}

    pub fn get_size<K: Display>(&self, key: K) -> f32 {*self.sizes.get(&key.to_string()).unwrap()}
}

impl Default for FontResources {
    fn default() -> Self {
        let bold = Font::from_bytes(include_bytes!("../resources/fonts/outfit_bold.ttf")).unwrap();
        let regular = Font::from_bytes(include_bytes!("../resources/fonts/outfit_regular.ttf")).unwrap();
        let mut resources = FontResources {fonts: HashMap::new(), sizes: HashMap::new()};
        resources.insert_font(FontStyle::Heading, bold.clone());
        resources.insert_font(FontStyle::Text, regular);
        resources.insert_font(FontStyle::Label, bold);
        resources.insert_size(TextSize::Title, 72.0);
        resources.insert_size(TextSize::H1, 48.0);
        resources.insert_size(TextSize::H2, 32.0);
        resources.insert_size(TextSize::H3, 24.0);
        resources.insert_size(TextSize::H4, 20.0);
        resources.insert_size(TextSize::H5, 16.0);
        resources.insert_size(TextSize::H6, 14.0);
        resources.insert_size(TextSize::Xl, 24.0);
        resources.insert_size(TextSize::Lg, 20.0);
        resources.insert_size(TextSize::Md, 16.0);
        resources.insert_size(TextSize::Sm, 14.0);
        resources.insert_size(TextSize::Xs, 12.0);
        resources
    }
}

#[derive(Debug, Clone)]
pub struct IconResources {
    stored: HashMap<String, Svg>,
    public: HashMap<String, Svg>,
}
impl IconResources {
    pub fn new(assets: &Assets) -> Self {
        let mut resources = Self {stored: HashMap::new(), public: HashMap::new()};
        resources.include(assets);
        for (name, image) in Self::default().stored {resources.stored.entry(name).or_insert(image);}
        resources
    }
    pub fn insert<K: Display>(&mut self, key: K, icon: &str) {
        self.public.insert(key.to_string(), self.stored.get(icon).unwrap_or_else(|| {
            self.stored.get("ptsd_error").expect("IconResources corrupted.")
        }).clone());
    }
    pub fn get<K: Display>(&self, key: K) -> Svg {
        self.public.get(&key.to_string()).unwrap_or_else(|| {
            self.stored.get("ptsd_error").expect("IconResources corrupted.")
        }).clone()
    }
    fn include(&mut self, assets: &Assets) {
        for file in assets.files() {
            if file.path().extension().is_none_or(|ext| ext != "svg") {continue}
            let path = file.path().to_str().unwrap();
            let name = path.strip_prefix("icons/").unwrap_or(path)
                .strip_suffix(".svg").unwrap().replace(' ', "_");
            self.stored.entry(name).or_insert_with(|| assets.get_svg(path).unwrap());
        }
    }
}
impl Default for IconResources {
    fn default() -> Self {
        let mut resources = Self {stored: HashMap::new(), public: HashMap::new()};
        resources.include(&Assets::new(vec![include_dir!("resources/icons")]));
        resources
    }
}

#[derive(Debug, Clone)]
pub struct ColorResources(HashMap<String, Color>);

impl ColorResources {
    pub fn insert<K: Display>(&mut self, key: K, color: Color) {
        self.0.insert(key.to_string(), color);
    }

    pub fn get<K: Display>(&self, key: K) -> Color {
        self.0.get(&key.to_string()).cloned().unwrap_or_default()
    }

    pub fn light(brand: Color) -> Self {
        let mut colors = ColorResources(HashMap::new());
        colors.insert(Background::Primary, Color::white());
        colors.insert(Background::Secondary, Color::from_hex("#DDDDDD", 255));
        colors.insert(Text::Primary, Color::black());
        colors.insert(Text::Secondary, Color::from_hex("#585250", 255));
        colors.insert(Text::Heading, Color::black());
        colors.insert(Outline::Primary, Color::from_hex("#DDDDDD", 255));
        colors.insert(Outline::Secondary, Color::from_hex("#585250", 255));
        colors.insert(Status::Success, Color::from_hex("#3ccb5a", 255));
        colors.insert(Status::Warning, Color::from_hex("#f5bd14", 255));
        colors.insert(Status::Danger, Color::from_hex("#ff330a", 255));
        colors.insert(Brand, brand);
        colors
    }

    pub fn dark(brand: Color) -> Self {
        let mut colors = ColorResources(HashMap::new());
        colors.insert(Background::Primary, Color::black());
        colors.insert(Background::Secondary, Color::from_hex("#262322", 255));
        colors.insert(Text::Primary, Color::white());
        colors.insert(Text::Secondary, Color::from_hex("#a7a29d", 255));
        colors.insert(Text::Heading, Color::white());
        colors.insert(Outline::Primary, Color::white());
        colors.insert(Outline::Secondary, Color::from_hex("#a7a29d", 255));
        colors.insert(Status::Success, Color::from_hex("#3ccb5a", 255));
        colors.insert(Status::Warning, Color::from_hex("#f5bd14", 255));
        colors.insert(Status::Danger, Color::from_hex("#ff330a", 255));
        colors.insert(Brand, brand);
        colors
    }
}

impl Default for ColorResources {
    fn default() -> Self {Self::dark(Color::from_hex("#1ca758", 255))}
}
