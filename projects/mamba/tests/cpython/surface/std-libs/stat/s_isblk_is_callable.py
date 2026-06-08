# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_isblk_is_callable"
# subject = "stat.S_ISBLK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISBLK: s_isblk_is_callable (surface)."""
import stat

assert callable(stat.S_ISBLK)
print("s_isblk_is_callable OK")
