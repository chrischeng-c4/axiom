# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "join_mixed_str_bytes_raises"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join_mixed_str_bytes_raises (errors)."""
import os.path

_raised = False
try:
    os.path.join('a', b'b')
except TypeError:
    _raised = True
assert _raised, "join_mixed_str_bytes_raises: expected TypeError"
print("join_mixed_str_bytes_raises OK")
