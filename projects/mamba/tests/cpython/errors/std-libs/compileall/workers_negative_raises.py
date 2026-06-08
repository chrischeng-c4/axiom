# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "workers_negative_raises"
# subject = "compileall.compile_dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: workers_negative_raises (errors)."""
import compileall
import tempfile

_raised = False
try:
    compileall.compile_dir(tempfile.mkdtemp(), workers=-1)
except ValueError:
    _raised = True
assert _raised, "workers_negative_raises: expected ValueError"
print("workers_negative_raises OK")
