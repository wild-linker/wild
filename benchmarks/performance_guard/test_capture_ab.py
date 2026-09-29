import tempfile
import unittest
from pathlib import Path

import capture_ab


class CaptureAbTest(unittest.TestCase):
    def test_parse_perf_reads_counters_and_skips_unsupported(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "perf.csv"
            path.write_text(
                "# started on Mon Sep 28\n"
                "\n"
                "455.70,msec,task-clock,455700000,100.00,5.998,CPUs utilized\n"
                "1383462596,,cycles:u,455000000,100.00,,\n"
                "<not supported>,,dTLB-load-misses:u,0,100.00,,\n"
            )
            counters = capture_ab.parse_perf(path)
        self.assertEqual(counters, {"task-clock": 455.70, "cycles:u": 1383462596.0})

    def test_parse_perf_converts_nanosecond_task_clock_to_ms(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "perf.csv"
            path.write_text("334053834.5,ns,task-clock,334053834,100.00,,\n")
            counters = capture_ab.parse_perf(path)
        self.assertAlmostEqual(counters["task-clock"], 334.0538345)

    def test_summarise_takes_median_per_metric(self):
        summary = capture_ab.summarise(
            [{"wall_ms": 3.0, "cycles:u": 10.0}, {"wall_ms": 1.0}, {"wall_ms": 2.0, "cycles:u": 30.0}]
        )
        self.assertEqual(summary["wall_ms"]["median"], 2.0)
        self.assertEqual(summary["wall_ms"]["min"], 1.0)
        self.assertEqual(summary["cycles:u"]["median"], 20.0)


if __name__ == "__main__":
    unittest.main()
