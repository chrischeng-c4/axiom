# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_isreg_is_callable"
# subject = "stat.S_ISREG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISREG: s_isreg_is_callable (surface)."""
import stat

assert callable(stat.S_ISREG)
print("s_isreg_is_callable OK")
