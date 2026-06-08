# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "global"
# dimension = "errors"
# case = "global_tests__test1"
# subject = "cpython.test_global.GlobalTests.test1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_global.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GlobalTests.test1: assignment before global declaration is a SyntaxError."""

SOURCE = """\
def wrong1():
    a = 1
    b = 2
    global a
    global b
"""

try:
    compile(SOURCE, "<test string>", "exec")
except SyntaxError as exc:
    assert exc.msg == "name 'a' is assigned to before global declaration", exc.msg
    assert exc.lineno == 4, exc.lineno
    assert exc.offset == 5, exc.offset
    assert exc.end_lineno == 4, exc.end_lineno
    assert exc.end_offset == 13, exc.end_offset
else:
    raise AssertionError("compile() did not raise SyntaxError")

print("GlobalTests::test1: ok")
