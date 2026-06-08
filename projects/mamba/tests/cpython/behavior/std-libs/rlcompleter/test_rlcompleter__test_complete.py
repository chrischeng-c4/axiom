# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_complete"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_complete"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_complete
"""Auto-ported test: TestRlcompleter::test_complete (CPython 3.12 oracle)."""


import rlcompleter
from unittest.mock import patch


with patch("rlcompleter._readline_available", False):
    completer = rlcompleter.Completer()
    assert completer.complete("", 0) == "\t"
    assert completer.complete("a", 0) == "and "
    assert completer.complete("a", 1) == "as "
    assert completer.complete("as", 2) == "assert "
    assert completer.complete("an", 0) == "and "
    assert completer.complete("pa", 0) == "pass"
    assert completer.complete("Fa", 0) == "False"
    assert completer.complete("el", 0) == "elif "
    assert completer.complete("el", 1) == "else"
    assert completer.complete("tr", 0) == "try:"
    assert completer.complete("_", 0) == "_"
    assert completer.complete("match", 0) == "match "
    assert completer.complete("case", 0) == "case "

print("TestRlcompleter::test_complete: ok")
