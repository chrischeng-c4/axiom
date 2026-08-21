# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "missing_sentinel_present"
# subject = "dataclasses.MISSING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.MISSING: missing_sentinel_present (surface)."""
import dataclasses

assert not callable(dataclasses.MISSING)
print("missing_sentinel_present OK")
