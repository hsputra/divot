//! Python bindings via PyO3. Mirrors the shape of the Node binding
//! (`diff_lines`/`diff_words`/`diff_chars`/`unified_diff`, snake_case
//! per Python convention rather than napi-rs's camelCase), defaulting
//! to the Histogram algorithm for the same reason as the Node side.

use pyo3::prelude::*;

use crate::{Algorithm, Change as CoreChange};

#[pyclass(get_all)]
struct Change {
    value: String,
    added: bool,
    removed: bool,
    count: usize,
}

#[pymethods]
impl Change {
    fn __repr__(&self) -> String {
        format!(
            "Change(value={:?}, added={}, removed={}, count={})",
            self.value, self.added, self.removed, self.count
        )
    }
}

impl From<CoreChange<'_>> for Change {
    fn from(c: CoreChange<'_>) -> Self {
        Change { value: c.value.to_string(), added: c.added, removed: c.removed, count: c.count }
    }
}

#[pyfunction]
fn diff_lines(before: &str, after: &str) -> Vec<Change> {
    crate::line_diff(before, after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[pyfunction]
fn diff_words(before: &str, after: &str) -> Vec<Change> {
    crate::word_diff(before, after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[pyfunction]
fn diff_chars(before: &str, after: &str) -> Vec<Change> {
    crate::char_diff(before, after, Algorithm::Histogram).into_iter().map(Change::from).collect()
}

#[pyfunction]
fn unified_diff(before: &str, after: &str) -> String {
    crate::unified_diff(before, after, Algorithm::Histogram)
}

/// Diffs many `(before, after)` pairs at once across a Rayon thread
/// pool -- releases the GIL for the duration, same reasoning as the
/// Node binding's `diffLinesMany`: a real capability difflib-adjacent
/// Python tooling doesn't offer built in.
#[pyfunction]
fn diff_lines_many(py: Python<'_>, pairs: Vec<(String, String)>) -> Vec<Vec<Change>> {
    py.detach(|| {
        let pair_refs: Vec<(&str, &str)> =
            pairs.iter().map(|(b, a)| (b.as_str(), a.as_str())).collect();
        crate::line_diff_many(&pair_refs, Algorithm::Histogram)
            .into_iter()
            .map(|changes| changes.into_iter().map(Change::from).collect())
            .collect()
    })
}

#[pymodule]
fn divot(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Change>()?;
    m.add_function(wrap_pyfunction!(diff_lines, m)?)?;
    m.add_function(wrap_pyfunction!(diff_words, m)?)?;
    m.add_function(wrap_pyfunction!(diff_chars, m)?)?;
    m.add_function(wrap_pyfunction!(unified_diff, m)?)?;
    m.add_function(wrap_pyfunction!(diff_lines_many, m)?)?;
    Ok(())
}
