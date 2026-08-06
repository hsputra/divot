const fs = require("fs");
const { diffLines } = require("../index.js");

const corpusDir = process.argv[2];
const nPairs = parseInt(process.argv[3], 10);

const pairs = [];
for (let i = 1; i <= nPairs; i++) {
  const beforePath = `${corpusDir}/${i}.before`;
  const afterPath = `${corpusDir}/${i}.after`;
  if (fs.existsSync(beforePath) && fs.existsSync(afterPath)) {
    pairs.push([fs.readFileSync(beforePath, "utf8"), fs.readFileSync(afterPath, "utf8")]);
  }
}
console.error(`loaded ${pairs.length} pairs`);

// Warm-up (not timed) -- same reasoning as every other benchmark in this
// project: let the JIT/allocator settle before the timed pass.
for (const [before, after] of pairs) {
  diffLines(before, after);
}

let totalChanges = 0;
const t0 = process.hrtime.bigint();
for (const [before, after] of pairs) {
  const result = diffLines(before, after);
  for (const c of result) {
    if (c.added || c.removed) totalChanges += c.count;
  }
}
const t1 = process.hrtime.bigint();
const elapsedMs = Number(t1 - t0) / 1e6;

console.log(`pairs: ${pairs.length}`);
console.log(`total_changed_lines: ${totalChanges}`);
console.log(`total_time_ms: ${elapsedMs.toFixed(3)}`);
console.log(`per_pair_us: ${((elapsedMs * 1000) / pairs.length).toFixed(2)}`);
