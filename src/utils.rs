use itertools::Itertools;
use color_eyre::eyre::{Result, eyre};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::time::Duration;
use std::path::Path;

pub(crate) fn with_progress<T, F: FnMut() -> T>(msg: &'static str, mut f: F) -> T {
    let bar = indicatif::ProgressBar::new_spinner();
    bar.set_message(msg);
    bar.enable_steady_tick(Duration::from_millis(16));

    let ret = f();

    bar.finish_and_clear();

    print!("\r");

    ret
}

pub(crate) fn px(inches: f32) -> String {
    format!("{}px", (inches * 48.0).round())
}

pub(crate) fn sanitize_output_filename(filename: &str) -> Result<String> {
    let basename = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| eyre!("Output filename has no valid basename"))?;
    let sanitized = sanitize_filename::sanitize(basename);

    if sanitized.is_empty() {
        return Err(eyre!("Output filename is empty after sanitization"));
    }

    Ok(sanitized)
}

pub(crate) struct AttributeSet(HashMap<&'static str, String>);

impl AttributeSet {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn set(&mut self, attribute: &'static str, value: String) {
        self.0.insert(attribute, value);
    }
}

impl Display for AttributeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
                .map(|(attr, value)| attr.to_string() + "=\"" + value + "\"")
                .join(" ")
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StyleSet(HashMap<&'static str, String>);

impl StyleSet {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn set(&mut self, prop: &'static str, value: String) {
        self.0.insert(prop, value);
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0)
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Display for StyleSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
                .map(|(attr, value)| attr.to_string() + ": " + value + ";")
                .join(" ")
        )
    }
}
