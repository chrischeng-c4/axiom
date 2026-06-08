# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "interpreter_runcode_is_callable"
# subject = "code.InteractiveInterpreter.runcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runcode: interpreter_runcode_is_callable (surface)."""
import code

assert callable(code.InteractiveInterpreter.runcode)
print("interpreter_runcode_is_callable OK")
