# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "surface"
# case = "main_is_callable"
# subject = "compileall.main"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.main: main_is_callable (surface)."""
import compileall

assert callable(compileall.main)
print("main_is_callable OK")
