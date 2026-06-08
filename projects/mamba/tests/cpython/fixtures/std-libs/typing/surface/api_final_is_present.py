# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_final_is_present"
# subject = "typing.Final"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Final: api_final_is_present (surface)."""
import typing

assert hasattr(typing, "Final")
print("api_final_is_present OK")
