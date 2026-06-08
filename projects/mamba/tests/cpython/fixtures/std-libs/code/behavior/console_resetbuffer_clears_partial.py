# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_resetbuffer_clears_partial"
# subject = "code.InteractiveConsole.resetbuffer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.resetbuffer: resetbuffer discards a partial multi-line statement so the next complete push ('y = 7') starts fresh and returns False"""
import code

_cons = code.InteractiveConsole({})
_cons.push("for i in range(10):")
_cons.resetbuffer()
# After reset the buffered 'for' header is gone, so a fresh complete statement
# completes immediately rather than being parsed as the loop body.
assert _cons.push("y = 7") is False, "fresh statement completes after reset"

print("console_resetbuffer_clears_partial OK")
