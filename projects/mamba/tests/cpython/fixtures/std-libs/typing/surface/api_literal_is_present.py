# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_literal_is_present"
# subject = "typing.Literal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Literal: api_literal_is_present (surface)."""
import typing

assert hasattr(typing, "Literal")
print("api_literal_is_present OK")
