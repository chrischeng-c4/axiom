# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_imode_is_callable"
# subject = "stat.S_IMODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IMODE: s_imode_is_callable (surface)."""
import stat

assert callable(stat.S_IMODE)
print("s_imode_is_callable OK")
