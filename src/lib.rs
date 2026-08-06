//! Fast diff/patch engine built on [`imara-diff`](https://docs.rs/imara-diff)
//! (Apache-2.0), the diff engine that powers `gitoxide`. This crate adds:
//!
//! - Word- and char-granularity tokenization (`imara-diff` only ships
//!   line tokenization out of the box).
//! - A `jsdiff`-shaped [`Change`] result type (unchanged/added/removed
//!   runs across the *whole* input, not just the changed hunks), so the
//!   npm binding can offer an easy migration path from the incumbent
//!   `diff` package it measurably outperforms.
//!
//! See the [repository README](https://github.com/hsputra/divot) for the
//! real, reproducible benchmark against jsdiff.

mod diff;
mod tokenize;

#[cfg(feature = "node")]
mod node;

pub use diff::{char_diff, line_diff, unified_diff, word_diff, Change};
pub use imara_diff::Algorithm;
