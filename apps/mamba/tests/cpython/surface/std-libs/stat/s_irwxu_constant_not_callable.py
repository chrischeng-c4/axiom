# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "s_irwxu_constant_not_callable"
# subject = "stat.S_IRWXU"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IRWXU: s_irwxu_constant_not_callable (surface)."""
import stat

assert not callable(stat.S_IRWXU)
print("s_irwxu_constant_not_callable OK")
