# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_caches_size_and_format"
# subject = "struct.Struct"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: a compiled Struct('>IB') caches .size == 5 and .format == '>IB'; a Struct built from a bytes format normalizes .format back to the same str"""
import struct

# A compiled Struct caches its size and format.
s = struct.Struct(">IB")
assert s.size == 5, f"size = {s.size!r}"
assert s.format == ">IB", f"format = {s.format!r}"

# A Struct built from a bytes format normalizes .format back to the same str.
s2 = struct.Struct(s.format.encode())
assert s2.format == s.format, "bytes format normalizes to the same string"

print("struct_caches_size_and_format OK")
