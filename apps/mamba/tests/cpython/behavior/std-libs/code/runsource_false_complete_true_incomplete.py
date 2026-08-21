# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_false_complete_true_incomplete"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: runsource returns False for complete source ('z = 10') and True for incomplete source ('def foo():') that needs more input"""
import code

_interp = code.InteractiveInterpreter({})
assert _interp.runsource("z = 10") is False, "complete source -> False"
assert _interp.runsource("def foo():") is True, "incomplete source -> True (more input)"

print("runsource_false_complete_true_incomplete OK")
