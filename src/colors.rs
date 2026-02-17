use std::fmt::{self, Display};

pub enum Background {Primary, Secondary}

impl Display for Background { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self {
    Background::Primary => write!(f, "Background::Primary"),
    Background::Secondary => write!(f, "Background::Secondary"),
} } }

pub enum Text {Primary, Secondary, Heading}

impl Display for Text { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self {
    Text::Primary => write!(f, "Text::Primary"),
    Text::Secondary => write!(f, "Text::Secondary"),
    Text::Heading => write!(f, "Text::Heading"),
} } }

pub enum Outline {Primary, Secondary}

impl Display for Outline { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self {
    Outline::Primary => write!(f, "Outline::Primary"),
    Outline::Secondary => write!(f, "Outline::Secondary"),
} } }

pub enum Status {Success, Warning, Danger}

impl Display for Status { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self {
    Status::Success => write!(f, "Status::Success"),
    Status::Warning => write!(f, "Status::Warning"),
    Status::Danger  => write!(f, "Status::Danger"),
} } }

pub struct Brand;
impl Display for Brand {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "Brand")}}