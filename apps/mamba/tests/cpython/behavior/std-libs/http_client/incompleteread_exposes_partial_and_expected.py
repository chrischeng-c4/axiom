# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "incompleteread_exposes_partial_and_expected"
# subject = "http.client.IncompleteRead"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.IncompleteRead: IncompleteRead(partial, expected) stores the bytes read so far on .partial and the missing count on .expected"""
import http.client as hc

ir = hc.IncompleteRead(b"got_some", 100)
assert ir.partial == b"got_some", f"partial = {ir.partial!r}"
assert ir.expected == 100, f"expected = {ir.expected!r}"
assert isinstance(ir, hc.HTTPException), "IncompleteRead is an HTTPException"

print("incompleteread_exposes_partial_and_expected OK")
