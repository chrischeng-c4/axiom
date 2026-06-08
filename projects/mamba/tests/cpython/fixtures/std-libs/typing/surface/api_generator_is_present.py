# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_generator_is_present"
# subject = "typing.Generator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Generator: api_generator_is_present (surface)."""
import typing

assert hasattr(typing, "Generator")
print("api_generator_is_present OK")
