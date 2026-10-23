use chrono::{Datelike, Timelike};
pub use chrono::{DateTime, Local, Utc, Duration};
use prism::{Context, drawable::Drawable};
use prism::drawable::Svg;
use prism::hardware::Image;
use include_dir::{DirEntry, Dir, File};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::Validate;

/// `Timestamp` contains the date time in an easy-to-read format.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Timestamp(Option<DateTime<Utc>>);

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.friendly())
    }
}

impl Default for Timestamp {
    fn default() -> Self {Timestamp::new(Some(Local::now()))}
}

impl Timestamp {
    /// Create a `Timestamp` from a local [`DateTime<Local>`].
    pub fn new(dt: Option<DateTime<Local>>) -> Self {Timestamp(dt.map(|s| s.into()))}

    pub fn from_i64(i: i64) -> Self {
        Timestamp::new(Some(DateTime::<Utc>::from_timestamp_nanos(i).with_timezone(&Local)))
    }

    pub fn from_u64(u: u64) -> Self {Timestamp::from_i64(u.try_into().unwrap())}

    /// Create a `Timestamp` with date and time set as pending (`"-"`).
    pub fn pending() -> (String, String) {("-".to_string(), "-".to_string())}

    /// Tries to convert the `Timestamp` into a `DateTime<Local>`.
    ///
    /// Parses the stored date and time strings using the format `M/D/YY H:MM AM/PM`.
    pub fn as_local(&self) -> Option<DateTime<Local>> {self.0.map(|dt| dt.into())}

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

pub trait ValidationFn: for<'a> Validate<Vec<&'a mut Box<dyn Drawable>>, bool> {}
impl<V: for<'a> Validate<Vec<&'a mut Box<dyn Drawable>>, bool>> ValidationFn for V {}
dyn_clone::clone_trait_object!(ValidationFn);

impl std::fmt::Debug for dyn ValidationFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  write!(f, "ValidationFn...") }
}

pub trait Callback: FnMut(&mut Context) + 'static {
    fn clone_box(&self) -> Box<dyn Callback>;
}

impl PartialEq for dyn Callback{fn eq(&self, _: &Self) -> bool {true}}

impl<F: FnMut(&mut Context) + Clone + 'static> Callback for F {
    fn clone_box(&self) -> Box<dyn Callback> {Box::new(self.clone())}
}

impl Clone for Box<dyn Callback> {
    fn clone(&self) -> Self {self.as_ref().clone_box()}
}

impl std::fmt::Debug for dyn Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clonable Closure")
    }
}
#[derive(Clone, Debug)]
pub struct Assets {
    pub inner: Vec<Dir<'static>>,
    images: Arc<Mutex<HashMap<String, Image>>>,
    svgs: Arc<Mutex<HashMap<String, Svg>>>,
}
impl Assets {
    pub fn new(inner: Vec<Dir<'static>>) -> Self {
        Self {
            inner, images: Arc::new(Mutex::new(HashMap::new())),
            svgs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn all(&self) -> &Vec<Dir<'static>> {&self.inner}
    pub fn get_image(&self, path: &str) -> Option<Image> {
        let file = self.find_file(path)?;
        let key = file.path().to_string_lossy().into_owned();
        Some(self.images.lock().unwrap().entry(key).or_insert_with(|| {
            Image::read(file.contents()).expect("Invalid image asset")
        }).clone())
    }
    pub fn get_font(&self, path: &str) -> Option<Vec<u8>> {
        Some(self.find_file(path)?.contents().to_vec())
    }
    pub fn get_svg(&self, path: &str) -> Option<Svg> {
        let file = self.find_file(path)?;
        let key = file.path().to_string_lossy().into_owned();
        Some(self.svgs.lock().unwrap().entry(key).or_insert_with(|| {
            Svg::read(file.contents()).expect("Invalid SVG asset")
        }).clone())
    }
    pub fn load_file(&self, path: &str) -> Option<Vec<u8>> {
        Some(self.find_file(path)?.contents().to_vec())
    }
    pub fn get_asset_image(&self, name: &str) -> Option<Image> {
        let file = self.files().into_iter().find(|file| {
            file.path().with_extension("").ends_with(name)
                && file.path().extension().is_some_and(|ext| matches!(ext.to_str(),
                    Some("png" | "jpg" | "jpeg")))
        })?;
        self.get_image(&file.path().to_string_lossy())
    }
    pub fn get_asset_svg(&self, name: &str) -> Option<Svg> {
        let file = self.files().into_iter().find(|file| {
            file.path().with_extension("").ends_with(name)
                && file.path().extension().is_some_and(|ext| ext == "svg")
        })?;
        self.get_svg(&file.path().to_string_lossy())
    }
    pub fn files(&self) -> Vec<&File<'static>> {
        self.inner.iter().flat_map(Self::directory_files).collect()
    }

    fn find_file(&self, name: &str) -> Option<&File<'static>> {
        self.files().into_iter().find(|file| {
            file.path() == std::path::Path::new(name)
                || file.path().file_name().is_some_and(|file| file == name)
                || file.path().ends_with(name)
        })
    }
    fn directory_files<'a>(dir: &'a Dir<'static>) -> Vec<&'a File<'static>> {
        dir.entries().iter().flat_map(|entry| match entry {
            DirEntry::File(file) => vec![file],
            DirEntry::Dir(dir) => Self::directory_files(dir),
        }).collect()
    }
}
