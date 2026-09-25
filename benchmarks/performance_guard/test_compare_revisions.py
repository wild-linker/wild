import contextlib
import io
import unittest

import compare_revisions


BUILDS = [
    {"label": "upstream-main", "sha": "a" * 40, "path": "/a/wild"},
    {"label": "upstream-main+5", "sha": "b" * 40, "path": "/b/wild"},
    {"label": "fork-tip", "sha": "c" * 40, "path": "/c/wild"},
]


class CompareRevisionsTest(unittest.TestCase):
    def test_summarize(self):
        summary = compare_revisions.summarize([1.0, 2.0, 3.0, 4.0])
        self.assertAlmostEqual(summary["median_ms"], 2500.0)
        self.assertAlmostEqual(summary["min_ms"], 1000.0)
        self.assertAlmostEqual(summary["max_ms"], 4000.0)
        self.assertEqual(summary["runs"], 4)
        self.assertGreater(summary["stddev_ms"], 0.0)

    def test_classify(self):
        self.assertEqual(compare_revisions.classify(100.0, 105.0, 2.0), "slower")
        self.assertEqual(compare_revisions.classify(100.0, 95.0, 2.0), "faster")
        self.assertEqual(compare_revisions.classify(100.0, 101.0, 2.0), "noise")

    def test_render_markdown(self):
        results = {
            case["name"]: {
                build["label"]: compare_revisions.summarize([0.1, 0.2, 0.3])
                for build in BUILDS
            }
            for case in compare_revisions.CASES
        }
        markdown = compare_revisions.render_markdown(BUILDS, results)
        self.assertIn("| Workload | Build | SHA | Median ms | Stddev ms | Runs | vs baseline |", markdown)
        for build in BUILDS:
            self.assertIn(build["label"], markdown)
            self.assertIn(build["sha"], markdown)
        self.assertEqual(len(compare_revisions.CASES), 4)
        for case in compare_revisions.CASES:
            self.assertIn(case["name"], markdown)

    def test_rejects_too_few_runs(self):
        argv = ["--manifest", "m.json", "--output", "out", "--runs", "9"]
        for build in BUILDS:
            argv += ["--build", f"{build['label']}={build['sha']}={build['path']}"]
        with contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                compare_revisions.parse_args(argv)
        argv[argv.index("9")] = "10"
        self.assertEqual(compare_revisions.parse_args(argv).runs, 10)


if __name__ == "__main__":
    unittest.main()
