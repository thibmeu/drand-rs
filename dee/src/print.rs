use anyhow::Result;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    /// Text based format with a single result
    Short,
    /// Text based format with colors and font weight
    Long,
    /// Raw and minified JSON
    Json,
}

pub trait Print {
    fn short(&self) -> Result<String>;
    fn long(&self) -> Result<String>;
    fn json(&self) -> Result<String>;
}

pub fn print_with_format<T: Print>(t: T, format: Format) -> Result<String> {
    match format {
        Format::Short => t.short(),
        Format::Long => t.long(),
        Format::Json => t.json(),
    }
}
