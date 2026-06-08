# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "flufl"
# dimension = "errors"
# case = "flufl_tests__test_barry_as_bdfl_relative_import"
# subject = "cpython.test_flufl.FLUFLTests.test_barry_as_bdfl_relative_import"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_flufl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""FLUFLTests.test_barry_as_bdfl_relative_import: relative import is not future."""

SOURCE = "from .__future__ import barry_as_FLUFL;2 {op} 3"
REJECTED_SOURCE = SOURCE.format(op="<>")

compile(SOURCE.format(op="!="), "<FLUFL test>", "exec")

try:
    compile(REJECTED_SOURCE, "<BDFL test>", "exec")
except SyntaxError as exc:
    assert "<BDFL test>" in str(exc), exc
    assert "2 <> 3" in exc.text, exc.text
    assert exc.filename == "<BDFL test>", exc.filename
    assert exc.lineno == 1, exc.lineno
    assert exc.offset == REJECTED_SOURCE.index("<>") + 1, exc.offset
else:
    raise AssertionError("relative __future__ import made <> legal")

print("FLUFLTests::test_barry_as_bdfl_relative_import: ok")
