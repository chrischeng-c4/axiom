# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "name_lookup_round_trip"
# subject = "unicodedata.lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.lookup: name() and lookup() are inverses over a sample of named characters (A Z 0 9 e-acute n-tilde alpha)"""
import unicodedata

for _ch in ["A", "Z", "0", "9", "é", "ñ", "α"]:
    _nm = unicodedata.name(_ch, None)
    assert _nm is not None, f"sample char {_ch!r} should be named"
    assert unicodedata.lookup(_nm) == _ch, f"round-trip {_ch!r} via {_nm!r}"

print("name_lookup_round_trip OK")
