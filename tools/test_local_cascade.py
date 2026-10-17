"""Unit tests for the judge-block rule in local_cascade.py (no data or models needed)."""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import local_cascade as lc  # noqa: E402

T = (0.5,)


def split(rows):
    """rows: (label, layer-1 verdict, score, gate ok, judge-blockable)."""
    data = [{"label": r[0]} for r in rows]
    l1 = {i: r[1] for i, r in enumerate(rows)}
    scores = {i: (r[2],) for i, r in enumerate(rows)}
    ok = [r[3] for r in rows]
    scored = {i for i, r in enumerate(rows) if r[4]}
    return data, l1, scores, ok, scored


class BlockTest(unittest.TestCase):
    def test_blocks_count_mismatches_and_low_unsure_rows_only(self):
        d = split([
            ("contradicts", "mismatch", 0.9, True, True),  # layer-1 block
            ("contradicts", "unsure", 0.01, True, True),  # judge block
            ("supports", "unsure", 0.01, True, False),  # figures backed: never judge-blocked
            ("supports", "unsure", 0.9, True, True),  # stated, not blocked
            ("supports", "unsure", 0.2, True, True),  # above the block threshold
        ])
        self.assertEqual(lc.blocks(d, T, None), (1, 1))
        self.assertEqual(lc.blocks(d, T, 0.05), (2, 2))

    def test_fit_needs_the_block_bar_and_twenty_blocks(self):
        rows = [("contradicts", "unsure", 0.001 * k, True, True) for k in range(1, 101)]
        fb = lc.fit_block(split(rows), T)
        self.assertEqual(fb[1:], (100, 100))
        self.assertIsNone(lc.fit_block(split(rows[:19]), T))
        noisy = [("supports" if k % 2 else "contradicts", "unsure", 0.001 * k, True, True) for k in range(1, 101)]
        self.assertIsNone(lc.fit_block(split(noisy), T))


if __name__ == "__main__":
    unittest.main()
