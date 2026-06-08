# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "interactive_interpreter_is_callable"
# subject = "code.InteractiveInterpreter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter: interactive_interpreter_is_callable (surface)."""
import code

assert callable(code.InteractiveInterpreter)
print("interactive_interpreter_is_callable OK")
