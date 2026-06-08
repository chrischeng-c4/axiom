# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "extract_tb_is_callable"
# subject = "traceback.extract_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb_is_callable (surface)."""
import traceback

assert callable(traceback.extract_tb)
print("extract_tb_is_callable OK")
