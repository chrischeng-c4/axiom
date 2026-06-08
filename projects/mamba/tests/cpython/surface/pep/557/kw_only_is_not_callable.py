# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "kw_only_is_not_callable"
# subject = "dataclasses.KW_ONLY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.KW_ONLY: kw_only_is_not_callable (surface)."""
import dataclasses

assert not callable(dataclasses.KW_ONLY)
print("kw_only_is_not_callable OK")
