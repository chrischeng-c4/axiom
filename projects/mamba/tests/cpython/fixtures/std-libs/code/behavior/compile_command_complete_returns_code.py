# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "compile_command_complete_returns_code"
# subject = "code.compile_command"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command returns a types.CodeType object for complete source ('1 + 2'), not None"""
import code
import types

_cc = code.compile_command("1 + 2")
assert _cc is not None, "complete source compiles"
assert isinstance(_cc, types.CodeType), f"code object: {type(_cc)!r}"

print("compile_command_complete_returns_code OK")
