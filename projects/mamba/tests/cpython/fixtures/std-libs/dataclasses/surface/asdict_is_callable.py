# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "asdict_is_callable"
# subject = "dataclasses.asdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.asdict: asdict_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.asdict)
print("asdict_is_callable OK")
