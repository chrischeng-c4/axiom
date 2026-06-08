# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_no_type_check_decorator_is_present"
# subject = "typing.no_type_check_decorator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.no_type_check_decorator: api_no_type_check_decorator_is_present (surface)."""
import typing

assert hasattr(typing, "no_type_check_decorator")
print("api_no_type_check_decorator_is_present OK")
