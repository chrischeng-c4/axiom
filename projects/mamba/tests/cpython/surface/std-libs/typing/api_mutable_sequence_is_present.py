# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_mutable_sequence_is_present"
# subject = "typing.MutableSequence"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.MutableSequence: api_mutable_sequence_is_present (surface)."""
import typing

assert hasattr(typing, "MutableSequence")
print("api_mutable_sequence_is_present OK")
