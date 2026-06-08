# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_pattern_is_present"
# subject = "typing.Pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Pattern: api_pattern_is_present (surface)."""
import typing

assert hasattr(typing, "Pattern")
print("api_pattern_is_present OK")
