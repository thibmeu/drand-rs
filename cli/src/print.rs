use anyhow::Result;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    /// Text based format with colors and font weight
    Pretty,
    /// Raw and minified JSON
    Json,
}

pub trait Print {
    fn pretty(&self) -> Result<String>;
    fn json(&self) -> Result<String>;
}

pub fn print_with_format<T: Print>(t: T, format: Format) -> Result<String> {
    match format {
        Format::Pretty => t.pretty(),
        Format::Json => t.json(),
    }
}
