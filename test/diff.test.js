const { test } = require("node:test");
const assert = require("node:assert/strict");
const { diffLines, diffWords, diffChars, unifiedDiff } = require("../index.js");

function reconstruct(changes, side) {
  return changes.filter((c) => !c[side]).map((c) => c.value).join("");
}

test("diffLines reconstructs before and after exactly", () => {
  const before = "abc\ndef\nghi\n";
  const after = "abc\nDEF\nghi\n";
  const changes = diffLines(before, after);
  assert.equal(reconstruct(changes, "added"), before);
  assert.equal(reconstruct(changes, "removed"), after);
});

test("diffLines on identical input returns one unchanged run", () => {
  const text = "same\nlines\n";
  const changes = diffLines(text, text);
  assert.equal(changes.length, 1);
  assert.equal(changes[0].added, false);
  assert.equal(changes[0].removed, false);
  assert.equal(changes[0].value, text);
});

test("diffWords reconstructs before and after, isolates the changed word", () => {
  const before = "the quick brown fox";
  const after = "the slow brown fox";
  const changes = diffWords(before, after);
  assert.equal(reconstruct(changes, "added"), before);
  assert.equal(reconstruct(changes, "removed"), after);
  assert.ok(changes.some((c) => c.removed && c.value === "quick"));
  assert.ok(changes.some((c) => c.added && c.value === "slow"));
});

test("diffChars reconstructs before and after, including multibyte UTF-8", () => {
  const before = "héllo wörld";
  const after = "héllo wArld";
  const changes = diffChars(before, after);
  assert.equal(reconstruct(changes, "added"), before);
  assert.equal(reconstruct(changes, "removed"), after);
});

test("unifiedDiff produces a real @@ hunk header and +/- lines", () => {
  const patch = unifiedDiff("a\nb\nc\n", "a\nB\nc\n");
  assert.match(patch, /@@/);
  assert.match(patch, /-b/);
  assert.match(patch, /\+B/);
});
