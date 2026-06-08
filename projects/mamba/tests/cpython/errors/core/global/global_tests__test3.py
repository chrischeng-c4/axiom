# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "global"
# dimension = "errors"
# case = "global_tests__test3"
# subject = "cpython.test_global.GlobalTests.test3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_global.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GlobalTests.test3: read and assign before global declaration is a SyntaxError."""

SOURCE = """\
def wrong3():
    print(x)
    x = 2
    global x
"""

try:
    compile(SOURCE, "<test string>", "exec")
except SyntaxError as exc:
    assert exc.msg == "name 'x' is used prior to global declaration", exc.msg
    assert exc.lineno == 4, exc.lineno
    assert exc.offset == 5, exc.offset
    assert exc.end_lineno == 4, exc.end_lineno
    assert exc.end_offset == 13, exc.end_offset
else:
    raise AssertionError("compile() did not raise SyntaxError")

print("GlobalTests::test3: ok")
