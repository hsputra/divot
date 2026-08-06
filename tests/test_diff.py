import divot


def reconstruct(changes, side):
    return "".join(c.value for c in changes if not getattr(c, side))


def test_diff_lines_reconstructs_before_and_after():
    before = "abc\ndef\nghi\n"
    after = "abc\nDEF\nghi\n"
    changes = divot.diff_lines(before, after)
    assert reconstruct(changes, "added") == before
    assert reconstruct(changes, "removed") == after


def test_diff_lines_identical_input_is_all_unchanged():
    text = "same\nlines\n"
    changes = divot.diff_lines(text, text)
    assert len(changes) == 1
    assert not changes[0].added and not changes[0].removed
    assert changes[0].value == text


def test_diff_words_reconstructs_and_isolates_changed_word():
    before = "the quick brown fox"
    after = "the slow brown fox"
    changes = divot.diff_words(before, after)
    assert reconstruct(changes, "added") == before
    assert reconstruct(changes, "removed") == after
    assert any(c.removed and c.value == "quick" for c in changes)
    assert any(c.added and c.value == "slow" for c in changes)


def test_diff_chars_handles_multibyte_utf8():
    before = "héllo wörld"
    after = "héllo wArld"
    changes = divot.diff_chars(before, after)
    assert reconstruct(changes, "added") == before
    assert reconstruct(changes, "removed") == after


def test_unified_diff_has_real_hunk_header():
    patch = divot.unified_diff("a\nb\nc\n", "a\nB\nc\n")
    assert "@@" in patch
    assert "-b" in patch
    assert "+B" in patch


def test_diff_lines_many_matches_diff_lines_per_pair():
    pairs = [
        ("a\nb\n", "a\nB\n"),
        ("x\ny\nz\n", "x\ny\nz\n"),
        ("one\ntwo\n", "one\nthree\ntwo\n"),
    ]
    batched = divot.diff_lines_many(pairs)
    assert len(batched) == len(pairs)
    for i, (before, after) in enumerate(pairs):
        expected = divot.diff_lines(before, after)
        assert [(c.value, c.added, c.removed, c.count) for c in batched[i]] == [
            (c.value, c.added, c.removed, c.count) for c in expected
        ]


def test_change_repr_is_readable():
    changes = divot.diff_lines("a\n", "A\n")
    assert repr(changes[0]).startswith("Change(value=")
