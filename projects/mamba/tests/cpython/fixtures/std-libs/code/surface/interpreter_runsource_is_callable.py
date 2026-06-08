# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "interpreter_runsource_is_callable"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: interpreter_runsource_is_callable (surface)."""
import code

assert callable(code.InteractiveInterpreter.runsource)
print("interpreter_runsource_is_callable OK")
