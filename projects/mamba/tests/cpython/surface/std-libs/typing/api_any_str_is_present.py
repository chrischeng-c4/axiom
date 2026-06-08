# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_any_str_is_present"
# subject = "typing.AnyStr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AnyStr: api_any_str_is_present (surface)."""
import typing

assert hasattr(typing, "AnyStr")
print("api_any_str_is_present OK")
