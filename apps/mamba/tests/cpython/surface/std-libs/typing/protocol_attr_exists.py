# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "protocol_attr_exists"
# subject = "typing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: protocol_attr_exists (surface)."""
import typing

assert hasattr(typing, "Protocol")
print("protocol_attr_exists OK")
