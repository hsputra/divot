"""
Regenerates the benchmark charts in this directory from the raw data in
plots/data/. That data comes from real runs against the compiled native
addon (percase_divot_npm.csv, batch_scaling.csv) and the real jsdiff
v9.0.0 npm package (percase_jsdiff.csv) on the same 270-pair corpus --
see the "How the corpus and data were generated" section in
plots/README.md for the exact commands.

Usage:
    pip install matplotlib
    python plots/make_charts.py
"""

import csv
import os

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

# Validated categorical palette (Claude Code's "dataviz" skill, light
# mode, slots 1/2/3 -- passes the skill's CVD/contrast validator for
# this 3-series case).
BLUE = "#2a78d6"     # jsdiff
ORANGE = "#eb6834"   # divot (npm, single-call)
AQUA = "#1baf7a"     # divot (npm, batch/parallel)
TEXT_PRIMARY = "#0b0b0b"
TEXT_SECONDARY = "#52514e"
SURFACE = "#fcfcfb"
GRID = "#e3e2dc"

plt.rcParams.update({
    "font.family": "sans-serif",
    "font.size": 11,
    "text.color": TEXT_PRIMARY,
    "axes.edgecolor": GRID,
    "axes.labelcolor": TEXT_PRIMARY,
    "xtick.color": TEXT_SECONDARY,
    "ytick.color": TEXT_SECONDARY,
    "axes.facecolor": SURFACE,
    "figure.facecolor": SURFACE,
    "savefig.facecolor": SURFACE,
})


def read_csv(path):
    with open(path) as f:
        rows = list(csv.DictReader(f))
    return [int(r["before_len"]) for r in rows], [float(r["time_us"]) for r in rows]


jsdiff_size, jsdiff_time = read_csv(os.path.join(DATA, "percase_jsdiff.csv"))
divot_size, divot_time = read_csv(os.path.join(DATA, "percase_divot_npm.csv"))

# ---- Chart 1: per-diff latency, divot vs jsdiff (log-log scatter) ----
fig, ax = plt.subplots(figsize=(7.5, 5), dpi=150)
ax.scatter(jsdiff_size, jsdiff_time, s=14, color=BLUE, alpha=0.65, edgecolors="none", label="jsdiff v9.0.0")
ax.scatter(divot_size, divot_time, s=14, color=ORANGE, alpha=0.75, edgecolors="none", label="divot (real npm binding)")
ax.set_xscale("log")
ax.set_yscale("log")
ax.set_xlabel("Diff input size (bytes, before-side)")
ax.set_ylabel("Time per diff (µs)")
ax.set_title("divot vs. jsdiff: per-diff latency\n270 real before/after pairs, jQuery git history", fontsize=12, color=TEXT_PRIMARY)
ax.grid(True, which="both", linewidth=0.5, color=GRID)
ax.spines[["top", "right"]].set_visible(False)
legend = ax.legend(frameon=False, loc="upper left", fontsize=10)
for text in legend.get_texts():
    text.set_color(TEXT_PRIMARY)
fig.text(0.01, -0.02, "Both warmed up first. Same corpus, same machine. github.com/hsputra/divot", fontsize=8, color=TEXT_SECONDARY)
fig.tight_layout()
fig.savefig(os.path.join(HERE, "jsdiff_comparison.svg"), bbox_inches="tight")
fig.savefig(os.path.join(HERE, "jsdiff_comparison.png"), bbox_inches="tight")
plt.close(fig)

# ---- Chart 2: per-pair speedup ratio vs size ----
# Matched by index -- same 270 pairs, same order, in both CSVs.
speedups = [j / d for j, d in zip(jsdiff_time, divot_time)]
fig, ax = plt.subplots(figsize=(7.5, 5), dpi=150)
ax.scatter(jsdiff_size, speedups, s=14, color=ORANGE, alpha=0.7, edgecolors="none")
mean_speedup = sum(speedups) / len(speedups)
ax.axhline(mean_speedup, color=TEXT_SECONDARY, linewidth=1, linestyle="--")
ax.text(max(jsdiff_size) * 0.98, mean_speedup * 1.05, f"mean {mean_speedup:.1f}x", fontsize=9, color=TEXT_SECONDARY, ha="right")
ax.set_xscale("log")
ax.set_xlabel("Diff input size (bytes, before-side)")
ax.set_ylabel("Speedup (jsdiff time / divot time)")
ax.set_title("divot speedup vs. jsdiff, per diff\n270 real before/after pairs, jQuery git history", fontsize=12, color=TEXT_PRIMARY)
ax.grid(True, which="both", linewidth=0.5, color=GRID)
ax.spines[["top", "right"]].set_visible(False)
ax.set_ylim(bottom=0)
fig.tight_layout()
fig.savefig(os.path.join(HERE, "jsdiff_speedup.svg"), bbox_inches="tight")
fig.savefig(os.path.join(HERE, "jsdiff_speedup.png"), bbox_inches="tight")
plt.close(fig)

# ---- Chart 3: batch/parallel scaling ----
with open(os.path.join(DATA, "batch_scaling.csv")) as f:
    rows = list(csv.DictReader(f))
threads = [int(r["threads"]) for r in rows]
batch_time = [float(r["per_pair_us"]) for r in rows]
jsdiff_mean = sum(jsdiff_time) / len(jsdiff_time)

fig, ax = plt.subplots(figsize=(7.5, 5), dpi=150)
ax.plot(threads, batch_time, color=AQUA, linewidth=2, marker="o", markersize=5, label="divot diffLinesMany (real npm binding)")
ax.axhline(jsdiff_mean, color=BLUE, linewidth=1.5, linestyle="--", label=f"jsdiff v9.0.0, single-threaded (mean {jsdiff_mean:.0f}µs)")
ax.set_xlabel("Rayon threads used")
ax.set_ylabel("Time per diff, batched (µs)")
ax.set_title("Batch/parallel diffing: divot vs. jsdiff\n270 real pairs diffed in one diffLinesMany() call", fontsize=12, color=TEXT_PRIMARY)
ax.grid(True, linewidth=0.5, color=GRID)
ax.spines[["top", "right"]].set_visible(False)
ax.xaxis.set_major_locator(mticker.FixedLocator(threads))
ax.set_ylim(top=jsdiff_mean * 1.32)
legend = ax.legend(frameon=True, loc="center right", fontsize=10)
legend.get_frame().set_facecolor(SURFACE)
legend.get_frame().set_edgecolor(GRID)
for text in legend.get_texts():
    text.set_color(TEXT_PRIMARY)
for x, y in zip(threads, batch_time):
    speedup = jsdiff_mean / y
    ax.annotate(f"{speedup:.1f}x", (x, y), textcoords="offset points", xytext=(0, 10), fontsize=8, color=TEXT_SECONDARY, ha="center")
fig.text(0.01, -0.02, "18-core machine. Same 270-pair corpus as above. github.com/hsputra/divot", fontsize=8, color=TEXT_SECONDARY)
fig.tight_layout()
fig.savefig(os.path.join(HERE, "batch_scaling.svg"), bbox_inches="tight")
fig.savefig(os.path.join(HERE, "batch_scaling.png"), bbox_inches="tight")
plt.close(fig)

print("done. mean single-call speedup:", round(mean_speedup, 2), "| jsdiff mean:", round(jsdiff_mean, 1), "us")
