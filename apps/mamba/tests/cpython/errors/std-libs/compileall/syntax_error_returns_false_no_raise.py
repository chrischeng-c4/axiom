# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "syntax_error_returns_false_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: a .py file with a SyntaxError makes compile_file return False rather than raising; the bad source is reported, not propagated"""
import compileall
import os
import tempfile

# A broken source does not propagate the SyntaxError: compile_file catches it,
# reports the failure, and returns a falsy verdict so callers stay in control.
with tempfile.TemporaryDirectory() as d:
    bad = os.path.join(d, "bad.py")
    with open(bad, "w") as f:
        f.write("def f(\n  syntax error here\n")
    result = compileall.compile_file(bad, quiet=2)
    assert result is False, result
print("syntax_error_returns_false_no_raise OK")
