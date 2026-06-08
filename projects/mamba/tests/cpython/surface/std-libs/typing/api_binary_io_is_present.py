# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_binary_io_is_present"
# subject = "typing.BinaryIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.BinaryIO: api_binary_io_is_present (surface)."""
import typing

assert hasattr(typing, "BinaryIO")
print("api_binary_io_is_present OK")
