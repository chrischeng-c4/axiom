# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "chr_lone_surrogate_roundtrip"
# subject = "builtins.chr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Lone surrogate strings created by chr() round-trip like CPython."""

s = chr(0xD800)

assert len(s) == 1
assert ord(s) == 0xD800
assert repr(s) == "'\\ud800'"
assert ascii(s) == "'\\ud800'"
assert s == chr(0xD800)
assert hash(s) == hash(chr(0xD800))

d = {s: "surrogate"}
assert d[chr(0xD800)] == "surrogate"
assert list(d.keys())[0] == s

print("chr_lone_surrogate_roundtrip OK")
