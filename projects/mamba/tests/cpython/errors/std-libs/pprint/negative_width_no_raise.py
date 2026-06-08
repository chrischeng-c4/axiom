# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_width_no_raise"
# subject = "pprint.PrettyPrinter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.PrettyPrinter: PrettyPrinter(width=-1) does NOT raise under CPython 3.12 (only depth/indent and zero width/depth are validated); constructing it succeeds"""
import pprint

# Unlike depth/indent and a zero width/depth, a negative width is NOT
# validated by the constructor, so this must succeed without raising.
pp = pprint.PrettyPrinter(width=-1)
assert isinstance(pp, pprint.PrettyPrinter)
print("negative_width_no_raise OK")
