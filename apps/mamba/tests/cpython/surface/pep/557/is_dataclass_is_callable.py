# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "is_dataclass_is_callable"
# subject = "dataclasses.is_dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.is_dataclass: is_dataclass_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.is_dataclass)
print("is_dataclass_is_callable OK")
