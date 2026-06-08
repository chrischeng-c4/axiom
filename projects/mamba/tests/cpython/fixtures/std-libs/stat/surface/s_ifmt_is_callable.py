# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_ifmt_is_callable"
# subject = "stat.S_IFMT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IFMT: s_ifmt_is_callable (surface)."""
import stat

assert callable(stat.S_IFMT)
print("s_ifmt_is_callable OK")
