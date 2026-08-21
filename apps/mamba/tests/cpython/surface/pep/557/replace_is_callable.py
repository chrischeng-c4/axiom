# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "replace_is_callable"
# subject = "dataclasses.replace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.replace: replace_is_callable (surface)."""
import dataclasses

assert callable(dataclasses.replace)
print("replace_is_callable OK")
