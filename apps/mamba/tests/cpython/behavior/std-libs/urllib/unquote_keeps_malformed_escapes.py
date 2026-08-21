# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_keeps_malformed_escapes"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: malformed percent sequences (%xab, %x, %) are left verbatim rather than raised on; mixed-case hex decodes case-insensitively"""
from urllib.parse import unquote, unquote_to_bytes

for bad in ("%xab", "%x", "%"):
    assert unquote(bad) == bad, f"unquote keeps malformed {bad!r}"
    assert unquote_to_bytes(bad) == bad.encode("ascii"), \
        f"unquote_to_bytes keeps malformed {bad!r}"
assert unquote_to_bytes("%Ab%eA") == b"\xab\xea", "mixed-case hex"

print("unquote_keeps_malformed_escapes OK")
