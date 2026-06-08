# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_byte_string_is_present"
# subject = "typing.ByteString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ByteString: api_byte_string_is_present (surface)."""
import typing

assert hasattr(typing, "ByteString")
print("api_byte_string_is_present OK")
