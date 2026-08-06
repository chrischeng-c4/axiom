from __future__ import annotations

from pathlib import Path
import sys
import unittest

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from build_stamp.domain.directive import (
    Directive,
    DirectiveKind,
    DirectiveRejection,
    make_directive,
    sanitize_key,
)


class TestDomainDirective(unittest.TestCase):
    def test_render_rustc_env(self) -> None:
        d = Directive(DirectiveKind.RUSTC_ENV, "LUMEN_GIT_SHA", "c3ff13cd")
        self.assertEqual(d.render(), "cargo:rustc-env=LUMEN_GIT_SHA=c3ff13cd")

    def test_render_rerun_if_changed(self) -> None:
        d = Directive(DirectiveKind.RERUN_IF_CHANGED, "", "../../.git/HEAD")
        self.assertEqual(d.render(), "cargo:rerun-if-changed=../../.git/HEAD")

    def test_make_directive_valid(self) -> None:
        d = make_directive(DirectiveKind.RUSTC_ENV, "PREFIX_KEY", "value")
        self.assertIsInstance(d, Directive)
        if isinstance(d, Directive):
            self.assertEqual(d.kind, DirectiveKind.RUSTC_ENV)
            self.assertEqual(d.key, "PREFIX_KEY")
            self.assertEqual(d.value, "value")

    def test_make_directive_no_blocklist_on_value(self) -> None:
        d = make_directive(DirectiveKind.RUSTC_ENV, "K", "cargo:rustc-cfg=x")
        self.assertIsInstance(d, Directive)
        if isinstance(d, Directive):
            self.assertEqual(d.value, "cargo:rustc-cfg=x")

    def test_make_directive_control_character_newline(self) -> None:
        res = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\nb")
        self.assertEqual(res, DirectiveRejection.CONTROL_CHARACTER)

    def test_make_directive_control_character_carriage_return(self) -> None:
        res = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\rb")
        self.assertEqual(res, DirectiveRejection.CONTROL_CHARACTER)

    def test_make_directive_control_character_tab(self) -> None:
        res = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\tb")
        self.assertEqual(res, DirectiveRejection.CONTROL_CHARACTER)

    def test_make_directive_control_character_delete(self) -> None:
        res = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\x7fb")
        self.assertEqual(res, DirectiveRejection.CONTROL_CHARACTER)

    def test_make_directive_empty_key(self) -> None:
        res = make_directive(DirectiveKind.RUSTC_ENV, "", "v")
        self.assertEqual(res, DirectiveRejection.EMPTY_KEY)

    def test_sanitize_key(self) -> None:
        self.assertEqual(sanitize_key("A\nB\tC"), "ABC")
        self.assertEqual(sanitize_key("CLEAN_KEY"), "CLEAN_KEY")
        self.assertEqual(sanitize_key("\x7fKEY\r"), "KEY")


if __name__ == "__main__":
    unittest.main()
