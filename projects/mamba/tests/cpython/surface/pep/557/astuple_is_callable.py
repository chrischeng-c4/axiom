# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "astuple_is_callable"
# subject = "dataclasses.astuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.astuple: astuple_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.astuple)
print("astuple_is_callable OK")
