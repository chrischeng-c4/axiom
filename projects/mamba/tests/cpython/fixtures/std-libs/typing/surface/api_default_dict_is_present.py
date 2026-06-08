# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_default_dict_is_present"
# subject = "typing.DefaultDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.DefaultDict: api_default_dict_is_present (surface)."""
import typing

assert hasattr(typing, "DefaultDict")
print("api_default_dict_is_present OK")
