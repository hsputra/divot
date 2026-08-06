# divot

Fast diff/patch engine, in Rust, with Python and Node bindings.

## Why

[jsdiff](https://github.com/kpdecker/jsdiff) (138M weekly downloads) is
the dominant diff library on npm, and its own docs admit a real
performance compromise: linked-list diagonal tracking instead of
optimized array-based Myers diff. divot's core is built on
[`imara-diff`](https://github.com/pascalkuthe/imara-diff) — the diff
engine that powers `gitoxide` in production — which doesn't make that
trade-off, and additionally implements the Histogram algorithm (git's
modern default: faster *and* more human-readable for code than plain
Myers).

## Performance

Real, measured, reproducible — not projected. 270 before/after pairs
extracted from jQuery's actual git history (`src/event.js`, `css.js`,
`core.js`, `manipulation.js`, `selector.js`, `ajax.js`, `effects.js`,
`deferred.js`), diffed with the real `diff` npm package (jsdiff v9.0.0)
and with divot, same corpus, both warmed up first.

| | Time/pair | vs. jsdiff |
|---|---|---|
| jsdiff v9.0.0 (`diffLines`) | 118.96µs | — |
| **divot, via the real npm binding** | **29.9µs** | **~4.0x** |
| divot, pure Rust (no FFI) | ~19.8µs | ~5.9x |

**~4.0x is the number that matters** — it's what `npm install divot`
actually delivers, measured through the real compiled native addon, not
the underlying Rust library in isolation. The pure-Rust figure is shown
for transparency about where FFI (UTF-16→UTF-8 conversion in, JS object
construction out) costs real time, not to imply it's what you'll
observe from JS.

### Batch diffing: a real capability jsdiff can't offer

`diffLinesMany()` diffs many pairs at once across all available CPU
cores (via Rayon internally) — not an algorithm improvement over
`imara-diff`, a different capability entirely: jsdiff is single-threaded
with no built-in batch API, so this isn't something it can match by
being faster, only by the caller manually parallelizing across worker
threads/processes themselves.

| | Time/pair | vs. jsdiff (looped) |
|---|---|---|
| jsdiff v9.0.0, called in a loop | ~118.4µs | — |
| **`diffLinesMany`, real npm binding, 18 cores** | **~10.5µs** | **~11x** |

Verified this is genuinely about parallelism and not some batching
trick: forcing a single Rayon thread reproduces the non-batched ~19.8µs
pure-Rust number exactly. **The multiplier scales with available cores**
— it's a batch-throughput number, not a claim that any single diff got
11x faster. Useful for realistic bulk workloads (lint a whole PR, diff
every file in a directory), less useful for a one-off single diff, where
`diffLines()` is the right call.

## Status

Early. Implemented and tested: line/word/char diffing, unified diff
output, npm binding (`diffLines`/`diffWords`/`diffChars`/`unifiedDiff`/
`diffLinesMany`, real native addon, real tests via Node's built-in test
runner). No CLI yet.

Not yet implemented: Python bindings (PyO3), a fuzzy/atomic
patch-application layer (the planned differentiator for LLM-coding-agent
workflows — see the project's technical spec for detail on the failure
modes it targets). Nothing here is published to any registry yet.

## Credit

Built on [`imara-diff`](https://github.com/pascalkuthe/imara-diff)
(Apache-2.0) by Pascal Kuthe — the diff computation itself is that
crate's work; this project adds word/char tokenization (not provided
upstream), a `jsdiff`-compatible result shape, and (planned) the
patch-application layer.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
