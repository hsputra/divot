# Benchmark charts

Three charts, generated from real measurements against the actual
compiled native addon and the real `diff` npm package (jsdiff v9.0.0) —
not the underlying `imara-diff` library in isolation, and not projected
numbers. Raw data in `data/`, regenerate with:

```sh
pip install matplotlib
python plots/make_charts.py
```

## Charts

- **`jsdiff_comparison.svg`** — per-diff latency, divot vs. jsdiff, log-log
  scatter across 270 real before/after pairs.
- **`jsdiff_speedup.svg`** — per-diff speedup ratio (jsdiff time / divot
  time) across the same pairs, with the mean marked.
- **`batch_scaling.svg`** — `diffLinesMany()` throughput vs. Rayon thread
  count, with jsdiff's single-threaded mean as a reference line. Confirms
  the speedup scales with available cores rather than being a fixed
  number (see main README's Performance section for why that
  distinction matters).

## How the corpus and data were generated

The corpus is 270 real before/after pairs extracted from consecutive
commits of 8 files in jQuery's actual git history (`src/event.js`,
`css.js`, `core.js`, `manipulation.js`, `selector.js`, `ajax.js`,
`effects.js`, `deferred.js`) — not synthetic text. Extraction was a
shallow clone (`git clone --depth 800`) plus a loop over
`git log --follow` per file, diffing each pair of consecutive versions.
The corpus itself isn't checked into this repo (jQuery's source, not
divot's), but the extraction is mechanical and reproducible from any
jQuery clone.

Per-pair timing (`data/percase_jsdiff.csv`, `data/percase_divot_npm.csv`):
each pair's diff is run 20 times back-to-back and averaged, after an
untimed warm-up pass over the full corpus — reduces timer-granularity
noise on individual fast calls without hiding real per-pair variation
(the resulting scatter still shows real spread, including jsdiff's
occasional much-slower outliers on specific inputs).

Batch scaling (`data/batch_scaling.csv`): `diffLinesMany()` called on
all 270 pairs at once, 5 reps averaged, with `RAYON_NUM_THREADS`
explicitly set per data point (1/2/4/8/12/18) to isolate the effect of
core count rather than relying on whatever the default happened to be.

All measurements were taken through the actual compiled `.node` addon
(`npm run build`, release mode) and the actual `diff` npm package — see
the main README's Performance section for why that distinction from
pure-Rust numbers matters.
