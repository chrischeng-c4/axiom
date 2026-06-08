# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_subclasses_interpreter"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: InteractiveConsole is a subclass of InteractiveInterpreter, so an instance isinstance-checks as both"""
import code

assert issubclass(code.InteractiveConsole, code.InteractiveInterpreter), \
    "InteractiveConsole subclasses InteractiveInterpreter"
_cons = code.InteractiveConsole({"y": 99})
assert isinstance(_cons, code.InteractiveConsole), "instance is a Console"
assert isinstance(_cons, code.InteractiveInterpreter), "instance is also an Interpreter"

print("console_subclasses_interpreter OK")
