# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "surface"
# case = "compile_dir_is_callable"
# subject = "compileall.compile_dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir_is_callable (surface)."""
import compileall

assert callable(compileall.compile_dir)
print("compile_dir_is_callable OK")
