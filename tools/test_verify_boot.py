"""Verify boot-tool paths and cleanup without starting a server or using the network."""

import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import tomllib
from types import ModuleType
import unittest
from unittest import mock


class VerifyBootPathsTests(unittest.TestCase):
    """Check boot-tool paths and process cleanup without network or child processes."""

    def load_verify_boot(self) -> ModuleType:
        """Load a fresh boot-tool module so its repository paths are recomputed.

        Returns:
            The independently loaded verification module.
        """
        path = Path(__file__).resolve().with_name("verify_boot.py")
        spec = importlib.util.spec_from_file_location("verify_boot_under_test", path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_repository_paths_do_not_depend_on_the_working_directory(self) -> None:
        """Check that loading outside the repository preserves repository paths."""
        original = Path.cwd()
        with tempfile.TemporaryDirectory() as directory:
            try:
                os.chdir(directory)
                module = self.load_verify_boot()
            finally:
                # Restore the working directory before the temporary cleanup:
                # Windows refuses to delete the process working directory.
                os.chdir(original)
        root = Path(__file__).resolve().parent.parent
        self.assertEqual(module.ROOT, root)
        self.assertEqual(module.BIN, root / "target" / "release" / "rustxng")

    def test_boot_check_uses_repository_paths_and_stops_only_its_own_process(self) -> None:
        """Check command paths and cleanup using mocked process and HTTP calls."""
        module = self.load_verify_boot()
        settings = tomllib.loads((module.ROOT / "settings.toml").read_text(encoding="utf-8"))
        payload = json.dumps({"engines": [{"name": engine["name"]} for engine in settings["engines"]]})
        with mock.patch.object(module.subprocess, "Popen") as popen, mock.patch.object(
            module.urllib.request, "urlopen", side_effect=[io.BytesIO(b"ok"), io.BytesIO(payload.encode())]
        ), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(module.main(), 0)
        self.assertEqual(popen.call_args.args[0][0], module.BIN)
        popen.return_value.terminate.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()
