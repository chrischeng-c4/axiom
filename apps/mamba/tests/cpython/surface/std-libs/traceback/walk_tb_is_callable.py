# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "walk_tb_is_callable"
# subject = "traceback.walk_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_tb: walk_tb_is_callable (surface)."""
import traceback

assert callable(traceback.walk_tb)
print("walk_tb_is_callable OK")
