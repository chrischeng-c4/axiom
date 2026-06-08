# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "ifloordiv_attr_exists"
# subject = "operator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator: ifloordiv_attr_exists (surface)."""
import operator

assert hasattr(operator, "ifloordiv")
print("ifloordiv_attr_exists OK")
