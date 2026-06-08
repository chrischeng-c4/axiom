# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_warning"
# subject = "cpython.test_codeop.CodeopTests.test_warning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: CodeopTests::test_warning (CPython 3.12 oracle)."""

import warnings
from codeop import compile_command
from test.support import warnings_helper


with warnings_helper.check_warnings(
    ('"is" with \'str\' literal', SyntaxWarning),
    ("invalid escape sequence", SyntaxWarning),
) as w:
    compile_command(r"'\e' is 0")
    assert len(w.warnings) == 2

with warnings.catch_warnings():
    warnings.simplefilter("error", SyntaxWarning)
    try:
        compile_command("1 is 1", symbol="exec")
    except SyntaxError:
        pass
    else:
        raise AssertionError("SyntaxWarning-as-error did not raise SyntaxError")

with warnings.catch_warnings():
    warnings.simplefilter("error", SyntaxWarning)
    try:
        compile_command(r"'\e'", symbol="exec")
    except SyntaxError:
        pass
    else:
        raise AssertionError("invalid escape SyntaxWarning-as-error did not raise SyntaxError")

print("CodeopTests::test_warning: ok")
