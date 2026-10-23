use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Background {Primary, Secondary}
impl Display for Background {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "Background::{self:?}")}
}

#[derive(Debug)]
pub enum Text {Primary, Secondary, Heading}
impl Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "Text::{self:?}")}
}

#[derive(Debug)]
pub enum Outline {Primary, Secondary}
impl Display for Outline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "Outline::{self:?}")}
}

#[derive(Debug)]
pub enum Status {Success, Warning, Danger}
impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {write!(f, "Status::{self:?}")}
}

pub struct Brand;
impl Display for Brand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {f.write_str("Brand")}
}
