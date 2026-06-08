# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "surface"
# case = "dataclass_is_callable"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.dataclass: dataclass_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.dataclass)
print("dataclass_is_callable OK")
