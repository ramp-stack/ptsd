use chrono::{DateTime, Local, Datelike, Timelike, TimeZone};
use prism::{Context, drawable::Drawable};

// #[derive(Clone, Copy, Deserialize, Serialize, Debug)]
// pub struct InternetConnection(pub bool);

/// `Timestamp` contains the date time in an easy-to-read format.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Timestamp(String, String);

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.friendly())
    }
}

impl Timestamp {
    /// Create a `Timestamp` from a local [`DateTime<Local>`].
    ///
    /// Formats as `M/D/YY` for the date and `H:MM AM/PM` for the time.
    pub fn new(dt: DateTime<Local>) -> Self {
        Timestamp(
            dt.format("%-m/%-d/%y").to_string(), 
            dt.format("%-I:%M %p").to_string()
        )
    }

    /// Create a `Timestamp` with date and time set as pending (`"-"`).
    pub fn pending() -> Self {
        Timestamp("-".to_string(), "-".to_string())
    }

    /// Tries to convert the `Timestamp` into a `DateTime<Local>`.
    ///
    /// Parses the stored date and time strings using the format `M/D/YY H:MM AM/PM`.
    pub fn to_datetime(&self) -> Option<DateTime<Local>> {
        let combined = format!("{} {}", self.date(), self.time());
        let format = "%m/%d/%y %I:%M %p";
        let naive = chrono::NaiveDateTime::parse_from_str(&combined, format).expect("Could not parse time");
        Local.from_local_datetime(&naive).single()
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
        let dt = self.to_datetime().expect("Invalid date and time");
        let today = Local::now().date_naive();
        let date = dt.date_naive();
        let hour = dt.hour();
        let minute = dt.minute();
        let (hour12, am_pm) = match hour == 0 {
            true => (12, "am"),
            false if hour < 12 => (hour, "am"),
            false if hour == 12 => (12, "pm"),
            false => (hour - 12, "pm")
        };

        let the_time = format!("{hour12}:{minute:02} {am_pm}");

        match date == today {
            true => the_time,
            false if date == today.pred_opt().unwrap_or(today) => format!("yesterday, {the_time}"),
            false if date.iso_week() == today.iso_week() => format!("{}", dt.format("%A")),
            false if date.year() == today.year() => format!("{}", dt.format("%B %-d")),
            false => format!("{}", dt.format("%m/%d/%y")),
        }
    }

    pub fn date_friendly(&self) -> String {
        let dt = self.to_datetime().expect("Invalid date and time");
        let today = Local::now().date_naive();
        let date = dt.date_naive();

        if date == today {return "Today".to_string();}
        if date.iso_week() == today.iso_week() { return format!("{}", dt.format("%A")); }
        if date.year() == today.year() { return format!("{}", dt.format("%B %-d")); }
        format!("{}", dt.format("%m/%d/%y"))
    }

    /// Returns the date.
    pub fn date(&self) -> String {self.0.clone()}
    /// Returns the time.
    pub fn time(&self) -> String {self.1.clone()}
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


pub trait ValidationFn: FnMut(&Vec<Box<dyn Drawable>>) -> bool + 'static {
    fn clone_box(&self) -> Box<dyn ValidationFn>;
}

impl<F> ValidationFn for F where F: FnMut(&Vec<Box<dyn Drawable>>) -> bool + Clone + 'static {
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