# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_protocol_is_present"
# subject = "typing.Protocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Protocol: api_protocol_is_present (surface)."""
import typing

assert hasattr(typing, "Protocol")
print("api_protocol_is_present OK")
