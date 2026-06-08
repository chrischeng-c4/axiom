# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "flufl"
# dimension = "errors"
# case = "flufl_tests__test_barry_as_bdfl"
# subject = "cpython.test_flufl.FLUFLTests.test_barry_as_bdfl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_flufl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""FLUFLTests.test_barry_as_bdfl: future flag accepts <> and rejects !=."""
import __future__

SOURCE = "from __future__ import barry_as_FLUFL\n2 {op} 3"

compile(
    SOURCE.format(op="<>"),
    "<BDFL test>",
    "exec",
    __future__.CO_FUTURE_BARRY_AS_BDFL,
)

try:
    compile(
        SOURCE.format(op="!="),
        "<FLUFL test>",
        "exec",
        __future__.CO_FUTURE_BARRY_AS_BDFL,
    )
except SyntaxError as exc:
    assert "with Barry as BDFL, use '<>' instead of '!='" in str(exc), exc
    assert "2 != 3" in exc.text, exc.text
    assert exc.filename == "<FLUFL test>", exc.filename
    assert exc.lineno == 2, exc.lineno
    assert exc.offset == 3, exc.offset
else:
    raise AssertionError("barry_as_FLUFL did not reject !=")

print("FLUFLTests::test_barry_as_bdfl: ok")
