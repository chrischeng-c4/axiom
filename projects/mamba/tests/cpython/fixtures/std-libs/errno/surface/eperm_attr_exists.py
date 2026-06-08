# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "eperm_attr_exists"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno: eperm_attr_exists (surface)."""
import errno

assert hasattr(errno, "EPERM")
print("eperm_attr_exists OK")
