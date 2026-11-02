"""Offline regression tests for engine registration and TOML validation."""

import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import tomllib
from types import ModuleType
import unittest
from unittest import mock


def load_register_engines() -> ModuleType:
    """Load the registration tool without running its command-line entry point."""
    path = Path(__file__).resolve().with_name("register_engines.py")
    spec = importlib.util.spec_from_file_location("register_engines_under_test", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RegisterEnginesTests(unittest.TestCase):
    """Exercise filtering and file writes using only temporary repository fixtures."""

    def setUp(self) -> None:
        """Create a minimal engine module, dispatch table and settings file."""
        self.tool = load_register_engines()
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.engines = self.root / "engines"
        self.engines.mkdir()
        self.mod = self.engines / "mod.rs"
        self.mod.write_text(
            'pub mod sample;\npub mod deleted;\n'
            '        "sample" => sample::Sample::from_config(config)?,\n'
            '        "deleted" => deleted::Deleted::from_config(config)?,\n'
            '        other => {\n',
            encoding="utf-8",
        )
        (self.engines / "sample.rs").write_text(
            '// engine-id: sample\npub struct Sample;\n'
            'impl Sample { pub fn from_config() {} }\n'
            'impl Engine for Sample {}\n',
            encoding="utf-8",
        )
        self.settings = self.root / "settings.toml"

    def run_registration(self) -> None:
        """Run registration against the temporary fixture rather than the checkout."""
        with mock.patch.multiple(
            self.tool,
            MOD=str(self.mod),
            SETTINGS=str(self.settings),
            ENGINES_DIR=str(self.engines),
        ), contextlib.redirect_stdout(io.StringIO()):
            self.tool.main()

    def test_filter_accepts_toml_quotes_spacing_and_comments(self) -> None:
        """Keep valid types and reject deleted types independently of formatting."""
        settings = (
            '# settings header\n'
            '[[engines]] # supported\n'
            "name='sample alias'\n  engine= 'sample' # valid\n"
            '[[engines]]\nname = "deleted"\nengine="deleted"\n'
        )
        result = self.tool.filter_engine_settings(settings, {"sample"})
        self.assertEqual(result, settings[:settings.index('[[engines]]\n')])
        self.assertEqual(tomllib.loads(result)["engines"][0]["engine"], "sample")

    def test_missing_or_invalid_engine_type_leaves_both_files_unchanged(self) -> None:
        """Report an invalid entry before mutating the module table or settings."""
        for field in ["", "engine = 42\n", 'engine = ""\n']:
            with self.subTest(field=field):
                settings = '[[engines]]\nname = "broken"\n' + field
                self.settings.write_text(settings, encoding="utf-8")
                before = self.mod.read_bytes()
                with self.assertRaisesRegex(ValueError, "engine registration: entry 'broken'"):
                    self.run_registration()
                self.assertEqual(self.mod.read_bytes(), before)
                self.assertEqual(self.settings.read_text(encoding="utf-8"), settings)

    def test_malformed_toml_leaves_both_files_unchanged(self) -> None:
        """Do not partially regenerate modules when settings cannot be parsed."""
        self.settings.write_text("[[engines]\n", encoding="utf-8")
        before = self.mod.read_bytes()
        with self.assertRaises(tomllib.TOMLDecodeError):
            self.run_registration()
        self.assertEqual(self.mod.read_bytes(), before)
        self.assertEqual(self.settings.read_text(encoding="utf-8"), "[[engines]\n")

    def test_registration_is_idempotent_with_alternative_toml_formatting(self) -> None:
        """Avoid duplicate entries after filtering old implementations."""
        self.settings.write_text(
            "[[engines]]\nname='alias'\nengine='sample'\nshortcut='s'\n"
            '[[engines]]\nname="deleted"\nengine="deleted"\n',
            encoding="utf-8",
        )
        self.run_registration()
        first_mod = self.mod.read_bytes()
        first_settings = self.settings.read_bytes()
        self.run_registration()
        self.assertEqual(self.mod.read_bytes(), first_mod)
        self.assertEqual(self.settings.read_bytes(), first_settings)
        self.assertNotIn("deleted", self.mod.read_text(encoding="utf-8"))
        self.assertEqual(
            tomllib.loads(self.settings.read_text(encoding="utf-8"))["engines"],
            [{"name": "alias", "engine": "sample", "shortcut": "s"}],
        )


if __name__ == "__main__":
    unittest.main()
