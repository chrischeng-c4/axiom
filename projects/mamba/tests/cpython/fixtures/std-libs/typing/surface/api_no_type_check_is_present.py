# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_no_type_check_is_present"
# subject = "typing.no_type_check"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.no_type_check: api_no_type_check_is_present (surface)."""
import typing

assert hasattr(typing, "no_type_check")
print("api_no_type_check_is_present OK")
