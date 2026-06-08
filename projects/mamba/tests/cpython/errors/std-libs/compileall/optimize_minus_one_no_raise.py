# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "optimize_minus_one_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: optimize=-1 means 'use the interpreter default'; compile_file accepts it without raising and returns True"""
import compileall
import os
import tempfile

# optimize=-1 is the sentinel for "use the running interpreter's optimization
# level" — a documented valid value, not an out-of-range error.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "opt.py")
    with open(src, "w") as f:
        f.write("x = 1\n")
    result = compileall.compile_file(src, optimize=-1, quiet=2)
    assert result is True, result
print("optimize_minus_one_no_raise OK")
