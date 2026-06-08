# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "surface"
# case = "compile_file_is_callable"
# subject = "compileall.compile_file"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file_is_callable (surface)."""
import compileall

assert callable(compileall.compile_file)
print("compile_file_is_callable OK")
