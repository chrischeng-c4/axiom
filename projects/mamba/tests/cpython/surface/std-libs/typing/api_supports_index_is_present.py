# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_supports_index_is_present"
# subject = "typing.SupportsIndex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.SupportsIndex: api_supports_index_is_present (surface)."""
import typing

assert hasattr(typing, "SupportsIndex")
print("api_supports_index_is_present OK")
