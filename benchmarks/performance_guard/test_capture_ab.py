import os
import stat
import tempfile
import unittest
from pathlib import Path

import capture_ab


class CaptureAbTest(unittest.TestCase):
    def test_summarise_takes_median_per_metric(self):
        summary = capture_ab.summarise(
            [{"wall_ms": 3.0, "cycles": 10.0}, {"wall_ms": 1.0}, {"wall_ms": 2.0, "cycles": 30.0}]
        )
        self.assertEqual(summary["wall_ms"]["median"], 2.0)
        self.assertEqual(summary["wall_ms"]["min"], 1.0)
        self.assertEqual(summary["cycles"]["median"], 20.0)

    def test_keep_going_honours_minimum_budget_and_cap(self):
        self.assertTrue(capture_ab.keep_going(3, 100.0, 5, 5, 0))  # below minimum
        self.assertFalse(capture_ab.keep_going(5, 0.0, 5, 5, 60))  # cap reached
        self.assertTrue(capture_ab.keep_going(5, 10.0, 5, 50, 60))  # within budget
        self.assertFalse(capture_ab.keep_going(5, 61.0, 5, 50, 60))  # budget spent
        self.assertFalse(capture_ab.keep_going(0, 0.0, 0, 0, 0))  # identity-only

    def test_counters_disabled_report_reason(self):
        counters = capture_ab.Counters(enabled=False)
        self.assertEqual(counters.available, [])
        self.assertIn("--no-counters", counters.reason)

    def test_run_once_measures_a_fake_capture(self):
        with tempfile.TemporaryDirectory() as tmp:
            capture = Path(tmp)
            run_with = capture / "run-with"
            # Like a real save-dir script: the linker is the first argument, then `--` and extras.
            run_with.write_text('#!/bin/sh\nlinker="$1"; shift; [ "$1" = -- ] && shift\n'
                                'exec "$linker" "$OUT" "$@"\n')
            linker = capture / "linker"
            linker.write_text('#!/bin/sh\necho "$@" > "$1"\n')
            for path in (run_with, linker):
                path.chmod(path.stat().st_mode | stat.S_IXUSR)
            out = capture / "out"
            counters = capture_ab.Counters()
            sample = capture_ab.run_once(capture, linker, out, counters, ["--no-fork"], None)
            self.assertEqual(out.read_text(), f"{out} --no-fork\n")
        for key in ("wall_ms", "user_ms", "sys_ms", "max_rss_kib"):
            self.assertIn(key, sample)
        self.assertGreater(sample["wall_ms"], 0)
        self.assertGreater(sample["max_rss_kib"], 0)
        for name, _ in counters.available:
            self.assertGreater(sample[name], 0, name)

    def test_run_once_raises_on_linker_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            capture = Path(tmp)
            run_with = capture / "run-with"
            run_with.write_text("#!/bin/sh\nexit 3\n")
            run_with.chmod(0o755)
            with self.assertRaises(capture_ab.subprocess.CalledProcessError):
                capture_ab.run_once(capture, Path("/bin/true"), capture / "o", None, [], None)


if __name__ == "__main__":
    unittest.main()
