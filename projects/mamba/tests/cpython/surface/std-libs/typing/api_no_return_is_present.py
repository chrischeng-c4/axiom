# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_no_return_is_present"
# subject = "typing.NoReturn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.NoReturn: api_no_return_is_present (surface)."""
import typing

assert hasattr(typing, "NoReturn")
print("api_no_return_is_present OK")
