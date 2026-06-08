# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "surface"
# case = "fields_is_callable"
# subject = "dataclasses.fields"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.fields: fields_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.fields)
print("fields_is_callable OK")
