# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_literal_string_is_present"
# subject = "typing.LiteralString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.LiteralString: api_literal_string_is_present (surface)."""
import typing

assert hasattr(typing, "LiteralString")
print("api_literal_string_is_present OK")
