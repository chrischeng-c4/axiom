# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "global"
# dimension = "errors"
# case = "global_tests__test2"
# subject = "cpython.test_global.GlobalTests.test2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_global.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GlobalTests.test2: use before global declaration is a SyntaxError."""

SOURCE = """\
def wrong2():
    print(x)
    global x
"""

try:
    compile(SOURCE, "<test string>", "exec")
except SyntaxError as exc:
    assert exc.msg == "name 'x' is used prior to global declaration", exc.msg
    assert exc.lineno == 3, exc.lineno
    assert exc.offset == 5, exc.offset
    assert exc.end_lineno == 3, exc.end_lineno
    assert exc.end_offset == 13, exc.end_offset
else:
    raise AssertionError("compile() did not raise SyntaxError")

print("GlobalTests::test2: ok")
