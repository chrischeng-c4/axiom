# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_abstract_set_is_present"
# subject = "typing.AbstractSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AbstractSet: api_abstract_set_is_present (surface)."""
import typing

assert hasattr(typing, "AbstractSet")
print("api_abstract_set_is_present OK")
