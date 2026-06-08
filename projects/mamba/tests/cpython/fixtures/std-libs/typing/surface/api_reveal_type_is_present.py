# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_reveal_type_is_present"
# subject = "typing.reveal_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.reveal_type: api_reveal_type_is_present (surface)."""
import typing

assert hasattr(typing, "reveal_type")
print("api_reveal_type_is_present OK")
