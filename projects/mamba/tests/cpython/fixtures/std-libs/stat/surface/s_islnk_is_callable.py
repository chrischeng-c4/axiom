# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_islnk_is_callable"
# subject = "stat.S_ISLNK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISLNK: s_islnk_is_callable (surface)."""
import stat

assert callable(stat.S_ISLNK)
print("s_islnk_is_callable OK")
