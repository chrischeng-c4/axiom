# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "flufl"
# dimension = "errors"
# case = "flufl_tests__test_guido_as_bdfl"
# subject = "cpython.test_flufl.FLUFLTests.test_guido_as_bdfl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_flufl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""FLUFLTests.test_guido_as_bdfl: normal parser accepts != and rejects <>."""

SOURCE = "2 {op} 3"

compile(SOURCE.format(op="!="), "<BDFL test>", "exec")

try:
    compile(SOURCE.format(op="<>"), "<FLUFL test>", "exec")
except SyntaxError as exc:
    assert "invalid syntax" in str(exc), exc
    assert "2 <> 3" in exc.text, exc.text
    assert exc.filename == "<FLUFL test>", exc.filename
    assert exc.lineno == 1, exc.lineno
    assert exc.offset == 3, exc.offset
else:
    raise AssertionError("normal parser did not reject <>")

print("FLUFLTests::test_guido_as_bdfl: ok")
