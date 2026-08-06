const fs = require("fs");
const { diffLinesMany } = require("../index.js");

const corpusDir = process.argv[2];
const nPairs = parseInt(process.argv[3], 10);

const pairs = [];
for (let i = 1; i <= nPairs; i++) {
  const beforePath = `${corpusDir}/${i}.before`;
  const afterPath = `${corpusDir}/${i}.after`;
  if (fs.existsSync(beforePath) && fs.existsSync(afterPath)) {
    pairs.push({ before: fs.readFileSync(beforePath, "utf8"), after: fs.readFileSync(afterPath, "utf8") });
  }
}
console.error(`loaded ${pairs.length} pairs`);

diffLinesMany(pairs); // warm-up, not timed

const t0 = process.hrtime.bigint();
const results = diffLinesMany(pairs);
const t1 = process.hrtime.bigint();
const elapsedMs = Number(t1 - t0) / 1e6;

let totalChanges = 0;
for (const changes of results) {
  for (const c of changes) if (c.added || c.removed) totalChanges += c.count;
}

console.log(`pairs: ${pairs.length}`);
console.log(`total_changed_lines: ${totalChanges}`);
console.log(`total_time_ms: ${elapsedMs.toFixed(3)}`);
console.log(`per_pair_us: ${((elapsedMs * 1000) / pairs.length).toFixed(2)}`);
