# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "make_dataclass_is_callable"
# subject = "dataclasses.make_dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.make_dataclass)
print("make_dataclass_is_callable OK")
