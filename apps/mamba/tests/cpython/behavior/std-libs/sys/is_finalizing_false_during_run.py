# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "is_finalizing_false_during_run"
# subject = "sys.is_finalizing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.is_finalizing: sys.is_finalizing() is False during normal (non-shutdown) execution"""
import sys

assert sys.is_finalizing() is False, f"is_finalizing = {sys.is_finalizing()!r}"
print("is_finalizing_false_during_run OK")
