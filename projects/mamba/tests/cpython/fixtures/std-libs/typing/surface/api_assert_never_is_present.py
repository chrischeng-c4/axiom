# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_assert_never_is_present"
# subject = "typing.assert_never"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.assert_never: api_assert_never_is_present (surface)."""
import typing

assert hasattr(typing, "assert_never")
print("api_assert_never_is_present OK")
