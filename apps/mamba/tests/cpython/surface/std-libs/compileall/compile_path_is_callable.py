# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "surface"
# case = "compile_path_is_callable"
# subject = "compileall.compile_path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_path: compile_path_is_callable (surface)."""
import compileall

assert callable(compileall.compile_path)
print("compile_path_is_callable OK")
