# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winconsoleio"
# dimension = "behavior"
# case = "windows_console_io_tests__test_abc"
# subject = "cpython.test_winconsoleio.WindowsConsoleIOTests.test_abc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_winconsoleio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_winconsoleio.py::WindowsConsoleIOTests::test_abc
"""Auto-ported test: WindowsConsoleIOTests::test_abc (CPython 3.12 oracle)."""


import io
import sys


if sys.platform != "win32":
    print("WindowsConsoleIOTests::test_abc: skipped, win32 only")
    raise SystemExit(0)

ConIO = io._WindowsConsoleIO
assert issubclass(ConIO, io.RawIOBase)
assert not issubclass(ConIO, io.BufferedIOBase)
assert not issubclass(ConIO, io.TextIOBase)

print("WindowsConsoleIOTests::test_abc: ok")
