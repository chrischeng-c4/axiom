# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "errors"
# case = "compile_command_bad_syntax_raises"
# subject = "code.compile_command"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command_bad_syntax_raises (errors)."""
import code

_raised = False
try:
    code.compile_command("def (:")
except SyntaxError:
    _raised = True
assert _raised, "compile_command_bad_syntax_raises: expected SyntaxError"
print("compile_command_bad_syntax_raises OK")
