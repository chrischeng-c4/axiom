# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_isdir_is_callable"
# subject = "stat.S_ISDIR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISDIR: s_isdir_is_callable (surface)."""
import stat

assert callable(stat.S_ISDIR)
print("s_isdir_is_callable OK")
