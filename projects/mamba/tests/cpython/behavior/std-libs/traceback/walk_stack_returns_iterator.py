# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "walk_stack_returns_iterator"
# subject = "traceback.walk_stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_stack: walk_stack(None) starts from the current frame and returns an iterable (has __iter__)"""
import traceback

_gen = traceback.walk_stack(None)
assert hasattr(_gen, "__iter__"), f"walk_stack iterable: {_gen!r}"

print("walk_stack_returns_iterator OK")
