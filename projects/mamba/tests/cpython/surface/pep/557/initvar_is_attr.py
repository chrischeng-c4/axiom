# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "initvar_is_attr"
# subject = "dataclasses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses: initvar_is_attr (surface)."""
import dataclasses

assert hasattr(dataclasses, "InitVar")
print("initvar_is_attr OK")
