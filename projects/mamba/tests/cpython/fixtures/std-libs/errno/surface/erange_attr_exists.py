# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "erange_attr_exists"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno: erange_attr_exists (surface)."""
import errno

assert hasattr(errno, "ERANGE")
print("erange_attr_exists OK")
