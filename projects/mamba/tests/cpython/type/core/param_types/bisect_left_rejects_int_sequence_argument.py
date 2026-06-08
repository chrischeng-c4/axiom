# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "bisect_left_rejects_int_sequence_argument"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""bisect.bisect_left: bisect_left_rejects_int_sequence_argument (errors)."""
import bisect

try:
    result = bisect.bisect_left(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
