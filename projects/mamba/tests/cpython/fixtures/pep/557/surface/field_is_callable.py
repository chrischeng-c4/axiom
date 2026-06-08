# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "field_is_callable"
# subject = "dataclasses.field"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.field: field_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.field)
print("field_is_callable OK")
