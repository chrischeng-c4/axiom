# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_invalid_warning"
# subject = "cpython.test_codeop.CodeopTests.test_invalid_warning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: CodeopTests::test_invalid_warning (CPython 3.12 oracle)."""

import re
import warnings
from codeop import compile_command


def assert_invalid(source, symbol="single", is_syntax=True):
    try:
        compile_command(source, symbol=symbol)
    except SyntaxError:
        assert is_syntax
    except OverflowError:
        assert not is_syntax
    else:
        raise AssertionError("No exception raised for invalid code")


with warnings.catch_warnings(record=True) as w:
    warnings.simplefilter("always")
    assert_invalid("'\\e' 1")

assert len(w) == 1
assert w[0].category is SyntaxWarning
assert re.search("invalid escape sequence", str(w[0].message))
assert w[0].filename == "<input>"

print("CodeopTests::test_invalid_warning: ok")
