# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "recursive_marker_rendered"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: a self-referential list is rendered with a <Recursion ...> marker instead of looping forever"""
import pprint

# A cyclic structure must terminate: pprint detects the back-reference and
# renders a <Recursion on ...> marker rather than recursing forever.
lst: list = [1, 2]
lst.append(lst)
out = pprint.pformat(lst)
assert "Recursion" in out, out
print("recursive_marker_rendered OK")
