import json
import math
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import pr_table

HERE = Path(__file__).resolve().parent


def stats(mean, ci_pct, sd_pct, n):
    return {"mean": mean, "ci_half": mean * ci_pct / 100, "sd": mean * sd_pct / 100, "n": n}


class FormatTest(unittest.TestCase):
    """Rows must match the maintainer's layout character for character (wild#2621)."""

    def test_header_matches(self):
        self.assertEqual(
            pr_table.HEADER,
            "  metric                       mean        CI ±         SD       n   delta vs #1 (95% CI)")

    def test_time_row_matches(self):
        line = pr_table.format_row("wall", "ms", stats(81.263, 0.09, 1.87, 1771), (-6.60, -6.71, -6.49))
        self.assertEqual(
            line, "  wall                    81.263 ms       0.09%      1.87%    1771   -6.60% [-6.71%, -6.49%]")

    def test_counter_row_matches(self):
        line = pr_table.format_row("instructions", "count", stats(89.521e9, 1.51, 10.81, 200),
                                   (-26.73, -28.60, -24.87))
        self.assertEqual(
            line,
            "  instructions            89.521 G~       1.51%     10.81%     200   -26.73% [-28.60%, -24.87%]")

    def test_rss_row_matches(self):
        line = pr_table.format_row("peak RSS", "kib", stats(24874.56 * 1024, 0.0, 0.03, 200),
                                   (0.0, -0.001, 0.01))
        self.assertEqual(
            line, "  peak RSS             24874.56 MiB       0.00%      0.03%     200   +0.00% [-0.00%, +0.01%]")

    def test_si_prefixes(self):
        self.assertEqual(pr_table.format_mean(18.96e6, "count"), "18.960 M~")
        self.assertEqual(pr_table.format_mean(1500, "count"), "1.500 K~")
        self.assertEqual(pr_table.format_mean(12, "count"), "12.000 ~")


class StatisticsTest(unittest.TestCase):
    def test_t_quantiles(self):
        for df, expected in ((1, 12.706), (2, 4.303), (3, 3.182), (5, 2.571), (10, 2.228),
                             (30, 2.042), (1000, 1.962)):
            self.assertAlmostEqual(pr_table.t_quantile_975(df), expected, delta=2e-3, msg=df)

    def test_describe(self):
        d = pr_table.describe([1.0, 2.0, 3.0, 4.0])
        self.assertEqual(d["mean"], 2.5)
        self.assertAlmostEqual(d["sd"], 1.2909944, places=6)
        self.assertAlmostEqual(d["ci_half"], 3.182446 * 1.2909944 / 2, places=3)

    def test_delta_of_ratio_of_means(self):
        base = {"mean": 100.0, "sd": 1.0, "n": 1000}
        cand = {"mean": 90.0, "sd": 0.9, "n": 1000}
        pct, lo, hi = pr_table.delta(base, cand)
        self.assertAlmostEqual(pct, -10.0)
        # Each mean is known to 0.1% / sqrt(1000) relative SE, combined * sqrt(2).
        half = 1.9623 * 0.9 * math.sqrt(2) * 0.01 / math.sqrt(1000) * 100
        self.assertAlmostEqual(hi - pct, half, places=3)
        self.assertAlmostEqual(pct - lo, half, places=3)

    def test_delta_undefined_for_one_sample(self):
        self.assertTrue(math.isnan(pr_table.delta({"mean": 1, "sd": math.nan, "n": 1},
                                                  {"mean": 1, "sd": math.nan, "n": 1})[0]))


class RenderTest(unittest.TestCase):
    def samples(self, scale):
        return [{"wall_ms": scale * w, "user_ms": 2.0 * w, "sys_ms": 1.0, "max_rss_kib": 1024.0 * w}
                for w in (10.0, 11.0, 12.0, 11.0)]

    def test_unavailable_counters_are_listed(self):
        table = pr_table.render({"label": "upstream-main aaaa", "samples": self.samples(1.0)},
                                {"label": "pr-head bbbb", "samples": self.samples(0.9)},
                                {"cycles": "hardware counter unavailable: No such file or directory"})
        lines = table.splitlines()
        self.assertEqual(lines[0], "#1 baseline  upstream-main aaaa")
        self.assertEqual(lines[1], "#2 candidate pr-head bbbb")
        self.assertEqual(len(lines), 3 + len(pr_table.METRICS))
        self.assertTrue(lines[3].startswith("  wall                     9.900 ms"))
        self.assertIn("-10.00% [", lines[3])
        self.assertEqual(lines[7], "  cycles                        n/a   "
                                   "hardware counter unavailable: No such file or directory")
        self.assertTrue(lines[8].endswith("n/a   not measured"))

    def test_cli_renders_markdown_from_reports(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "r.json"
            path.write_text(json.dumps({"benchmark": "wild", "capture": "/c", "table": "T",
                                        "table_notes": "N."}))
            out = subprocess.run([sys.executable, str(HERE / "pr_table.py"), str(path)],
                                 capture_output=True, text=True, check=True).stdout
        self.assertEqual(out, "wild:\n\n```\nT\n```\n\nN.\n")


if __name__ == "__main__":
    unittest.main()
