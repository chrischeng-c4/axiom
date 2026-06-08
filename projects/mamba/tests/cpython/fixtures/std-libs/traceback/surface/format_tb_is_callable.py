# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "format_tb_is_callable"
# subject = "traceback.format_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_tb: format_tb_is_callable (surface)."""
import traceback

assert callable(traceback.format_tb)
print("format_tb_is_callable OK")
