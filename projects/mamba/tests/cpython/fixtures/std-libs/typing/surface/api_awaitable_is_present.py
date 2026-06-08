# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_awaitable_is_present"
# subject = "typing.Awaitable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Awaitable: api_awaitable_is_present (surface)."""
import typing

assert hasattr(typing, "Awaitable")
print("api_awaitable_is_present OK")
