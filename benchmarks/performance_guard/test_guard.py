import json
import subprocess
import tempfile
import unittest
from pathlib import Path
import guard


class GuardTest(unittest.TestCase):
    def test_slowdown_is_regression(self):
        result = guard.classify([1.0] * 12, [1.03] * 12, 1.5, 1.5)
        self.assertEqual(result["status"], "regression")

    def test_unchanged_is_not_a_win(self):
        result = guard.classify([1.0] * 12, [1.0] * 12, 1.5, 1.5)
        self.assertEqual(result["status"], "no-material-change")

    def test_speedup_passes(self):
        result = guard.classify([1.0] * 12, [0.97] * 12, 1.5, 1.5)
        self.assertEqual(result["status"], "improvement")

    def test_threshold_overlap_is_inconclusive(self):
        baseline = [1.0] * 12
        candidate = [0.97, 1.01] * 6
        self.assertEqual(
            guard.classify(baseline, candidate, 1.5, 1.5)["status"], "inconclusive"
        )

    def test_reversed_order_results_are_aggregated_by_name(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "block.json"
            path.write_text(
                json.dumps(
                    {
                        "results": [
                            {"command": "candidate", "times": [0.8, 0.9]},
                            {"command": "baseline", "times": [1.0, 1.1]},
                        ]
                    }
                )
            )
            self.assertEqual(
                guard.read_hyperfine(path),
                {"candidate": [0.8, 0.9], "baseline": [1.0, 1.1]},
            )

    def test_command_failure_is_an_error(self):
        with self.assertRaises(guard.GuardError):
            guard.run_checked(["sh", "-c", "exit 7"], stdout=subprocess.DEVNULL)

    def test_missing_perf_counters_are_nonfatal_diagnostic(self):
        self.assertEqual(
            guard.counter_status("perf_event_open: permission denied", 1), "unavailable"
        )
        self.assertEqual(
            guard.counter_status("normal output", 1, perf_paranoid=4), "unavailable"
        )

    def test_blocking_policy_rejects_regression(self):
        cases = [{"target": False, "classification": {"status": "regression"}}]
        self.assertTrue(guard.would_fail_policy(cases, False))

    def test_performance_claim_requires_target_improvement(self):
        unchanged = [
            {"target": True, "classification": {"status": "no-material-change"}}
        ]
        faster = [{"target": True, "classification": {"status": "improvement"}}]
        self.assertTrue(guard.would_fail_policy(unchanged, True))
        self.assertFalse(guard.would_fail_policy(faster, True))

    def test_non_target_win_cannot_satisfy_performance_claim(self):
        cases = [{"target": False, "classification": {"status": "improvement"}}]
        self.assertTrue(guard.would_fail_policy(cases, True))


if __name__ == "__main__":
    unittest.main()
