# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "extract_tb_none_returns_empty_summary"
# subject = "traceback.extract_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb(None) returns a StackSummary of length 0 (no frames)"""
import traceback

assert len(traceback.extract_tb(None)) == 0

print("extract_tb_none_returns_empty_summary OK")
