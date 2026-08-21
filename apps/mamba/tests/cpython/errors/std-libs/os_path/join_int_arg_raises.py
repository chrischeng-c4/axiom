# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "join_int_arg_raises"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join_int_arg_raises (errors)."""
import os.path

_raised = False
try:
    os.path.join(123, 'x')
except TypeError:
    _raised = True
assert _raised, "join_int_arg_raises: expected TypeError"
print("join_int_arg_raises OK")
