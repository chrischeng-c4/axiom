# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "version_info_indexable_and_named"
# subject = "sys.version_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.version_info: version_info is a 5-field named tuple: index i equals the named field, it slices to a tuple, and compares as a tuple (> (1, 0, 0))"""
import sys

vi = sys.version_info
assert len(vi) == 5, f"version_info len = {len(vi)!r}"
assert isinstance(vi[:], tuple), "version_info slice is a tuple"
assert vi[0] == vi.major, "vi[0] == major"
assert vi[1] == vi.minor, "vi[1] == minor"
assert vi[2] == vi.micro, "vi[2] == micro"
assert vi[3] == vi.releaselevel, "vi[3] == releaselevel"
assert vi[4] == vi.serial, "vi[4] == serial"
assert vi > (1, 0, 0), "version_info compares as a tuple"
print("version_info_indexable_and_named OK")
