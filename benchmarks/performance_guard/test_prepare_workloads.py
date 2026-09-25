import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import prepare_workloads


class PrepareWorkloadsTest(unittest.TestCase):
    def test_generate_unit_source(self):
        source = prepare_workloads.generate_unit_source(1, 3)
        self.assertIn("f3", source)
        self.assertIn("f5", source)
        self.assertIn("table_1[]", source)
        self.assertIn("typedef uint64_t (*fn)(uint64_t);\n", source)

    def test_main_writes_all_workloads(self):
        if shutil.which("clang") is None:
            self.skipTest("clang is not available")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            argv = ["prepare_workloads.py", "--output", str(root)]
            with mock.patch("sys.argv", argv):
                self.assertEqual(prepare_workloads.main(), 0)
            manifest = json.loads((root / "manifest.json").read_text())
            for key in ("ordinary", "full-lto", "large-debug"):
                self.assertIn(key, manifest)
                for obj in manifest[key]["objects"]:
                    self.assertTrue(Path(obj).is_file(), obj)


if __name__ == "__main__":
    unittest.main()
