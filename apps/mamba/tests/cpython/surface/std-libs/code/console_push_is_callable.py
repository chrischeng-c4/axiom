# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "console_push_is_callable"
# subject = "code.InteractiveConsole.push"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.push: console_push_is_callable (surface)."""
import code

assert callable(code.InteractiveConsole.push)
print("console_push_is_callable OK")
