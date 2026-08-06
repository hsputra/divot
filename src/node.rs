//! Node bindings via napi-rs. Exposes a `jsdiff`-shaped API
//! (`diffLines`/`diffWords`/`diffChars`, camelCase via napi-rs's default
//! snake_case-to-camelCase conversion) so migrating from the `diff`
//! package is close to a drop-in swap. Defaults to the Histogram
//! algorithm (measurably faster and more readable than Myers per this
//! project's own benchmark) -- jsdiff itself doesn't expose an algorithm
//! choice, so neither does this binding, to keep the migration surface
//! minimal.

use napi_derive::napi;

use crate::{Algorithm, Change as CoreChange};

#[napi(object)]
pub struct Change {
    pub value: String,
    pub added: bool,
    pub removed: bool,
    pub count: u32,
}

impl From<CoreChange<'_>> for Change {
    fn from(c: CoreChange<'_>) -> Self {
        Change { value: c.value.to_string(), added: c.added, removed: c.removed, count: c.count as u32 }
    }
}

#[napi(js_name = "diffLines")]
pub fn diff_lines(before: String, after: String) -> Vec<Change> {
    crate::line_diff(&before, &after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[napi(js_name = "diffWords")]
pub fn diff_words(before: String, after: String) -> Vec<Change> {
    crate::word_diff(&before, &after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[napi(js_name = "diffChars")]
pub fn diff_chars(before: String, after: String) -> Vec<Change> {
    crate::char_diff(&before, &after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[napi(js_name = "unifiedDiff")]
pub fn unified_diff_js(before: String, after: String) -> String {
    crate::unified_diff(&before, &after, Algorithm::Histogram)
}
