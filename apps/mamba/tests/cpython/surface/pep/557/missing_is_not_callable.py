# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "missing_is_not_callable"
# subject = "dataclasses.MISSING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.MISSING: missing_is_not_callable (surface)."""
import dataclasses

assert not callable(dataclasses.MISSING)
print("missing_is_not_callable OK")
