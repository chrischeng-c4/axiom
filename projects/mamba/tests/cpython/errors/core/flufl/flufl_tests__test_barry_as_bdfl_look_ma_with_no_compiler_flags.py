# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "flufl"
# dimension = "errors"
# case = "flufl_tests__test_barry_as_bdfl_look_ma_with_no_compiler_flags"
# subject = "cpython.test_flufl.FLUFLTests.test_barry_as_bdfl_look_ma_with_no_compiler_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_flufl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""FLUFLTests.test_barry_as_bdfl_look_ma_with_no_compiler_flags."""

SOURCE = "from __future__ import barry_as_FLUFL;2 {op} 3"
REJECTED_SOURCE = SOURCE.format(op="!=")

compile(SOURCE.format(op="<>"), "<BDFL test>", "exec")

try:
    compile(REJECTED_SOURCE, "<FLUFL test>", "exec")
except SyntaxError as exc:
    assert "with Barry as BDFL, use '<>' instead of '!='" in str(exc), exc
    assert "2 != 3" in exc.text, exc.text
    assert exc.filename == "<FLUFL test>", exc.filename
    assert exc.lineno == 1, exc.lineno
    assert exc.offset == REJECTED_SOURCE.index("!=") + 1, exc.offset
else:
    raise AssertionError("future import did not reject != without explicit flags")

print("FLUFLTests::test_barry_as_bdfl_look_ma_with_no_compiler_flags: ok")
