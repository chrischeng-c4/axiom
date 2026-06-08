# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_sequence_is_present"
# subject = "typing.Sequence"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Sequence: api_sequence_is_present (surface)."""
import typing

assert hasattr(typing, "Sequence")
print("api_sequence_is_present OK")
