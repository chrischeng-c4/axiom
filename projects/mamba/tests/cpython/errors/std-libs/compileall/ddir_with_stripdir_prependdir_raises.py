# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "ddir_with_stripdir_prependdir_raises"
# subject = "compileall.compile_dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: ddir_with_stripdir_prependdir_raises (errors)."""
import compileall
import tempfile

_raised = False
try:
    compileall.compile_dir(tempfile.mkdtemp(), quiet=True, ddir="/bar", stripdir="/foo", prependdir="/bar")
except ValueError:
    _raised = True
assert _raised, "ddir_with_stripdir_prependdir_raises: expected ValueError"
print("ddir_with_stripdir_prependdir_raises OK")
