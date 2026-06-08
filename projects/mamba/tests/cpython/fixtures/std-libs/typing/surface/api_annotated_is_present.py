# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_annotated_is_present"
# subject = "typing.Annotated"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Annotated: api_annotated_is_present (surface)."""
import typing

assert hasattr(typing, "Annotated")
print("api_annotated_is_present OK")
