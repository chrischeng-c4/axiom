# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_mutable_mapping_is_present"
# subject = "typing.MutableMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.MutableMapping: api_mutable_mapping_is_present (surface)."""
import typing

assert hasattr(typing, "MutableMapping")
print("api_mutable_mapping_is_present OK")
