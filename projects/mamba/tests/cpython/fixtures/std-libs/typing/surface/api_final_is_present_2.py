# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_final_is_present_2"
# subject = "typing.final"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.final: api_final_is_present_2 (surface)."""
import typing

assert hasattr(typing, "final")
print("api_final_is_present_2 OK")
