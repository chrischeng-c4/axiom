# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_issock_is_callable"
# subject = "stat.S_ISSOCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_ISSOCK: s_issock_is_callable (surface)."""
import stat

assert callable(stat.S_ISSOCK)
print("s_issock_is_callable OK")
