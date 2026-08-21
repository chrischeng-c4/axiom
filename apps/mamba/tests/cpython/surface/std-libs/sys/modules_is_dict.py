# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "modules_is_dict"
# subject = "sys.modules"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.modules: modules_is_dict (surface)."""
import sys

assert type(sys.modules).__name__ == "dict"
print("modules_is_dict OK")
