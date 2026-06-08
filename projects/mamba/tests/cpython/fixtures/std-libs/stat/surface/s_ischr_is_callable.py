# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_ischr_is_callable"
# subject = "stat.S_ISCHR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISCHR: s_ischr_is_callable (surface)."""
import stat

assert callable(stat.S_ISCHR)
print("s_ischr_is_callable OK")
