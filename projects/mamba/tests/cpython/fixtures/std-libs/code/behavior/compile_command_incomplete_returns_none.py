# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "compile_command_incomplete_returns_none"
# subject = "code.compile_command"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command returns None for source that is syntactically incomplete and may still be continued ('if True:' / 'def f():')"""
import code

for src in ["if True:", "def f():"]:
    assert code.compile_command(src) is None, f"incomplete -> None: {src!r}"

print("compile_command_incomplete_returns_none OK")
