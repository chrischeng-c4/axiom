# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "edom_attr_exists"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno: edom_attr_exists (surface)."""
import errno

assert hasattr(errno, "EDOM")
print("edom_attr_exists OK")
