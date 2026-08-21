# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "badstatusline_carries_offending_line"
# subject = "http.client.BadStatusLine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.BadStatusLine: BadStatusLine(line) is raisable and its string form carries the offending status line text"""
import http.client as hc

raised = False
try:
    raise hc.BadStatusLine("custom status line")
except hc.BadStatusLine as e:
    raised = True
    assert "custom status line" in str(e), f"BadStatusLine str = {str(e)!r}"
    assert isinstance(e, hc.HTTPException), "BadStatusLine is an HTTPException"
assert raised, "BadStatusLine was raised and caught"

print("badstatusline_carries_offending_line OK")
