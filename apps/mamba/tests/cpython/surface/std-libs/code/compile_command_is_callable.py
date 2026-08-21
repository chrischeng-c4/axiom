# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "compile_command_is_callable"
# subject = "code.compile_command"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command_is_callable (surface)."""
import code

assert callable(code.compile_command)
print("compile_command_is_callable OK")
