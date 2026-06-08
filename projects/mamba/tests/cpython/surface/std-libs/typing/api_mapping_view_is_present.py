# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_mapping_view_is_present"
# subject = "typing.MappingView"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.MappingView: api_mapping_view_is_present (surface)."""
import typing

assert hasattr(typing, "MappingView")
print("api_mapping_view_is_present OK")
