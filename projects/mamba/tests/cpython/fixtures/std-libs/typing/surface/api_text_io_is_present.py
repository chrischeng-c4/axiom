# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_text_io_is_present"
# subject = "typing.TextIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TextIO: api_text_io_is_present (surface)."""
import typing

assert hasattr(typing, "TextIO")
print("api_text_io_is_present OK")
