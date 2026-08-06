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

#[napi(object)]
pub struct DiffPair {
    pub before: String,
    pub after: String,
}

/// Diffs many pairs at once across a Rayon thread pool -- a real
/// capability jsdiff can't offer (single-threaded, no built-in batch
/// API), not just a convenience wrapper. Synchronous in v1: blocks the
/// JS event loop for the (much shorter, parallelized) duration of the
/// batch; a properly async version (releasing the event loop via napi's
/// task API) is natural follow-up work, not done here.
#[napi(js_name = "diffLinesMany")]
pub fn diff_lines_many(pairs: Vec<DiffPair>) -> Vec<Vec<Change>> {
    let pair_refs: Vec<(&str, &str)> = pairs.iter().map(|p| (p.before.as_str(), p.after.as_str())).collect();
    crate::line_diff_many(&pair_refs, Algorithm::Histogram)
        .into_iter()
        .map(|changes| changes.into_iter().map(Change::from).collect())
        .collect()
}
