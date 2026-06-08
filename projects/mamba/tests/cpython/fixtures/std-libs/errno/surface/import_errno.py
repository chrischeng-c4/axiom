# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "import_errno"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno: import_errno (surface)."""
import errno

assert hasattr(errno, "errorcode")
print("import_errno OK")
