use std::collections::HashMap;
use std::sync::Arc;
use image::RgbaImage;
use include_dir::{DirEntry, include_dir, Dir};
use prism::{canvas, Assets};

use std::fmt;
use std::fmt::Display;

pub use crate::color::Color;
pub use crate::colors::*;

#[derive(Default, Clone, Debug)]
pub struct Theme {
    pub colors: ColorResources,
    pub fonts: FontResources,
    pub icons: IconResources,
}

impl Theme {
    pub fn light(assets: &Dir<'static>, color: Color) -> Self {
        Theme { colors: ColorResources::light(color), icons: IconResources::new(assets), fonts: FontResources::default() }
    }

    pub fn dark(assets: &Dir<'static>, color: Color) -> Self {
        Theme { colors: ColorResources::dark(color), icons: IconResources::new(assets), fonts: FontResources::default() }
    }

    pub fn from(assets: &Dir<'static>, color: Color) -> (Self, bool) {
        let is_dark = color.is_high_contrast();
        (if is_dark {Self::dark(assets, color)} else {Self::light(assets, color)}, is_dark)
    }
}

#[derive(Debug, Clone)]
pub struct FontResources {
    sizes: HashMap<String, f32>,
    fonts: HashMap<String, canvas::Font>,
}

impl FontResources {
    pub fn insert_font<K: Display>(&mut self, key: K, font: canvas::Font) {
        self.fonts.insert(key.to_string(), font);
    }

    pub fn insert_size<K: Display>(&mut self, key: K, size: f32) {
        self.sizes.insert(key.to_string(), size);
    }

    pub fn get_font<K: Display>(&self, key: K) -> Option<&canvas::Font> {
        self.fonts.get(&key.to_string())
    }

    pub fn get_size<K: Display>(&self, key: K) -> f32 {
        self.sizes.get(&key.to_string()).copied().unwrap_or_default()
    }
}

impl Default for FontResources {
    fn default() -> Self {
        let bold = canvas::Font::from_bytes(include_bytes!("../resources/fonts/outfit_bold.ttf")).unwrap();
        let regular = canvas::Font::from_bytes(include_bytes!("../resources/fonts/outfit_regular.ttf")).unwrap();
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

pub enum FontStyle {Heading, Text, Label}
impl fmt::Display for FontStyle { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "FontStyle::{}", match self { 
    FontStyle::Heading => "Heading", FontStyle::Text => "Text", FontStyle::Label => "Label" 
}) } }

#[derive(Debug, Default, Copy, Clone)]
pub enum TextSize {Title, H1, H2, H3, H4, H5, H6, Xl, #[default] Lg, Md, Sm, Xs}
impl fmt::Display for TextSize { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "TextSize::{}", match self { 
    TextSize::Title => "Title", TextSize::H1 => "H1", TextSize::H2 => "H2", TextSize::H3 => "H3", TextSize::H4 => "H4", 
    TextSize::H5 => "H5", TextSize::H6 => "H6", TextSize::Xl => "Xl", TextSize::Lg => "Lg", TextSize::Md => "Md", 
    TextSize::Sm => "Sm", TextSize::Xs => "Xs" 
}) } }


/// - Icons will automatically be added to resources when they meet these conditions:
///     - Icons must be `.svg` files.
///     - Icons must be located in `project/resources/icons/`.
#[derive(Debug, Clone)]
pub struct IconResources(HashMap<String, Arc<RgbaImage>>);
impl IconResources {
    pub fn new(assets: &Dir<'static>) -> Self {
        let mut resources = IconResources::default();
        resources.include(assets);
        resources
    }

    fn include(&mut self, assets: &Dir<'static>) {
        fn walk(map: &mut HashMap<String, Arc<RgbaImage>>, dir: &Dir<'static>) {
            for entry in dir.entries() {
                match entry {
                    DirEntry::File(f)
                        if f.path().extension().and_then(|e| e.to_str()) == Some("svg") =>
                    {
                        let name = f.path().to_str().unwrap();
                        let name = name.strip_prefix("icons/").unwrap_or(name);
                        let name = name.strip_suffix(".svg").unwrap_or(name)
                            .replace(' ', "_");

                        map.insert(name, Arc::new(Assets::load_svg(f.contents())));
                    }
                    DirEntry::Dir(d) => walk(map, d),
                    _ => {}
                }
            }
        }

        walk(&mut self.0, assets);
    }
}
impl Default for IconResources {
    fn default() -> Self {
        let result = include_dir!("resources/icons").entries().iter().filter_map(|e| match e {
            DirEntry::File(f) => Some(f),
            _ => None,
        }).filter(|p| p.path().to_str().unwrap().ends_with(".svg")).collect::<Vec<_>>();


        Self(result.iter().map(|p| {
            let name = p.path().to_str().unwrap().strip_suffix(".svg").unwrap().replace(' ', "_");
            (name, Arc::new(Assets::load_svg(p.contents())))
        }).collect())
    }
}

impl IconResources {
    pub fn get(&self, name: &str) -> Arc<RgbaImage> {
        self.0.get(name).unwrap_or_else(|| {
            println!("Failed to get icon by name {name:?}");
            self.0.get("error").expect("IconResources corrupted.")
        }).clone()
    }

    pub fn all(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
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
        colors.insert(Background::Primary, Color::WHITE);
        colors.insert(Background::Secondary, Color::from_hex("#DDDDDD", 255));
        colors.insert(Text::Primary, Color::BLACK);
        colors.insert(Text::Secondary, Color::from_hex("#9e9e9e", 255));
        colors.insert(Text::Heading, Color::BLACK);
        colors.insert(Outline::Primary, Color::from_hex("#585250", 255));
        colors.insert(Outline::Secondary, Color::from_hex("#9e9e9e", 255));
        colors.insert(Status::Success, Color::from_hex("#3ccb5a", 255));
        colors.insert(Status::Warning, Color::from_hex("#f5bd14", 255));
        colors.insert(Status::Danger, Color::from_hex("#ff330a", 255));
        colors.insert(Brand, brand);
        colors
    }

    pub fn dark(brand: Color) -> Self {
        let mut colors = ColorResources(HashMap::new());
        colors.insert(Background::Primary, Color::BLACK);
        colors.insert(Background::Secondary, Color::from_hex("#262322", 255));
        colors.insert(Text::Primary, Color::WHITE);
        colors.insert(Text::Secondary, Color::from_hex("#a7a29d", 255));
        colors.insert(Text::Heading, Color::WHITE);
        colors.insert(Outline::Primary, Color::from_hex("#585250", 255));
        colors.insert(Outline::Secondary, Color::from_hex("#a7a29d", 255));
        colors.insert(Status::Success, Color::from_hex("#3ccb5a", 255));
        colors.insert(Status::Warning, Color::from_hex("#f5bd14", 255));
        colors.insert(Status::Danger, Color::from_hex("#ff330a", 255));
        colors.insert(Brand, brand);
        colors
    }
}

impl Default for ColorResources { fn default() -> Self { ColorResources::dark(Color::from_hex("#1ca758", 255)) } }
