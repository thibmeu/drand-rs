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

impl Format {
    pub fn new(long: bool, json: bool) -> Self {
        match (long, json) {
            (false, false) => Self::Short,
            (true, false) => Self::Long,
            (false, true) => Self::Json,
            (true, true) => unreachable!("long and json format cannot be true together"),
        }
    }
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
