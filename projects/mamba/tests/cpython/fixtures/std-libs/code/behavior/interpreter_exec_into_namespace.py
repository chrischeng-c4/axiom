# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "interpreter_exec_into_namespace"
# subject = "code.InteractiveInterpreter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter: an interpreter built over an explicit namespace dict execs source into it: after runsource('x = 42') the namespace holds x == 42"""
import code
import builtins

_ns = {"__builtins__": builtins}
_interp = code.InteractiveInterpreter(_ns)
_interp.runsource("x = 42")
assert _ns.get("x") == 42, f"namespace x = {_ns.get('x')!r}"

print("interpreter_exec_into_namespace OK")
