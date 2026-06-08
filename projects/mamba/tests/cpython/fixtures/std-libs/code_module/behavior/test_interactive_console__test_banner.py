# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code_module"
# dimension = "behavior"
# case = "test_interactive_console__test_banner"
# subject = "cpython.test_code_module.TestInteractiveConsole.test_banner"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code_module.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_code_module.py::TestInteractiveConsole::test_banner
"""Auto-ported test: TestInteractiveConsole::test_banner (CPython 3.12 oracle)."""


import code
import sys
from unittest import mock


missing = object()
old_ps1 = getattr(sys, "ps1", missing)
old_ps2 = getattr(sys, "ps2", missing)

try:
    for attr in ("ps1", "ps2"):
        if hasattr(sys, attr):
            delattr(sys, attr)

    console = code.InteractiveConsole()
    with (
        mock.patch("code.input") as infunc,
        mock.patch("code.sys.stdout"),
        mock.patch("code.sys.stderr") as stderr,
    ):
        infunc.side_effect = EOFError("Finished")
        console.interact(banner="Foo")
        assert stderr.method_calls == [
            mock.call.write("Foo\n"),
            mock.call.write("\n"),
            mock.call.write("now exiting InteractiveConsole...\n"),
        ], stderr.method_calls
finally:
    for attr in ("ps1", "ps2"):
        if hasattr(sys, attr):
            delattr(sys, attr)
    if old_ps1 is not missing:
        sys.ps1 = old_ps1
    if old_ps2 is not missing:
        sys.ps2 = old_ps2

print("TestInteractiveConsole::test_banner: ok")
