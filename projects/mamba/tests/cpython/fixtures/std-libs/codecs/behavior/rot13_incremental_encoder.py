# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "rot13_incremental_encoder"
# subject = "codecs.getincrementalencoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getincrementalencoder: rot-13 is a text transform reached via the incremental encoder: getincrementalencoder('rot-13')().encode('ABBA nag Cheryl Baker') is 'NOON ant Purely Onxre'"""
import codecs

_rot = codecs.getincrementalencoder("rot-13")().encode("ABBA nag Cheryl Baker")
assert _rot == "NOON ant Purely Onxre", f"rot-13 = {_rot!r}"

print("rot13_incremental_encoder OK")
