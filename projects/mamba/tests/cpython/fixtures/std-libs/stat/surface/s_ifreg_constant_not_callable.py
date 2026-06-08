# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_ifreg_constant_not_callable"
# subject = "stat.S_IFREG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IFREG: s_ifreg_constant_not_callable (surface)."""
import stat

assert not callable(stat.S_IFREG)
print("s_ifreg_constant_not_callable OK")
