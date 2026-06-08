# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_push_true_then_blank_completes"
# subject = "code.InteractiveConsole.push"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.push: push returns True while a compound statement is incomplete ('if True:'), accepts the indented body as a bool, and a final blank line completes the block (returns False)"""
import code

_cons = code.InteractiveConsole({})
assert _cons.push("if True:") is True, "compound header is incomplete"
# After the indented body push still returns a bool (more input may be wanted).
assert isinstance(_cons.push("    x = 1"), bool), "indented line returns bool"
# A blank line terminates the compound statement, completing the block.
assert _cons.push("") is False, "blank line completes the block"

print("console_push_true_then_blank_completes OK")
