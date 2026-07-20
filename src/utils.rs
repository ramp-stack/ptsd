use chrono::{Datelike, Timelike};
use prism::{Context, drawable::Drawable};
use image::{RgbaImage, load_from_memory};
use include_dir::{DirEntry, Dir, File};

pub use chrono::{DateTime, Local, Utc, Duration};
use std::sync::Arc;

// #[derive(Clone, Copy, Deserialize, Serialize, Debug)]
// pub struct InternetConnection(pub bool);

/// `Timestamp` contains the date time in an easy-to-read format.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Timestamp(Option<DateTime<Utc>>);

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.friendly())
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp::new(Some(Local::now()))
    }
}

impl Timestamp {
    /// Create a `Timestamp` from a local [`DateTime<Local>`].
    pub fn new(dt: Option<DateTime<Local>>) -> Self {Timestamp(dt.map(|s| s.into()))}

    pub fn from_i64(i: i64) -> Self {
        Timestamp::new(Some(DateTime::<Utc>::from_timestamp_nanos(i).with_timezone(&Local)))
    }

    pub fn from_u64(u: u64) -> Self {
        Timestamp::from_i64(u.try_into().unwrap_or_default())
    }

    /// Create a `Timestamp` with date and time set as pending (`"-"`).
    pub fn pending() -> (String, String) {
        ("-".to_string(), "-".to_string())
    }

    /// Tries to convert the `Timestamp` into a `DateTime<Local>`.
    ///
    /// Parses the stored date and time strings using the format `M/D/YY H:MM AM/PM`.
    pub fn as_local(&self) -> Option<DateTime<Local>> {
        self.0.map(|dt| dt.into())
    }

    /// Returns a human-readable, "direct" representation of the timestamp.
    ///
    /// Formats the timestamp based on how recent it is:
    /// - **Today**: `"H:MM am/pm"`
    /// - **Yesterday**: `"yesterday, H:MM am/pm"`
    /// - **Same week**: day of the week (e.g., `"Monday"`)
    /// - **Same year**: `"Month D"` (e.g., `"August 16"`)
    /// - **Otherwise**: `"MM/DD/YY"`
    ///
    /// Returns `None` if the timestamp cannot be converted to a local datetime.
    pub fn friendly(&self) -> String {
        if let Some(dt) = self.as_local() {
            let today = Local::now().date_naive();
            let date = dt.date_naive();
            let hour = dt.hour();
            let minute = dt.minute();
            let (hour12, am_pm) = match hour == 0 {
                true => (12, "AM"),
                false if hour < 12 => (hour, "AM"),
                false if hour == 12 => (12, "PM"),
                false => (hour - 12, "PM")
            };

            let the_time = format!("{hour12}:{minute:02} {am_pm}");

            match date == today {
                true => the_time,
                false if date == today.pred_opt().unwrap_or(today) => format!("yesterday, {the_time}"),
                false if date.iso_week() == today.iso_week() => format!("{}", dt.format("%A")),
                false if date.year() == today.year() => format!("{}", dt.format("%B %-d")),
                false => format!("{}", dt.format("%m/%d/%y")),
            }
        } else {"Pending".to_string()}
    }

    pub fn date(&self) -> String {
        self.as_local().map(|dt| dt.format("%-m/%-d/%y").to_string()).unwrap_or("Pending".to_string())
    }

    pub fn time(&self) -> String {
        self.as_local().map(|dt| dt.format("%-I:%M %p").to_string()).unwrap_or("Pending".to_string())
    }

    pub fn precise(&self) -> String {
        let dt = match self.0 {
            Some(dt) => dt.with_timezone(&Local),
            None => return "Pending".into(),
        };

        let now = Local::now().date_naive();
        let d = dt.date_naive();
        let t = dt.format("%-I:%M %p");

        if d == now {
            t.to_string()
        } else if d == now - Duration::days(1) {
            format!("Yesterday, {}", t)
        } else if d >= now - Duration::days(7) {
            format!("{}, {}", dt.format("%A"), t)
        } else {
            dt.format("%-m/%-d/%y, %-I:%M %p").to_string()
        }
    }
}

// impl From<String> for PelicanError {
//     fn from(s: String, ap: impl AppPage) -> Self {
//         PelicanError::Err(s, ap)
//     }
// }

#[derive(Debug, Clone)]
pub struct TitleSubtitle {
    pub title: String, 
    pub subtitle: Option<String>
}

impl TitleSubtitle {
    pub fn new(title: &str, subtitle: Option<&str>) -> Self {
        TitleSubtitle{
            title: title.to_string(), 
            subtitle: subtitle.map(|s| s.to_string())
        }
    }
}


pub trait ValidationFn: FnMut(&mut Context, Vec<&mut Box<dyn Drawable>>) -> bool + 'static {
    fn clone_box(&self) -> Box<dyn ValidationFn>;
}

impl<F> ValidationFn for F where F: FnMut(&mut Context, Vec<&mut Box<dyn Drawable>>) -> bool + Clone + 'static {
    fn clone_box(&self) -> Box<dyn ValidationFn> { Box::new(self.clone()) }
}

impl Clone for Box<dyn ValidationFn> { fn clone(&self) -> Self { self.as_ref().clone_box() } }

impl std::fmt::Debug for dyn ValidationFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  write!(f, "ValidationFn...") }
}

pub trait Callback: FnMut(&mut Context) + 'static {
    fn clone_box(&self) -> Box<dyn Callback>;
}

impl PartialEq for dyn Callback{fn eq(&self, _: &Self) -> bool {true}}

impl<F> Callback for F where F: FnMut(&mut Context) + Clone + 'static {
    fn clone_box(&self) -> Box<dyn Callback> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Callback> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl std::fmt::Debug for dyn Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clonable Closure")
    }
}
#[derive(Clone, Debug)]
pub struct Assets {
    pub inner: Vec<Dir<'static>>,
}

impl Assets {
    pub fn new(inner: Vec<Dir<'static>>) -> Self {
        println!("Assets:");
        for dir in &inner {
            println!("{:?}", dir);
        }

        Self { inner }
    }

    pub fn all(&self) -> &Vec<Dir<'static>> {
        &self.inner
    }

    pub fn get_image(&self, path: &str) -> Option<Arc<RgbaImage>> {
        let file = self.find_file(path)?;
        let bytes = file.contents().to_vec();

        Some(Arc::new(load_from_memory(&bytes).ok()?.to_rgba8()))
    }

    pub fn get_font(&self, path: &str) -> Option<Vec<u8>> {
        Some(self.find_file(path)?.contents().to_vec())
    }

    pub fn get_svg(&self, path: &str) -> Option<Arc<RgbaImage>> {
        let svg = self.find_file(path)?.contents();

        Some(Arc::new(Self::load_svg(svg)))
    }

    pub fn load_file(&self, file: &str) -> Option<Vec<u8>> {
        Some(self.find_file(file)?.contents().to_vec())
    }

    pub fn load_svg(svg: &[u8]) -> RgbaImage {
        let svg = std::str::from_utf8(svg).unwrap();
        let svg = nsvg::parse_str(svg, nsvg::Units::Pixel, 96.0).unwrap();
        let rgba = svg.rasterize(8.0).unwrap();
        let size = rgba.dimensions();

        RgbaImage::from_raw(size.0, size.1, rgba.into_raw()).unwrap()
    }

    pub fn load_png(&self, file: &str) -> Option<RgbaImage> {
        let bytes = self.load_file(file)?;
        Some(
            image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                .ok()?
                .into_rgba8()
        )
    }

    pub fn get_asset_image(&self, name: &str) -> Option<Arc<RgbaImage>> {
        let png = format!("{name}.png");
        let svg = format!("{name}.svg");

        if let Some(file) = self.find_file(&png) {

            let image = load_from_memory(file.contents())
                .ok()?
                .into_rgba8();

            return Some(Arc::new(image));
        }

        if let Some(file) = self.find_file(&svg) {

            let image = Self::load_svg(file.contents());

            return Some(Arc::new(image));
        }

        None
    }

    fn find_file(&self, name: &str) -> Option<&File<'static>> {
        for dir in &self.inner {
            if let Some(file) = self.find_file_recursive(dir, name) {
                return Some(file);
            }
        }

        None
    }

    fn find_file_recursive<'a>(
        &self,
        dir: &'a Dir<'static>,
        name: &str,
    ) -> Option<&'a File<'static>> {
        for entry in dir.entries().iter().rev() {
            match entry {
                DirEntry::File(file) => {
                    let path = file.path().to_str().unwrap_or("");

                    if path.ends_with(name) {
                        return Some(file);
                    }
                }

                DirEntry::Dir(dir) => {
                    if let Some(file) = self.find_file_recursive(dir, name) {
                        return Some(file);
                    }
                }
            }
        }

        None
    }

    pub fn files(&self) -> Vec<&File<'static>> {
        self.inner
            .iter()
            .flat_map(|dir| dir.files())
            .collect()
    }
}