import json
import tempfile
import unittest
from pathlib import Path

import render_performance


class RenderPerformanceTest(unittest.TestCase):
    def valid_report(self):
        return {
            "schema_version": 1,
            "title": "Example",
            "baseline_sha": "base",
            "candidate_sha": "candidate",
            "series": [
                {
                    "mode": "none",
                    "samples_ms": {
                        "baseline": [10.0, 12.0],
                        "candidate": [8.0, 9.0],
                    },
                }
            ],
        }

    def test_render_is_deterministic_and_self_contained(self):
        first = render_performance.render(self.valid_report())
        second = render_performance.render(self.valid_report())
        self.assertEqual(first, second)
        self.assertTrue(first.startswith('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertNotIn("http://", first.removeprefix('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertIn("22.7% faster", first)

    def test_load_rejects_malformed_and_invalid_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            path.write_text("not json")
            with self.assertRaises(render_performance.SchemaError):
                render_performance.load_report(path)
            path.write_text(json.dumps({"schema_version": 2, "series": []}))
            with self.assertRaisesRegex(render_performance.SchemaError, "schema_version"):
                render_performance.load_report(path)
            path.write_text(json.dumps({"schema_version": True, "series": []}))
            with self.assertRaisesRegex(render_performance.SchemaError, "schema_version"):
                render_performance.load_report(path)

    def test_load_rejects_bad_vectors(self):
        report = self.valid_report()
        report["series"][0]["samples_ms"]["candidate"] = [8.0]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            path.write_text(json.dumps(report))
            with self.assertRaisesRegex(render_performance.SchemaError, "candidate needs"):
                render_performance.load_report(path)

    def test_load_rejects_boolean_samples(self):
        report = self.valid_report()
        report["series"][0]["samples_ms"]["candidate"][0] = True
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            path.write_text(json.dumps(report))
            with self.assertRaisesRegex(render_performance.SchemaError, "finite positive numbers"):
                render_performance.load_report(path)


if __name__ == "__main__":
    unittest.main()
