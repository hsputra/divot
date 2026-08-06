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
`deferred.js`), diffed with both divot's actual public API and the real
`diff` npm package (jsdiff v9.0.0), same corpus, both warmed up first.

| Algorithm | divot | jsdiff v9.0.0 | Speedup |
|---|---|---|---|
| Histogram (default) | ~19.8µs/pair | 117.5µs/pair | **~5.9x** |
| Myers | ~20.4µs/pair | 117.5µs/pair | **~5.8x** |

This is the number divot's actual public API delivers, including result
construction — not a raw-library-only figure with wrapping overhead
quietly excluded.

## Status

Early. Implemented and tested: line/word/char diffing, unified diff
output — the core Rust crate only, no language bindings yet, no CLI.

Not yet implemented: Python bindings (PyO3), Node bindings (napi-rs), a
fuzzy/atomic patch-application layer (the planned differentiator for
LLM-coding-agent workflows — see the project's technical spec for detail
on the failure modes it targets). Nothing here is published to any
registry yet.

## Credit

Built on [`imara-diff`](https://github.com/pascalkuthe/imara-diff)
(Apache-2.0) by Pascal Kuthe — the diff computation itself is that
crate's work; this project adds word/char tokenization (not provided
upstream), a `jsdiff`-compatible result shape, and (planned) the
patch-application layer.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
