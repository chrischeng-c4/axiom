# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "ddir_none_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file with ddir=None is accepted (does not raise); it only affects the recorded source path and still returns True"""
import compileall
import os
import tempfile

# ddir=None is the default-equivalent: it only influences the source path
# recorded inside the .pyc, never an error condition.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "m.py")
    with open(src, "w") as f:
        f.write("x = 1\n")
    result = compileall.compile_file(src, ddir=None, quiet=2)
    assert result is True, result
print("ddir_none_no_raise OK")
