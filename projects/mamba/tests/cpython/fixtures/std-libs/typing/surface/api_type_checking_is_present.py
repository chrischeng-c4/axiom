# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_type_checking_is_present"
# subject = "typing.TYPE_CHECKING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TYPE_CHECKING: api_type_checking_is_present (surface)."""
import typing

assert hasattr(typing, "TYPE_CHECKING")
print("api_type_checking_is_present OK")
