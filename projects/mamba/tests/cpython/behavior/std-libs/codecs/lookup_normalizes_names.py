# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "lookup_normalizes_names"
# subject = "codecs.lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.lookup: codecs.lookup normalizes codec names: lookup('utf-8'), lookup('UTF-8'), lookup('utf_8') all report .name == 'utf-8'"""
import codecs

_a = codecs.lookup("utf-8")
_b = codecs.lookup("UTF-8")
_c = codecs.lookup("utf_8")
assert _a.name == _b.name == _c.name == "utf-8", \
    f"normalized names: {_a.name!r} {_b.name!r} {_c.name!r}"

print("lookup_normalizes_names OK")
