# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_push_false_for_complete"
# subject = "code.InteractiveConsole.push"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.push: InteractiveConsole.push returns False for complete statements: an assignment ('x = 42') and a bare expression ('1 + 1') each complete immediately"""
import code

_cons = code.InteractiveConsole({})
assert _cons.push("x = 42") is False, "assignment is a complete statement"
assert _cons.push("1 + 1") is False, "bare expression is a complete statement"

print("console_push_false_for_complete OK")
