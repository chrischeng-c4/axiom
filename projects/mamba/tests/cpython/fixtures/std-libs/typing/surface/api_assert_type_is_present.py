# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_assert_type_is_present"
# subject = "typing.assert_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.assert_type: api_assert_type_is_present (surface)."""
import typing

assert hasattr(typing, "assert_type")
print("api_assert_type_is_present OK")
